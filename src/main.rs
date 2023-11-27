use std::{net::SocketAddr, path::PathBuf, collections::HashMap, cell::RefCell, convert::Infallible, sync::{Arc, Mutex}, task::Poll, time::Duration};
use browser::setup_browser_ws;
use data::{Batch, Data, GlobalComms};
use python::{setup_python_ws};
use warp::{Filter, filters::{ws::{Message, Ws, WebSocket}, body}, reply::Reply, reject::Rejection};
use futures::{StreamExt, FutureExt, SinkExt, TryFutureExt, Future, future::Map};
use warp::hyper::body::Bytes;
use tokio::{sync::{mpsc::{self, UnboundedSender, UnboundedReceiver}, watch::{self, Sender, Receiver}}, stream};
use serde::{Serialize, Deserialize};

use crate::events::run_match;

mod data;
mod events;
mod python;
mod browser;
mod shared;


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


