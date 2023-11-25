use std::{net::SocketAddr, path::PathBuf, collections::HashMap, cell::RefCell, convert::Infallible, sync::{Arc, Mutex}, task::Poll};
use data::Batch;
use warp::{Filter, filters::{ws::{Message, Ws, WebSocket}, body}, reply::Reply, reject::Rejection};
use futures::{StreamExt, FutureExt, SinkExt, TryFutureExt, Future, future::Map};
use warp::hyper::body::Bytes;
use tokio::{sync::mpsc::{self, UnboundedSender, UnboundedReceiver}, stream};
use serde::{Serialize, Deserialize};

mod data;

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    batch : Batch,
    player_count : usize    
}

impl Data {
    fn new() -> Data {
        Data {batch : Batch::new(), player_count : 2}
    }
}

#[derive(Clone)]
enum Client {
    Javascript(UnboundedSender<()>),
    Python(UnboundedSender<()>)
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
    let clientele : Arc<Mutex<HashMap<u32, Client>>> = Arc::new(Mutex::new(HashMap::new()));

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

async fn handle_python_websocket(ws : warp::ws::WebSocket, data : Arc<Mutex<Data>>, clientele : Arc<Mutex<HashMap<u32, Client>>>) {
    let (mut sender, mut receiver) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<()>();

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
            let mut dh = data_handle.lock().unwrap();
            dh.batch = serde_json::from_str(m.unwrap().to_str().unwrap()).unwrap();
            println!("{:?}", dh.batch);
        }
    });
    // Check result of tokio spawns and disconnect properly
}

fn setup_python_ws(data : Arc<Mutex<Data>>, clientele : Arc<Mutex<HashMap<u32, Client>>>) ->  impl Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {
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

async fn handle_browser_websocket(ws : warp::ws::WebSocket, data : Arc<Mutex<Data>>, clientele : Arc<Mutex<HashMap<u32, Client>>>) {
    let (mut sender, mut receiver) = ws.split();
    // We are going to assume for now that python instance comes before anything else
    // Python is hardcoded to 0 and Browser is hardcoded to 1 for now as well
    let (tx, mut rx) = mpsc::unbounded_channel::<()>();
    let py_tx;
    {
        let mut c = clientele.lock().unwrap();  
        py_tx = c.remove(&0).unwrap();
        c.insert(1, Client::Python(tx));
    }
    if let Client::Python(s) = py_tx.clone() {
        // Wakeup python socket
        s.send(()); 
    };
    println!("Sent signal to python");
    tokio::spawn(async move {
        while let Some(_r) = rx.recv().await {
            sender.send(Message::text("Browser says hi!")); 
        }
    });
    tokio::spawn(async move {
       while let Some(_r) = receiver.next().await {
            if let Client::Javascript(ref s) = py_tx {
                println!("Got signal from browser");
                s.send(());
            } 
       } 
    });
    // Check result of tokio spawns and disconnect properly
}

fn setup_browser_ws(data : Arc<Mutex<Data>>, clientele : Arc<Mutex<HashMap<u32, Client>>>) -> impl Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {
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
