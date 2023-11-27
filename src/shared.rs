use std::{sync::{Arc, Mutex}, time::Duration};

use futures::StreamExt;
use futures::stream::{SplitSink, SplitStream};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use warp::filters::ws::{WebSocket, Message};

use crate::{data::{InternMessage, ExternMessage, Data, GENERATE}, events::run_match};

// This file is for shared functions between browser and python
pub async fn outgoing_thread(mut sender : SplitSink<WebSocket, Message>, mut rx_in_b : UnboundedReceiver<InternMessage>) {
    while let Some(m) = rx_in_b.recv().await {
        let incoming_msg : Vec<String> = serde_json::from_str(&m.msg).unwrap();
        let outgoing_msg = ExternMessage::new("GET_PAGE".to_string(), incoming_msg);
        outgoing_msg.send_message(&mut sender).await;
    } 
}

pub async fn main_thread(data : Arc<Mutex<Data>>, tx_brow : UnboundedSender<InternMessage>, tx_py : UnboundedSender<InternMessage>) {

        //NEED TO REWRITE MAIN THREAD!!!

        //let data_handle = data.clone();
        //let request_data;
        //{
        //    let data_hold = data_handle.lock().unwrap();
        //    let data_to_ser = (data_hold.player_count, data_hold.batch_size, &data_hold.players);
        //    request_data = ExternMessage::new(GENERATE.to_string(), data_to_ser);
        //}
        //request_data.send_message(&mut sender).await;
        //if let Some(m) = receiver.next().await {
        //    println!("{m:?}");
        //    {
        //    let mut dh = data_handle.lock().unwrap();
        //    dh.batch = serde_json::from_str(m.unwrap().to_str().unwrap()).unwrap();
        //    println!("{:?}", dh.batch);
        //    }
        //    println!("Match starting soon...");
        //    //** Send data to browser to match **//
        //    tokio::time::sleep(Duration::new(5, 0)).await; 
        //    println!("Match starting now!");
            //run_match(&watch_tx, data.clone()).await;
        //}
}
