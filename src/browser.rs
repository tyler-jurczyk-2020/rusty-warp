use std::sync::{Mutex, Arc};

use warp::{Filter, filters::{ws::{Message, Ws, WebSocket}, body}, reply::Reply, reject::Rejection};
use futures::{StreamExt, FutureExt, SinkExt, TryFutureExt, Future, future::Map, stream::{SplitSink, SplitStream}};
use warp::hyper::body::Bytes;
use tokio::{sync::{mpsc::{self, UnboundedSender, UnboundedReceiver}, watch::{self, Sender, Receiver}}, stream};
use serde::{Serialize, Deserialize};

use crate::{data::{messaging::{InternMessage}}, data::{browser_comms::{BrowserData}, SharedData}, shared::outgoing_thread};


async fn handle_browser_websocket(ws : warp::ws::WebSocket, shared_data : Arc<SharedData>) {
    let (sender, receiver) = ws.split();

    // Channel to recieve browser messages and pass them along
    println!("Setting up browser comms");
    let browser_data = shared_data.browser.clone();
    let mut c = shared_data.comms.lock().unwrap();
    let tx = c.send_to_brow.clone().unwrap();
    let python_sender = c.send_to_py.clone().unwrap();
    let rx = c.recv_from_py.take().unwrap();
    println!("{rx:?}");

    // Channel to pass messages along to the browser
    tokio::spawn(async move {
        outgoing_thread(sender, rx).await;
    }); 
    tokio::spawn(async move {
        incoming_browser_thread(receiver, tx, python_sender, browser_data).await;
    });
}

async fn incoming_browser_thread(mut receiver : SplitStream<WebSocket>, mut tx_out_b : UnboundedSender<InternMessage>, tx_out_py : UnboundedSender<InternMessage>, browser_data : Arc<Mutex<BrowserData>>) {
    while let Some(m) = receiver.next().await {
        println!("Received browser message: {m:?}");
        if let Ok(msg) = m {
            // NEED TO HANDLE CASE WHEN msg IS AN ERROR!!
            match msg.to_str().unwrap() {
                "GET_PAGE" => {
                    println!("Recieved page code");
                    //let message_to_send;
                    {
                        //let d  = data.lock().unwrap();
                        //match d.state {
                        //    Draft => PageType::DraftIP(DraftResp) 
                        //}
                        //message_to_send = InternMessage::new(Some("GET_PAGE".to_string()), Some(serde_json::to_string(&response).unwrap()));
                    }
                    //message_to_send.send_message(&mut tx_out_b).await;
                }
                _ => panic!("Unrecognized request code")
            } 
        } 
    }
    println!("We escaped while loop!");
} 

pub fn setup_browser_ws(shared_data : Arc<SharedData>) -> impl Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {
    let data_filter = warp::any().map(move || shared_data.clone());
    let filter = warp::path!("browser-ws")
        .and(warp::ws())
        .and(data_filter)
        .map(|ws : warp::ws::Ws, d| {
            ws.on_upgrade(move |ws| handle_browser_websocket(ws, d))
        });
    filter
}
