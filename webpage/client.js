 window.onload = () => {
                const socket = new WebSocket("ws://127.0.0.1:7878/ws")
                socket.onopen = () =>  { 
                    console.log("Socket Opened")
                }
                socket.onmessage = (msg) => {
                    let data = JSON.parse(msg.data)
                    let player_stats = document.getElementById("p1")
                    player_stats.textContent = data.play[0]
                } 
                socket.onerror = (err) => console.error(err)
                socket.onclose = () => console.log("Socket Closed")
}
