use std::{sync::{Arc, Mutex}, time::Duration, collections::HashMap};

use futures::StreamExt;
use futures::stream::{SplitSink, SplitStream};
use tokio::{sync::mpsc::{UnboundedReceiver, UnboundedSender}, fs::File, io::{AsyncReadExt, AsyncWriteExt}};
use warp::filters::ws::{WebSocket, Message};

use crate::{data::{GENERATE, Settings, messaging::{InternMessage, ExternMessage}, gamedata::{Data, Player, Team, State}, SharedData}, events::run_match};

// This file is for shared functions between browser and python
pub async fn outgoing_thread(mut sender : SplitSink<WebSocket, Message>, mut rx_in_b : UnboundedReceiver<InternMessage>) {
    while let Some(m) = rx_in_b.recv().await {
        let outgoing_msg : ExternMessage = m.into();
        outgoing_msg.send_message(&mut sender).await;
    } 
}

pub async fn main_thread(shared_data : Arc<SharedData>, tx_brow : UnboundedSender<InternMessage>, mut tx_py : UnboundedSender<InternMessage>, mut rx : UnboundedReceiver<InternMessage>) {

        //NEED TO REWRITE MAIN THREAD!!!
        let settings = match File::open("./saved/settings.txt").await {
            Ok(mut f) => {
                let mut buf = String::new(); 
                f.read_to_string(&mut buf).await.unwrap();
                let settings : Settings = serde_json::from_str(&buf).unwrap();
                settings
            }
            Err(_) => {
                let mut file = File::create("./saved/settings.txt").await.unwrap();
                let settings = Settings::new();
                file.write_all(serde_json::to_string(&settings).unwrap().as_bytes()).await.unwrap(); 
                settings
            }
        };
        let mut player_map : HashMap<String, Player>;
        if settings.gen_players {
            let msg = InternMessage::new(Some("GEN_PLAYERS".to_string()), None); 
            msg.send_message(&mut tx_py).await;
            let d = rx.recv().await.unwrap();
            let players_pool : Vec<Player> = serde_json::from_str(d.msg.unwrap().as_str()).unwrap();  
            player_map = players_pool.into_iter().map(|p| (p.name.clone(), p)).collect();
            println!("Mapped: {player_map:?}");
            println!("Players created");
            // Always overwrite player data if gen_players option is set
            let mut f = File::create("./saved/gen_players.txt").await.unwrap();
            f.write_all(serde_json::to_string(&player_map).unwrap().as_bytes()).await.unwrap();
            //f.write_all(src) 
        }
        else {
            // Need to load the generated player data
            player_map = HashMap::new();
            println!("Need to load player data...");
        }
        // NEED TO REORG
        {
            let mut d = shared_data.game.lock().unwrap();
            //d.player_pool = player_map;
        }
        if settings.draft {
            {
                let mut d = shared_data.game.lock().unwrap();
                d.state = State::Draft;
            } 
            // May want to separate out team creation
            let t1 = Team::new();
            let t2 = Team::new();
            let mut teams = vec![t1, t2];
            let mut next_up = 0;
            // Start actual draft
            println!("Starting the draft"); 
            let draft_map : HashMap<String, f64> = player_map.clone().into_iter().map(|h| (h.0, h.1.draftability)).collect();
            let start = InternMessage::new(Some("DRAFT".to_string()), Some(serde_json::to_string(&draft_map).unwrap()));
            start.send_message(&mut tx_py).await;
            if let Some(m) = rx.recv().await {
                if m.code.unwrap().eq("DRAFT_OK") {
                    println!("Ready to start drafting");
                    for _ in 0..draft_map.len() {
                        let get_next = InternMessage::new(Some("DRAFT".to_string()), None);  
                        get_next.send_message(&mut tx_py).await;
                        if let Some(m) = rx.recv().await {
                            if let Some(c) = m.msg {
                                let name : String = serde_json::from_str(&c).unwrap();
                                let player = player_map.get(&name).cloned().unwrap();
                                teams[next_up].members.push(player);
                                next_up = (next_up+1)%2;
                            }
                        }
                    }
                    let end_draft = InternMessage::new(Some("DRAFT_OVER".to_string()), None);
                    end_draft.send_message(&mut tx_py).await;
                }
            }
        }

        let data_handle = shared_data.game.clone();
        let request_data;
        {
            let data_hold = data_handle.lock().unwrap();
            let data_to_ser = (data_hold.player_count, data_hold.batch_size, &data_hold.players);
            request_data = InternMessage::new(Some(GENERATE.to_string()), Some(serde_json::to_string(&data_to_ser).unwrap()));
        }
        request_data.send_message(&mut tx_py).await;
        //if let Some(m) = receiver.next().await {
        //    println!("{m:?}");
        //    {
        //    let mut dh = data_handle.lock().unwrap();
        //    dh.batch = serde_json::from_str(m.unwrap().to_str().unwrap()).unwrap();
        //    println!("{:?}", dh.batch);
        //    }
        //    println!("Match starting soon...");
            //** Send data to browser to match **//
        //    tokio::time::sleep(Duration::new(5, 0)).await; 
        //    println!("Match starting now!");
        //    run_match(&watch_tx, data.clone()).await;
        //}
}

