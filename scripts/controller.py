import socket
from time import sleep
from random import randint

# Configuration
HOST = "10.29.229.180"
PORT = 1234


def varint_i32(value):
    zigzag = (value << 1) ^ (value >> 31)
    result = bytearray()
    while zigzag > 0x7F:
        result.append((zigzag & 0x7F) | 0x80)
        zigzag >>= 7
    result.append(zigzag & 0x7F)
    return result


def varint_u64(value):
    result = bytearray()
    while value > 0x7F:
        result.append((value & 0x7F) | 0x80)
        value >>= 7
    result.append(value & 0x7F)
    return result


def create_request_packet(stepper1, stepper2, stepper3, servo1=None):
    data = bytearray()
    data.extend(varint_i32(stepper1))
    data.extend(varint_i32(stepper2))
    data.extend(varint_i32(stepper3))

    data.append(0x01 if servo1 is not None else 0x00)
    if servo1 is not None:
        data.extend(varint_u64(servo1))

    data.append(0x01 if None is not None else 0x00)
    if None is not None:
        data.extend(varint_u64(None))

    return bytes(data)


def main():
    print(f"Connecting to {HOST}:{PORT}...")

    packets = [
        [0],
        # [2],
        # [1],
        # [3],
    ]

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.connect((HOST, PORT))
        print("Connected!")

        while True:
            for p in packets:
                packet = bytearray(p)
                if p[0] in [5, 9, 10]:
                    packet.extend(varint_i32(200))
                if p[0] in [6]:
                    packet.extend(varint_i32(90))
                print(f"Sending: {packet.hex()}")
                sock.sendall(packet)

                response = sock.recv(1024)
                print(f"Received: {response.hex()}")
                print(f"Response length: {len(response)} bytes")

                input()


if __name__ == "__main__":
    main()
