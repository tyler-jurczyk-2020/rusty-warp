import { useEffect, useState } from 'react'
import './App.css'

function App() {
    const [portrait, setPortrait] = useState('players/no_portrait.png');
    setup_websocket(setPortrait);
    return (
    <div> 
        <img src={portrait}/>
    </div>
    )
}

function setup_websocket(setPortrait) {
    let ws = new WebSocket("ws://127.0.0.1:7878/browser-ws");

    ws.onopen = function(event) {
        console.log("Connected to server") 
        ws.send("GET_PAGE"); 
    }
    ws.onmessage = function(event) {
        let msg = JSON.parse(event.data); 
        console.log(msg);
        switch(msg.preflight) {
        case "GET_PAGE":
            let page_data = JSON.parse(msg.contents);
            console.log(page_data[0])
            setPortrait("players/" + page_data[0]);
        }
    }
}

export default App
