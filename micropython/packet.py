# Packet encoding/decoding using postcard-compatible varint format.
# Matches the Rust RequestPacket / ResponsePacket used by controller.py.
#
# Command indices:
#   0 = Game
#   1 = GripperOpen
#   2 = GripperClose
#   3 = ExtensionPush
#   4 = ExtensionPull
#   5 = Drive(i32)
#   6 = Turn(i32)
#   7 = TestExtension
#   8 = TestRotation
#   9 = TestSquare(u32)
#  10 = TestLine(u32)

GAME          = 0
GRIPPER_OPEN  = 1
GRIPPER_CLOSE = 2
EXT_PUSH      = 3
EXT_PULL      = 4
DRIVE         = 5
TURN          = 6
TEST_EXT      = 7
TEST_ROT      = 8
TEST_SQUARE   = 9
TEST_LINE     = 10

# Commands that carry an i32 argument (ZigZag varint)
_SIGNED_ARG = {DRIVE, TURN}
# Commands that carry a u32 argument (unsigned varint)
_UNSIGNED_ARG = {TEST_SQUARE, TEST_LINE}


def _decode_varint_u32(data, offset):
    """Decode an unsigned LEB128 varint from data[offset:]. Returns (value, new_offset)."""
    value = 0
    shift = 0
    while True:
        b = data[offset]
        offset += 1
        value |= (b & 0x7F) << shift
        if not (b & 0x80):
            break
        shift += 7
    return value, offset


def _decode_varint_i32(data, offset):
    """Decode a ZigZag-encoded signed varint. Returns (value, new_offset)."""
    zigzag, offset = _decode_varint_u32(data, offset)
    value = (zigzag >> 1) ^ -(zigzag & 1)
    return value, offset


def decode_request(data):
    """
    Decode a RequestPacket from bytes.
    Returns (command, arg) where arg is an int or None.
    Raises ValueError on unknown command.
    """
    if not data:
        raise ValueError("empty packet")
    cmd = data[0]
    offset = 1
    if cmd in _SIGNED_ARG:
        arg, _ = _decode_varint_i32(data, offset)
        return cmd, arg
    if cmd in _UNSIGNED_ARG:
        arg, _ = _decode_varint_u32(data, offset)
        return cmd, arg
    return cmd, None


def encode_response(status=True):
    """Encode a ResponsePacket. postcard serializes bool as 0x01/0x00."""
    return bytes([0x01 if status else 0x00])
