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

def generate_players(ws):
    print("generating players")
    with open("../saved/girl_names_2022.json", "r") as file:
        names = json.load(file)
    with open("../saved/photos.json", "r") as img:
        photos = json.load(img)
    chosenNames = np.random.choice(names["names"], size=20, replace=False)
    randomPhotos = np.random.choice(photos["photos"], size=20) # Should have replace=False
    means = np.random.normal(0.1, 0.1, 20)
    std_devs = np.random.normal(0.3, 0.2, 20)
    std_devs = np.abs(std_devs)
    players = []
    for i in range(20):
        draft_rating = np.exp(means[i])*np.exp(-1*std_devs[i])
        players.append({
                "name" : chosenNames[i],
                "mean" : means[i],
                "std_dev" : std_devs[i],
                "photo" : randomPhotos[i],
                "draftability" : draft_rating
            })
    send_back = {"preflight": "GEN_PLAYERS", "contents": json.dumps(players)}
    ws.send(json.dumps(send_back))

def draft(ws, msg):
    print(msg)
    key_set = list(json.loads(msg["contents"]).keys())
    val_set = list(json.loads(msg["contents"]).values())
    resp = {"preflight": "DRAFT_OK", "contents": "N/A"}
    ws.send(json.dumps(resp))
    total = np.sum(np.array(val_set))
    while(True):
        print("Waiting to pick")
        incoming = ws.recv()
        next_draft = json.loads(incoming)
        if(next_draft["preflight"] == "DRAFT_OVER"):
            print("Exiting draft")
            return
        # If draft not over, pick out the next player
        print(np.array(val_set))
        pick = np.random.choice(key_set, size=1, replace=False, p=np.divide(np.array(val_set),total))
        idx = key_set.index(pick)
        key_set.remove(pick)
        dec = val_set.pop(idx)
        total -= dec
        send_back = {"preflight": "DRAFT_OK", "contents": json.dumps(list(pick).pop())}
        ws.send(json.dumps(send_back))
        
# Main function
url = "ws://127.0.0.1:7878/python-ws"
with connect("ws://127.0.0.1:7878/python-ws") as websocket:
    while(True):
        incoming = websocket.recv()
        msg = json.loads(incoming)
        print(msg['preflight'])
        match msg['preflight']:
            case "A3A3":
               generate_game(websocket) 
            case "GEN_PLAYERS":
                generate_players(websocket)
            case "DRAFT":
                draft(websocket, msg)
