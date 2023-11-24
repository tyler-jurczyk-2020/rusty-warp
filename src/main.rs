use std::{net::SocketAddr, path::PathBuf, collections::HashMap, sync::{Arc, Mutex}, cell::RefCell};
use warp::{Filter, filters::{ws::{Message, Ws, WebSocket}, body}, reply::Reply};
use futures::{StreamExt, FutureExt, SinkExt, TryFutureExt, Future};
use warp::hyper::body::Bytes;
use tokio::sync::watch;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Data {
    play : Vec<f64>
}

impl Data {
    fn new() -> Data {
        Data {play : Vec::new()}
    }
}

#[tokio::main]
async fn main() {
    let socket : SocketAddr = "127.0.0.1:7878".parse().unwrap();
    let mut path = PathBuf::new();
    let current_dir = std::env::current_dir().unwrap().to_string_lossy().to_string();
    path.push(&current_dir);
    path.push("webpage");
    let data : Arc<Mutex<Data>> = Arc::new(Mutex::new(Data::new()));
    let stuff = warp::any().map(move || data.clone());
    let (tx, mut rx) = watch::channel("New data from python");
    let single = Arc::new(tx); 
    let python_broadcast = warp::any().map(move || single.clone());
    let browser_listener = warp::any().map(move || rx.clone());
    let filter = warp::get()
        .and(warp::fs::dir(path));
    let python = warp::path!("python")
        .and(warp::post())
        .and(warp::body::json())
        .and(stuff.clone())
        .and(python_broadcast)
        .and_then(handle_python);
    let websocket = warp::path!("ws")
        .and(warp::ws())
        .and(stuff)
        .and(browser_listener)
        .map(|ws : warp::ws::Ws, data, listener| {
            ws.on_upgrade(move |ws| handle_websocket(ws, data, listener))
        });
    warp::serve(filter.or(python).or(websocket)).run(socket).await; 
}

async fn handle_websocket(ws : warp::ws::WebSocket, data : Arc<Mutex<Data>>, mut listener : watch::Receiver<&str>) {
    let (mut sender, mut receiver) = ws.split();
    loop {
        listener.changed().await.unwrap();
        println!("Websocket waking up");
        let mut stream;
        {
            let b = data.lock().unwrap();
            stream = futures::stream::iter(b.play.clone().into_iter().map(|v|Ok(Message::text(v.to_string()))));
        }
        sender.send_all(&mut stream).await.unwrap();
        //sender.send(Message::text("alrighy")).await;
    }
}

async fn handle_python(body : serde_json::Value, data : Arc<Mutex<Data>>, sender : Arc<watch::Sender<&str>>) -> Result<impl Reply, warp::Rejection> {
    let incoming_data = match serde_json::from_value::<Data>(body.clone()) {
        Ok(v) => v,
        Err(e) => panic!("Failed to load python data: {}", e) 
    };
    let mut b = data.lock().unwrap();
    b.play.clear();
    b.play.extend_from_slice(&incoming_data.play);
    println!("Signaling websocket");
    sender.send("bre");
    Ok(warp::reply::html(""))
}
