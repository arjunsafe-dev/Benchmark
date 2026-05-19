import socket
import struct
import threading
import time

TARGETS = [
    ("Rust", "127.0.0.1", 8080),
    ("Go", "127.0.0.1", 8081),
]

PACKET = struct.pack('<If', 42, 123.456)

def flood(name, host, port, duration=120):
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect((host, port))
    start = time.time()
    count = 0
    while time.time() - start < duration:
        sock.sendall(PACKET)
        count += 1
    sock.close()

if __name__ == "__main__":
    print("Load testing. Check Grafana.")
    threads = []
    for name, host, port in TARGETS:
        t = threading.Thread(target=flood, args=(name, host, port))
        t.start()
        threads.append(t)
    for t in threads:
        t.join()
    

