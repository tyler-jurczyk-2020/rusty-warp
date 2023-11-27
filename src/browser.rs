use std::sync::{Mutex, Arc};

use warp::{Filter, filters::{ws::{Message, Ws, WebSocket}, body}, reply::Reply, reject::Rejection};
use futures::{StreamExt, FutureExt, SinkExt, TryFutureExt, Future, future::Map, stream::{SplitSink, SplitStream}};
use warp::hyper::body::Bytes;
use tokio::{sync::{mpsc::{self, UnboundedSender, UnboundedReceiver}, watch::{self, Sender, Receiver}}, stream};
use serde::{Serialize, Deserialize};

use crate::{data::{Data, InternMessage, GlobalComms, ExternMessage}, shared::outgoing_thread};


async fn handle_browser_websocket(ws : warp::ws::WebSocket, data : Arc<Mutex<Data>>, comms : Arc<Mutex<GlobalComms>>) {
    let (sender, receiver) = ws.split();

    // Channel to recieve browser messages and pass them along
    let data_ref = data.clone();
    let mut c = comms.lock().unwrap();
    let tx = c.send_to_brow.clone().unwrap();
    let python_sender = c.send_to_py.clone().unwrap();
    let rx = c.recv_from_py.take().unwrap();
    tokio::spawn(async move {
        incoming_browser_thread(receiver, tx, python_sender, data_ref).await;
    });

    // Channel to pass messages along to the browser
    tokio::spawn(async move {
        outgoing_thread(sender, rx).await;
    }); 

    // Signal that a browser has connected 
    tokio::spawn(async move {
        

    });
}

async fn incoming_browser_thread(mut receiver : SplitStream<WebSocket>, mut tx_out_b : UnboundedSender<InternMessage>, tx_out_py : UnboundedSender<InternMessage>, data : Arc<Mutex<Data>>) {
    loop {
        if let Some(m) = receiver.next().await {
            println!("Incoming message on browser thread!");
            if let Ok(msg) = m {
                match msg.to_str().unwrap() {
                    "GET_PAGE" => {
                        println!("Recieved page code");
                        let message_to_send;
                        {
                            let data_handle = data.lock().unwrap();
                            let mut response : Vec<String> = Vec::new();
                            for players in &data_handle.players {
                                response.push(players.photo.clone()); 
                            }
                            message_to_send = InternMessage::new(serde_json::to_string(&response).unwrap());
                        }
                        message_to_send.send_message(&mut tx_out_b).await;
                    }
                    _ => panic!("Unrecognized request code")
                } 
            } 
        }
    } 
}

pub fn setup_browser_ws(data : Arc<Mutex<Data>>, comms : Arc<Mutex<GlobalComms>>) -> impl Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {
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
