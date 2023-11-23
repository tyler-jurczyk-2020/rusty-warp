use std::{net::SocketAddr, path::PathBuf};
use warp::Filter;
use futures::{StreamExt, FutureExt};

#[tokio::main]
async fn main() {
    let socket : SocketAddr = "127.0.0.1:7878".parse().unwrap();
    let mut path = PathBuf::new();
    let current_dir = std::env::current_dir().unwrap().to_string_lossy().to_string();
    path.push(&current_dir);
    path.push("webpage");
    println!("{path:?}");
    let filter = warp::get()
        .and(warp::fs::dir(path));
    let python = warp::path!("python")
        .and(warp::post())
        .and(warp::body::bytes())
        .map(|body| {
            println!("Data received: {:?}", body);
            warp::reply()
    });
    let websocket = warp::path!("ws")
        .and(warp::ws())
        .map(|ws : warp::ws::Ws| {
            ws.on_upgrade(|websocket| {
                let (tx, rx) = websocket.split();
                rx.forward(tx).map(|res| {
                    if let Err(e) = res {
                        println!("ERROR");
                    }
                })
            })
        });
    warp::serve(filter.or(python).or(websocket)).run(socket).await; 
}
