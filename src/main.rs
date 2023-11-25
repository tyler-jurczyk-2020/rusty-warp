use std::{net::SocketAddr, path::PathBuf, collections::HashMap, cell::RefCell, convert::Infallible, sync::{Arc, Mutex}, task::Poll, time::Duration};
use data::Batch;
use warp::{Filter, filters::{ws::{Message, Ws, WebSocket}, body}, reply::Reply, reject::Rejection};
use futures::{StreamExt, FutureExt, SinkExt, TryFutureExt, Future, future::Map};
use warp::hyper::body::Bytes;
use tokio::{sync::{mpsc::{self, UnboundedSender, UnboundedReceiver}, watch::{self, Sender, Receiver}}, stream};
use serde::{Serialize, Deserialize};

use crate::events::run_match;

mod data;
mod events;

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    batch : Batch,
    player_count : usize,    
    display1 : f64,
    display2 : f64 
}

impl Data {
    fn new() -> Data {
        Data {batch : Batch::new(), player_count : 2, display1 : 0.0, display2 : 0.0}
    }
}


const GENERATE : &str = "A3A3";

#[tokio::main]
async fn main() {
    let socket : SocketAddr = "127.0.0.1:7878".parse().unwrap();
    let mut path = PathBuf::new();
    let current_dir = std::env::current_dir().unwrap().to_string_lossy().to_string();
    path.push(&current_dir);
    path.push("webpage");
    // Shared data
    let data : Arc<Mutex<Data>> = Arc::new(Mutex::new(Data::new()));
    let clientele : Arc<Mutex<HashMap<u32, Receiver<usize>>>> = Arc::new(Mutex::new(HashMap::new()));

    let python_filter = setup_python_ws(data.clone(), clientele.clone());
    let browser_filter = setup_browser_ws(data, clientele);
    let main_filter = warp::get()
        .and(warp::fs::dir(path));

    let mut path = PathBuf::new();
    let current_dir = std::env::current_dir().unwrap().to_string_lossy().to_string();
    path.push(&current_dir);
    path.push("images");
    path.push("players");
    let images = warp::path("images")
        .and(warp::path("players"))
        .and(warp::fs::dir(path));
    warp::serve(main_filter.or(python_filter).or(browser_filter).or(images)).run(socket).await; 
}

async fn handle_python_websocket(ws : warp::ws::WebSocket, data : Arc<Mutex<Data>>, clientele : Arc<Mutex<HashMap<u32, Receiver<(usize)>>>>) {
    let (mut sender, mut receiver) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<()>();
    let (watch_tx, watch_rx) = watch::channel(0);
    watch_tx.send(1);
    let mut c = clientele.lock().unwrap();
    c.insert(0, watch_rx); 
    println!("Creating Main Thread");
    // Create main thread that manages state
    tokio::spawn(async move {
        let stream_send;
        let data_handle = data.clone();
        {
            let data_hold = data_handle.lock().unwrap();
            let mut stream_pre : Vec<Message> = Vec::new();
            stream_pre.push(Message::text(GENERATE));
            stream_pre.push(Message::text(serde_json::to_string(&data_hold.player_count).unwrap()));
            // Preflight message
            stream_send = futures::stream::iter(stream_pre);
        }
        sender.send_all(&mut stream_send.map(|v|Ok(v))).await;
        if let Some(m) = receiver.next().await {
            println!("{m:?}");
            {
            let mut dh = data_handle.lock().unwrap();
            dh.batch = serde_json::from_str(m.unwrap().to_str().unwrap()).unwrap();
            println!("{:?}", dh.batch);
            }
            println!("Match starting soon...");
            tokio::time::sleep(Duration::new(3, 0)); 
            run_match(&watch_tx, data.clone()).await;
        }
    });
    // Check result of tokio spawns and disconnect properly
}

fn setup_python_ws(data : Arc<Mutex<Data>>, clientele : Arc<Mutex<HashMap<u32, Receiver<usize>>>>) ->  impl Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {
    let data_filter = warp::any().map(move || data.clone());
    let clientele_filter = warp::any().map(move || clientele.clone());
    let filter = warp::path!("python-ws")
        .and(warp::ws())
        .and(data_filter)
        .and(clientele_filter)
        .map(|ws : warp::ws::Ws, d, c| {
            ws.on_upgrade(move |ws| handle_python_websocket(ws, d, c))
        });
    filter
}

async fn handle_browser_websocket(ws : warp::ws::WebSocket, data : Arc<Mutex<Data>>, clientele : Arc<Mutex<HashMap<u32, Receiver<usize>>>>) {
    let (mut sender, mut receiver) = ws.split();
    // We are going to assume for now that python instance comes before anything else
    // Python is hardcoded to 0 and Browser is hardcoded to 1 for now as well
    let (tx, mut rx) = mpsc::unbounded_channel::<()>();
    let mut c = clientele.lock().unwrap();
    let mut python_reciever = c.get(&0).unwrap().clone();
    tokio::spawn(async move {
        loop {
           if let Ok(v) = python_reciever.changed().await {
                // We have some new data to process
                let data_to_send : (f64, f64);
                {
                    let d = data.lock().unwrap();
                    data_to_send = (d.display1, d.display2);
                }
                sender.send(Message::text(serde_json::to_string(&data_to_send).unwrap())).await;
            }
        }
    });
}

fn setup_browser_ws(data : Arc<Mutex<Data>>, clientele : Arc<Mutex<HashMap<u32, Receiver<usize>>>>) -> impl Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {
    let data_filter = warp::any().map(move || data.clone());
    let clientele_filter = warp::any().map(move || clientele.clone());
    let filter = warp::path!("browser-ws")
        .and(warp::ws())
        .and(data_filter)
        .and(clientele_filter)
        .map(|ws : warp::ws::Ws, d, c| {
            ws.on_upgrade(move |ws| handle_browser_websocket(ws, d, c))
        });
    filter
}
