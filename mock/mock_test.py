import os
import socket

SERVER_IP = os.environ.get("server15440", "127.0.0.1")
SERVER_PORT = int(os.environ.get("serverport15440", "15440"))


def send_request(message: str, recv_size: int = 4096) -> str:
    """Opens a fresh connection per request -- matches a client that
    doesn't keep a long-lived socket. Switch to a shared connection here
    if your server design expects one socket per client session instead."""
    with socket.create_connection((SERVER_IP, SERVER_PORT), timeout=5) as sock:
        sock.sendall(message.encode())
        data = sock.recv(recv_size)
        return data.decode(errors="replace")


def check(label: str, message: str):
    print(f"--- {label} ---")
    print("sent:   ", repr(message))
    try:
        reply = send_request(message)
        print("recv:   ", repr(reply))
    except Exception as e:
        print("ERROR:  ", e)
    print()


if __name__ == "__main__":
    check("OPEN (existing file, read-only)", "OPEN /tmp/testfile.txt 0")
    check("OPEN (nonexistent file)", "OPEN /tmp/does_not_exist.txt 0")
    check("STAT", "STAT /tmp/testfile.txt")
    check("UNLINK (careful -- this deletes it)", "UNLINK 0 /tmp/scratch_delete_me.txt")

    fd_from_open = 3  # placeholder, replace with the real value printed above
    check("READ", f"READ {fd_from_open} 64")
    check("WRITE", f"WRITE {fd_from_open} hello")
    check("LSEEK", f"LSEEK {fd_from_open} 0 0")
    check("CLOSE", f"CLOSE {fd_from_open}")

    check("GETDIRENTRIES", f"GETDIRENTRIES {fd_from_open} 1024 0")
    check("GETDIRTREE", "GETDIRTREE /tmp")

    check("Unknown opcode (should hit Default)", "BOGUS foo bar")