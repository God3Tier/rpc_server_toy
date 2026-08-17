import os
import socket

SERVER_IP = os.environ["server15440"]
SERVER_PORT = int(os.environ["serverport15440"])


def send_request(message: str) -> bytes:
    with socket.create_connection((SERVER_IP, SERVER_PORT)) as sock:
        sock.sendall(message.encode())
        return sock.recv(4096)


if __name__ == "__main__":
    # TODO: replace with whatever requests/assertions your mock harness
    # actually needs -- this just proves the network path end to end.
    reply = send_request("OPEN /tmp/testfile.txt 0")
    print("server replied:", reply)