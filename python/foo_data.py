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
        print(img)
        photos = json.load(img)
    chosenNames = np.random.choice(names["names"], size=20, replace=False)
    randomPhotos = np.random.choice(photos["photos"], size=20) # Should have replace=False
    means = np.random.normal(0.1, 0.1, 20)
    std_devs = np.random.normal(0.3, 0.2, 20)
    print(means)
    print(std_devs)
    players = []
    for i in range(20):
        players.append({
                "name" : chosenNames[i],
                "mean" : means[i],
                "std_dev" : std_devs[i],
                "photo" : randomPhotos[i]
            })
    send_back = {"preflight": "GEN_PLAYERS", "contents": json.dumps(players)}
    ws.send(json.dumps(send_back))
url = "ws://127.0.0.1:7878/python-ws"
with connect("ws://127.0.0.1:7878/python-ws") as websocket:
    incoming = websocket.recv()
    msg = json.loads(incoming)
    print(msg['preflight'])
    match msg['preflight']:
        case "A3A3":
           generate_game(websocket) 
        case "GEN_PLAYERS":
            generate_players(websocket)
