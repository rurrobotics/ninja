import socket
from time import sleep
from random import randint

# Configuration
HOST = "192.168.0.4"
PORT = 1234


def create_request_packet(
    stepper1_dist,
    stepper1_dir,
    stepper2_dist,
    stepper2_dir,
    stepper3_dist,
    stepper3_dir,
    servo1_angle=None,
    servo2_angle=None,
):
    def varint_u32(value):
        result = bytearray()
        while value > 0x7F:
            result.append((value & 0x7F) | 0x80)
            value >>= 7
        result.append(value & 0x7F)
        return result

    data = bytearray()

    data.extend(varint_u32(stepper1_dist))
    data.append(stepper1_dir)

    data.extend(varint_u32(stepper2_dist))
    data.append(stepper2_dir)

    data.extend(varint_u32(stepper3_dist))
    data.append(stepper3_dir)

    if servo1_angle is None:
        data.append(0x00)
    else:
        data.append(0x01)
        data.extend(varint_u32(servo1_angle))

    if servo2_angle is None:
        data.append(0x00)
    else:
        data.append(0x01)
        data.extend(varint_u32(servo2_angle))

    return bytes(data)


def main():
    print(f"Connecting to {HOST}:{PORT}...")

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.connect((HOST, PORT))
        print("Connected!")

        while True:
            packet = create_request_packet(randint(0, 200), randint(0, 1), randint(0, 200), randint(0, 1), randint(0, 200), randint(0, 1))
            print(f"Sending: {packet.hex()}")
            sock.sendall(packet)

            response = sock.recv(1024)
            print(f"Received: {response.hex()}")
            print(f"Response length: {len(response)} bytes")

            sleep(2)


if __name__ == "__main__":
    main()
