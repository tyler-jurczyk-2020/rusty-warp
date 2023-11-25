import numpy as np
from websockets.sync.client import connect
import json

def generate_game(ws):
    print("generating data")
    recv = ws.recv()
    obj = json.loads(recv)
    print(obj)
    batch = np.empty((obj[0],obj[1]))
    for i in range(obj[0]):
        batch[i] = np.random.normal(obj[2][i]["mean"], obj[2][i]["std_dev"], obj[1])
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
