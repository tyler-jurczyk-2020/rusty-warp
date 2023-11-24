import time
#import numpy as np

from websockets.sync.client import connect

url = "ws://127.0.0.1:7878/python-ws"

with connect("ws://127.0.0.1:7878/python-ws") as websocket:
        websocket.send("Hello world!")
        message = websocket.recv()
        print(f"Received: {message}")
