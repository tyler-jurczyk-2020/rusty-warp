use std::{net::SocketAddr, path::PathBuf, collections::HashMap, sync::{Arc, Mutex}, cell::RefCell};
use warp::{Filter, filters::{ws::{Message, Ws, WebSocket}, body}, reply::Reply};
use futures::{StreamExt, FutureExt, SinkExt, TryFutureExt, Future};
use warp::hyper::body::Bytes;
use tokio::sync::watch;

#[tokio::main]
async fn main() {
    let socket : SocketAddr = "127.0.0.1:7878".parse().unwrap();
    let mut path = PathBuf::new();
    let current_dir = std::env::current_dir().unwrap().to_string_lossy().to_string();
    path.push(&current_dir);
    path.push("webpage");
    println!("{path:?}");
    let data : Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let stuff = warp::any().map(move || data.clone());
    let (tx, mut rx) = watch::channel("New data from python");
    let single = Arc::new(tx); 
    let python_broadcast = warp::any().map(move || single.clone());
    let browser_listener = warp::any().map(move || rx.clone());
    let filter = warp::get()
        .and(warp::fs::dir(path));
    let python = warp::path!("python")
        .and(warp::post())
        .and(warp::body::bytes())
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

async fn handle_websocket(ws : warp::ws::WebSocket, data : Arc<Mutex<Vec<u8>>>, mut listener : watch::Receiver<&str>) {
    let (mut sender, mut receiver) = ws.split();
    loop {
        listener.changed().await;
        let mail;
        {
            let b = data.lock().unwrap();
            mail = sender.send(Message::text(String::from_utf8(b.to_vec()).unwrap()));
        }
        mail.await;
    }
}

async fn handle_python(body : Bytes, data : Arc<Mutex<Vec<u8>>>, sender : Arc<watch::Sender<&str>>) -> Result<impl Reply, warp::Rejection> {
    let body_vec : Vec<u8> = body.to_vec();
    let mut b = data.lock().unwrap();
    b.clear();
    b.extend_from_slice(&body.to_vec());
    println!("{:?}", String::from_utf8(b.to_vec()));
    sender.send("bre");
    Ok(warp::reply::html(""))
}
