 window.onload = () => {
                const socket = new WebSocket("ws://127.0.0.1:7878/browser-ws")
                socket.onopen = () =>  { 
                    console.log("Socket Opened")
                }
                socket.onmessage = (msg) => {
                    let data = JSON.parse(msg.data)
                    console.log(data[2])
                    let p1 = document.getElementById("p1")
                    let p2 = document.getElementById("p2")
                    p1.textContent = data[0]
                    p2.textContent = data[1]
                    if(data[2] & 1) {
                        p1.style.color = "green"
                    } 
                    else {
                        p1.style.color = "red"
                    }
                    if(data[2] & 2) {
                        p2.style.color = "green" 
                    }
                    else {
                        p2.style.color = "red"
                    }
                } 
                socket.onerror = (err) => console.error(err)
                socket.onclose = () => console.log("Socket Closed")
}
