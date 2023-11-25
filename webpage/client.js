 window.onload = () => {
                const socket = new WebSocket("ws://127.0.0.1:7878/browser-ws")
                socket.onopen = () =>  { 
                    console.log("Socket Opened")
                }
                socket.onmessage = (msg) => {
                    let data = JSON.parse(msg.data)
                    console.log(data)
                    document.getElementById("p1").textContent = data[0]
                    document.getElementById("p2").textContent = data[1]
                } 
                socket.onerror = (err) => console.error(err)
                socket.onclose = () => console.log("Socket Closed")
}
