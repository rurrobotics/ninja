# Gripper servo: specialises Servo with gripper-specific pulse widths and
# a computed actuation time so open/close wait until motion is complete.

import asyncio
from servo import Servo
from config import (
    PIN_SERVO2_PWM,
    GRIPPER_MIN_PW_US, GRIPPER_MAX_PW_US, GRIPPER_REFRESH_US,
    GRIPPER_MIN_ANGLE, GRIPPER_MAX_ANGLE, GRIPPER_ACTUATE_MS,
)


class Gripper:
    def __init__(self):
        self._servo = Servo(
            PIN_SERVO2_PWM,
            min_pw_us=GRIPPER_MIN_PW_US,
            max_pw_us=GRIPPER_MAX_PW_US,
            refresh_us=GRIPPER_REFRESH_US,
            max_degrees=180,
        )

    async def open(self):
        """Open the gripper and wait for actuation to complete."""
        self._servo.rotate(GRIPPER_MIN_ANGLE)
        await asyncio.sleep_ms(GRIPPER_ACTUATE_MS)

    async def close(self):
        """Close the gripper and wait for actuation to complete."""
        self._servo.rotate(GRIPPER_MAX_ANGLE)
        await asyncio.sleep_ms(GRIPPER_ACTUATE_MS)
