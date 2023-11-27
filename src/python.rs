use std::{sync::{Arc, Mutex, mpsc::SyncSender}, collections::HashMap, time::Duration};
use futures::{StreamExt, SinkExt, stream::{SplitSink, SplitStream}, Future};
use tokio::sync::{watch::{Receiver, self, Sender}, mpsc::{self, UnboundedReceiver, UnboundedSender}};
use warp::{filters::ws::{Message, WebSocket}, Filter, reject::Rejection, reply::Reply};

use crate::{data::{GlobalComms, Data, ExternMessage, GENERATE, InternMessage}, events::run_match, shared::{outgoing_thread, main_thread}};




async fn handle_python_websocket(ws : warp::ws::WebSocket, data : Arc<Mutex<Data>>, comms : Arc<Mutex<GlobalComms>>) {
    let (mut sender, mut receiver) = ws.split();
    // Receives data from browser throught this channel
    let (tx, mut rx) = mpsc::unbounded_channel::<InternMessage>();
    let (tx_brow, mut rx_brow) = mpsc::unbounded_channel::<InternMessage>();
    let mut c = comms.lock().unwrap();
    c.send_to_py = Some(tx.clone());
    c.recv_from_py = Some(rx_brow);
    c.send_to_brow = Some(tx_brow.clone());
    let brow_cln = tx_brow.clone();
    let py_cln = tx.clone();
    tokio::spawn(async move {
        incoming_python_thread(receiver, py_cln, brow_cln).await;
    });

    tokio::spawn(async move {
        outgoing_thread(sender, rx).await;
    }); 
    
    println!("Creating Main Thread");
    tokio::spawn(async move {
       main_thread(data, tx_brow, tx).await 
    });
    
    // Check result of tokio spawns and disconnect properly
}

async fn incoming_python_thread(mut receiver : SplitStream<WebSocket>, mut tx_out_py : UnboundedSender<InternMessage>, tx_out_b : UnboundedSender<InternMessage>) {
    loop {
        if let Some(m) = receiver.next().await {
            println!("Incoming message from python!")
        }
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

