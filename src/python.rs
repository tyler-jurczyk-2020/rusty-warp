use std::{sync::{Arc, Mutex, mpsc::SyncSender}, collections::HashMap, time::Duration};
use futures::{StreamExt, SinkExt, stream::{SplitSink, SplitStream}, Future};
use tokio::sync::{watch::{Receiver, self, Sender}, mpsc::{self, UnboundedReceiver, UnboundedSender}};
use warp::{filters::ws::{Message, WebSocket}, Filter, reject::Rejection, reply::Reply};

use crate::{data::{GlobalComms, Data, ExternMessage, GENERATE}, events::run_match};




async fn handle_python_websocket(ws : warp::ws::WebSocket, data : Arc<Mutex<Data>>, comms : Arc<Mutex<GlobalComms>>) {
    let (mut sender, mut receiver) = ws.split();
    // Receives data from browser throught this channel
    let (tx, mut rx) = mpsc::unbounded_channel::<()>();
    let (watch_tx, watch_rx) = watch::channel(0);
    watch_tx.send(1);
    let mut c = comms.lock().unwrap();
    c.recv_from_py = Some(watch_rx); 
    c.send_to_py = Some(tx);
    let cln = data.clone();
    tokio::spawn(async move {
        browser_thread(cln, rx).await 
    });
    println!("Creating Main Thread");
    tokio::spawn(async move {
       main_thread(data, sender, receiver, watch_tx).await 
    });
    
    // Check result of tokio spawns and disconnect properly
}

async fn main_thread(data : Arc<Mutex<Data>>, mut sender : SplitSink<WebSocket, Message>, mut receiver : SplitStream<WebSocket>,
                     watch_tx : Sender<usize>) {
        let data_handle = data.clone();
        let request_data;
        {
            let data_hold = data_handle.lock().unwrap();
            let data_to_ser = (data_hold.player_count, data_hold.batch_size, &data_hold.players);
            request_data = ExternMessage::new(GENERATE.to_string(), data_to_ser);
        }
        request_data.send_message(&mut sender).await;

        if let Some(m) = receiver.next().await {
            println!("{m:?}");
            {
            let mut dh = data_handle.lock().unwrap();
            dh.batch = serde_json::from_str(m.unwrap().to_str().unwrap()).unwrap();
            println!("{:?}", dh.batch);
            }
            println!("Match starting soon...");
            //** Send data to browser to match **//
            tokio::time::sleep(Duration::new(5, 0)).await; 
            println!("Match starting now!");
            run_match(&watch_tx, data.clone()).await;
        }
}

async fn browser_thread(data : Arc<Mutex<Data>>, mut rx : UnboundedReceiver<()>) {
    loop {
        rx.recv().await;
        println!("Browser has something for me!");
    }
}

pub fn setup_python_ws(data : Arc<Mutex<Data>>, comms : Arc<Mutex<GlobalComms>>) ->  impl Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {
    let data_filter = warp::any().map(move || data.clone());
    let comms_filter = warp::any().map(move || comms.clone());
    let filter = warp::path!("python-ws")
        .and(warp::ws())
        .and(data_filter)
        .and(comms_filter)
        .map(|ws : warp::ws::Ws, d, c| {
            ws.on_upgrade(move |ws| handle_python_websocket(ws, d, c))
        });
    filter
}

