# Configuration constants for Ninja robot (MicroPython / Pico W)
import math

# WiFi credentials - edit these or use secrets.py
try:
    from secrets import WIFI_NETWORK, WIFI_PASSWORD
except ImportError:
    WIFI_NETWORK = "your_ssid"
    WIFI_PASSWORD = "your_password"

# TCP server
RECEIVER_PORT = 1234
RECEIVER_BUFFER_SIZE = 64
RECEIVER_KEEP_ALIVE_SECS = 10

# GPIO pins
PIN_ENABLE_1 = 3
PIN_ENABLE_2 = 12
PIN_ENABLE_3 = 20

PIN_STP1_DIR = 5
PIN_STP1_STEP = 6

PIN_STP2_DIR = 18
PIN_STP2_STEP = 17

PIN_STP3_DIR = 18
PIN_STP3_STEP = 17
PIN_STP3_HOME = 8

PIN_SERVO1_PWM = 10  # arm rotation (disabled)
PIN_SERVO2_PWM = 15  # gripper

# Stepper defaults
STEPPER_DEFAULT_FREQUENCY = 1000   # Hz
STEPPER_START_FREQUENCY = 100      # Hz (ramp start)
STEPPER_RAMP_STEPS = 40            # steps over which to ramp up/down

# Extension arm
EXTENSION_HOME_FREQUENCY = 200     # Hz (slow for homing)
EXTENSION_FREQUENCY = 700          # Hz (normal)
EXTENSION_HOME_OFFSET = 4          # steps back from home sensor
EXTENSION_PULL_OFFSET = 2

# Servo (arm rotation)
SERVO_MIN_PW_US = 1000
SERVO_MAX_PW_US = 2000
SERVO_REFRESH_US = 20000           # 50 Hz
SERVO_MAX_DEGREES = 180

# Gripper servo
GRIPPER_MIN_PW_US = 320
GRIPPER_MAX_PW_US = 1200
GRIPPER_REFRESH_US = 1786          # ~560 Hz
GRIPPER_MIN_ANGLE = 27
GRIPPER_MAX_ANGLE = 135
# Actuation time: ~40ms per 60 degrees + 5ms margin
GRIPPER_ACTUATE_MS = int((40 + 5) * (GRIPPER_MAX_ANGLE - GRIPPER_MIN_ANGLE) / 60)

# Drivetrain geometry
DRIVETRAIN_WHEEL_DIAMETER = 56.0   # mm
DRIVETRAIN_WHEEL_DISTANCE = 159.5  # mm (track width)
DRIVETRAIN_STEPS_PER_REV = 400
DRIVETRAIN_FREQUENCY = 400         # Hz
