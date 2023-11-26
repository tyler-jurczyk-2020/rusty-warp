use std::sync::{Mutex, Arc};

use warp::{Filter, filters::{ws::{Message, Ws, WebSocket}, body}, reply::Reply, reject::Rejection};
use futures::{StreamExt, FutureExt, SinkExt, TryFutureExt, Future, future::Map, stream::{SplitSink, SplitStream}};
use warp::hyper::body::Bytes;
use tokio::{sync::{mpsc::{self, UnboundedSender, UnboundedReceiver}, watch::{self, Sender, Receiver}}, stream};
use serde::{Serialize, Deserialize};

use crate::data::{Data, InternMessage, GlobalComms, ExternMessage};


async fn handle_browser_websocket(ws : warp::ws::WebSocket, data : Arc<Mutex<Data>>, comms : Arc<Mutex<GlobalComms>>) {
    let (mut sender, mut receiver) = ws.split();

    // Get comms sender to python side
    //let mut c = comms.lock().unwrap();
    //let mut python_sender = c.send_to_py.clone().unwrap();

    // Channel to recieve browser messages and pass them along 
    let (tx, mut rx) = mpsc::unbounded_channel::<InternMessage>();
    tokio::spawn(async move {
        incoming_browser_thread(receiver, tx, data.clone()).await;
    });
    // Channel to pass messages along to the browser
    tokio::spawn(async move {
        outgoing_browser_thread(sender, rx).await;
    }); 





        
    // Handle data received from python
    //tokio::spawn(async move {
    //    loop {
    //       if let Ok(v) = python_reciever.changed().await {
    //            // We have some new data to process
    //            let val = python_reciever.borrow_and_update().clone();
    //            let data_to_send : (f64, f64, usize);
    //            {
    //                let d = data.lock().unwrap();
    //                data_to_send = (d.display1, d.display2, val);
    //            }
    //            sender.send(Message::text(serde_json::to_string(&data_to_send).unwrap())).await;
    //        }
    //    }
    //});
    // Handle incoming browser messages
    //let mut python_sender = c.send_to_py.clone().unwrap();
    //tokio::spawn(async move {
    //    loop {
    //        if let Some(r) = receiver.next().await {
    //            println!("Caught!: {:?}", r);
    //            python_sender.send(());
    //        }
    //    }
    //});
}

async fn incoming_browser_thread(mut receiver : SplitStream<WebSocket>, mut tx : UnboundedSender<InternMessage>, data : Arc<Mutex<Data>>) {
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
                        message_to_send.send_message(&mut tx).await;
                    }
                    _ => panic!("Unrecognized request code")
                } 
            } 
        }
    } 
}

async fn outgoing_browser_thread(mut sender : SplitSink<WebSocket, Message>, mut rx : UnboundedReceiver<InternMessage>) {
    loop {
        if let Some(m) = rx.recv().await {
            let incoming_msg : Vec<String> = serde_json::from_str(&m.msg).unwrap();
            println!("{:?}", incoming_msg);
            let outgoing_msg = ExternMessage::new("GET_PAGE".to_string(), incoming_msg);
            outgoing_msg.send_message(&mut sender).await;
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
