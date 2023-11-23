use std::{net::SocketAddr, path::PathBuf, collections::HashMap, sync::{Arc, Mutex}, cell::RefCell};
use warp::{Filter, filters::{ws::{Message, Ws, WebSocket}, body}, reply::Reply};
use futures::{StreamExt, FutureExt, SinkExt, TryFutureExt, Future};
use warp::hyper::body::Bytes;

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
    let filter = warp::get()
        .and(warp::fs::dir(path));
    let python = warp::path!("python")
        .and(warp::post())
        .and(warp::body::bytes())
        .and(stuff.clone())
        .and_then(handle_python);
    let websocket = warp::path!("ws")
        .and(warp::ws())
        .and(stuff)
        .map(|ws : warp::ws::Ws, data| {
            ws.on_upgrade(move |ws| handle_websocket(ws, data))
        });
    warp::serve(filter.or(python).or(websocket)).run(socket).await; 
}

async fn handle_websocket(ws : warp::ws::WebSocket, data : Arc<Mutex<Vec<u8>>>) {
    let (mut sender, mut receiver) = ws.split();
     
    while let Some(r) = receiver.next().await {
        let mail;
        {
            let b = data.lock().unwrap();
            mail = sender.send(Message::text(String::from_utf8(b.to_vec()).unwrap()));
        }
        mail.await;
    }
}

async fn handle_python(body : Bytes, data : Arc<Mutex<Vec<u8>>>) -> Result<impl Reply, warp::Rejection> {
    let body_vec : Vec<u8> = body.to_vec();
    let mut b = data.lock().unwrap();
    b.clear();
    b.extend_from_slice(&body.to_vec());
    println!("{:?}", String::from_utf8(b.to_vec()));
    Ok(warp::reply::html(""))
}
