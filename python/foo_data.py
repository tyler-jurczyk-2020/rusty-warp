import requests
import time
import numpy as np

url = "http://127.0.0.1:7878/python"

session = requests.Session()

response = session.get(url)
print(response.text)

while(True):
    data = np.random.normal(0, 1, 20);
    print(data)
    json = {"play": data.tolist()}
    session.post(url, json=json)
    time.sleep(5)
