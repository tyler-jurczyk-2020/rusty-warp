use std::{sync::{Arc, Mutex, mpsc::SyncSender}, collections::HashMap, time::Duration};
use futures::{StreamExt, SinkExt, stream::{SplitSink, SplitStream}, Future};
use tokio::sync::{watch::{Receiver, self, Sender}, mpsc::{self, UnboundedReceiver, UnboundedSender}};
use warp::{filters::ws::{Message, WebSocket}, Filter, reject::Rejection, reply::Reply};

use crate::{data::{GENERATE, messaging::{InternMessage, ExternMessage, GlobalComms}, gamedata::Data, SharedData}, events::run_match, shared::{outgoing_thread, main_thread}};




async fn handle_python_websocket(ws : warp::ws::WebSocket, shared_data : Arc<SharedData>) {
    let (mut sender, mut receiver) = ws.split();
    // Receives data from browser throught this channel
    let (tx, mut rx) = mpsc::unbounded_channel::<InternMessage>();
    // Browser comms
    let (tx_brow, mut rx_brow) = mpsc::unbounded_channel::<InternMessage>();
    // Main thread comms
    let (tx_mt, rx_mt) = mpsc::unbounded_channel::<InternMessage>();
    let mut c = shared_data.comms.lock().unwrap();
    c.send_to_py = Some(tx.clone());
    c.recv_from_py = Some(rx_brow);
    c.send_to_brow = Some(tx_brow.clone());
    c.send_to_main = Some(tx_mt.clone());
    let brow_cln = tx_brow.clone();
    let py_cln = tx.clone();

    tokio::spawn(async move {
        outgoing_thread(sender, rx).await;
    }); 
    
    println!("Creating Main Thread");
    let borrow_shared = shared_data.clone();
    tokio::spawn(async move {
       main_thread(borrow_shared, tx_brow, tx, rx_mt).await 
    });
    tokio::spawn(async move {
        incoming_python_thread(receiver, py_cln, brow_cln, tx_mt).await;
    });
    
    // Check result of tokio spawns and disconnect properly
}

async fn incoming_python_thread(mut receiver : SplitStream<WebSocket>, mut tx_out_py : UnboundedSender<InternMessage>, tx_out_b : UnboundedSender<InternMessage>, mut tx_mt : UnboundedSender<InternMessage>) {
    while let Some(m) = receiver.next().await {
        println!("Receiving message : {m:?}");
        let received_msg : ExternMessage = match serde_json::from_str(m.as_ref().unwrap().to_str().unwrap()) {
            Ok(v) => v,
            Err(e) => panic!("Unable to deserialize: {e}")
        };
        match received_msg.preflight.as_str() {
        "GEN_PLAYERS" | "DRAFT_OK" => {
            let intern_msg : InternMessage = received_msg.into();
            intern_msg.send_message(&mut tx_mt).await;
        }
        _ => panic!("Unrecognized command")
        } 
    }
}




pub fn setup_python_ws(shared_data : Arc<SharedData>) ->  impl Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {
    let borrow_shared = shared_data.clone();
    let data_filter = warp::any().map(move || shared_data.clone());
    let filter = warp::path!("python-ws")
        .and(warp::ws())
        .and(data_filter)
        .map(|ws : warp::ws::Ws, d| {
            ws.on_upgrade(move |ws| handle_python_websocket(ws, d))
        });
    filter
}

