use std::{net::SocketAddr, path::PathBuf, collections::HashMap, cell::RefCell, convert::Infallible, sync::{Arc, Mutex}, task::Poll, time::Duration};
use data::Batch;
use python::{setup_python_ws, GlobalComms};
use warp::{Filter, filters::{ws::{Message, Ws, WebSocket}, body}, reply::Reply, reject::Rejection};
use futures::{StreamExt, FutureExt, SinkExt, TryFutureExt, Future, future::Map};
use warp::hyper::body::Bytes;
use tokio::{sync::{mpsc::{self, UnboundedSender, UnboundedReceiver}, watch::{self, Sender, Receiver}}, stream};
use serde::{Serialize, Deserialize};

use crate::events::run_match;

mod data;
mod events;
mod python;

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    batch : Batch,
    player_count : usize,    
    batch_size : usize,
    players : Vec<Player>,
    display1 : f64,
    display2 : f64 
}

#[derive(Debug, Serialize, Deserialize)]
struct Player {
    mean : f64,
    std_dev : f64,
    photo : String 
}

impl Player {
    fn new(mean : f64, std_dev : f64, photo : String) -> Player {
        Player { mean, std_dev, photo}
    }
}

impl Data {
    fn new() -> Data {
        Data { 
            batch : Batch::new(),
            player_count : 2,
            batch_size : 40,
            display1 : 0.0,
            display2 : 0.0,
            players : vec![Player::new(0.2, 0.1, "a4.png".to_string()), Player::new(0.1, 0.1, "a2.png".to_string())]
        }
    }
}


const GENERATE : &str = "A3A3";

#[tokio::main]
async fn main() {
    let socket : SocketAddr = "127.0.0.1:7878".parse().unwrap();
    
    // Shared data
    let data : Arc<Mutex<Data>> = Arc::new(Mutex::new(Data::new()));
    let comms : Arc<Mutex<GlobalComms>> = Arc::new(Mutex::new(GlobalComms::new()));
    let python_filter = setup_python_ws(data.clone(), comms.clone());
    let browser_filter = setup_browser_ws(data, comms.clone());
    let mut fs_path = PathBuf::new(); 
    fs_path.push(std::env::current_dir().unwrap().to_string_lossy().to_string());
    fs_path.push("ui_prod");
    let main_filter = warp::get()
        .and(warp::fs::dir(fs_path));

    let mut path = PathBuf::new();
    let current_dir = std::env::current_dir().unwrap().to_string_lossy().to_string();
    path.push(&current_dir);
    path.push("images");
    path.push("players");
    let images = warp::path("players")
        .and(warp::fs::dir(path));
    warp::serve(main_filter.or(python_filter).or(browser_filter).or(images)).run(socket).await; 
}


async fn handle_browser_websocket(ws : warp::ws::WebSocket, data : Arc<Mutex<Data>>, comms : Arc<Mutex<GlobalComms>>) {
    let (mut sender, mut receiver) = ws.split();
    // We are going to assume for now that python instance comes before anything else
    // Python is hardcoded to 0 and Browser is hardcoded to 1 for now as well
    let (tx, mut rx) = mpsc::unbounded_channel::<()>();
    let mut c = comms.lock().unwrap();
    let mut python_reciever = c.recv_from_py.clone().unwrap();
    // Handle data received from python
    tokio::spawn(async move {
        loop {
           if let Ok(v) = python_reciever.changed().await {
                // We have some new data to process
                let val = python_reciever.borrow_and_update().clone();
                let data_to_send : (f64, f64, usize);
                {
                    let d = data.lock().unwrap();
                    data_to_send = (d.display1, d.display2, val);
                }
                sender.send(Message::text(serde_json::to_string(&data_to_send).unwrap())).await;
            }
        }
    });
    // Handle incoming browser messages
    let mut python_sender = c.send_to_py.clone().unwrap();
    tokio::spawn(async move {
        loop {
            if let Some(r) = receiver.next().await {
                println!("Caught!: {:?}", r);
                python_sender.send(());
            }
        }
    });
}

fn setup_browser_ws(data : Arc<Mutex<Data>>, comms : Arc<Mutex<GlobalComms>>) -> impl Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {
    let data_filter = warp::any().map(move || data.clone());
    let comms_filter = warp::any().map(move || comms.clone());
    let filter = warp::path!("browser-ws")
        .and(warp::ws())
        .and(data_filter)
        .and(comms_filter)
        .map(|ws : warp::ws::Ws, d, c| {
            ws.on_upgrade(move |ws| handle_browser_websocket(ws, d, c))
        });
    filter
}
