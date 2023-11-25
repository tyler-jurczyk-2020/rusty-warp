import numpy as np
from websockets.sync.client import connect
import json

def generate_game(ws):
    print("generating data")
    iter = ws.recv()
    batch = np.empty((2,10))
    for i in range(int(iter)):
        batch[i] = np.random.normal(0, 1, 10)
    print(batch)
    json_form = { 
            "play_action" : batch.tolist()
    }
    ws.send(json.dumps(json_form))

url = "ws://127.0.0.1:7878/python-ws"
with connect("ws://127.0.0.1:7878/python-ws") as websocket:
        preflight = websocket.recv()
        print(preflight)
        match str(preflight):
            case "A3A3":
               generate_game(websocket) 
