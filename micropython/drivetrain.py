# Drivetrain: two steppers running in parallel for drive/turn.

import asyncio
import math
from stepper import Stepper
from config import (
    PIN_STP1_DIR, PIN_STP1_STEP,
    PIN_STP2_DIR, PIN_STP2_STEP,
    DRIVETRAIN_FREQUENCY,
    DRIVETRAIN_WHEEL_DIAMETER,
    DRIVETRAIN_WHEEL_DISTANCE,
    DRIVETRAIN_STEPS_PER_REV,
)


class Drivetrain:
    """
    Controls left and right stepper motors.

    Positive drive distance moves the robot forward.
    Positive turn degrees rotates the robot clockwise (right turn).
    """

    def __init__(self):
        self.left = Stepper(PIN_STP1_STEP, PIN_STP1_DIR, DRIVETRAIN_FREQUENCY)
        self.right = Stepper(PIN_STP2_STEP, PIN_STP2_DIR, DRIVETRAIN_FREQUENCY)

    async def step(self, steps_left, steps_right):
        """Run both steppers simultaneously."""
        await asyncio.gather(
            self.left.step(steps_left),
            self.right.step(steps_right),
        )

    async def drive(self, distance_mm):
        """Drive straight. Positive = forward."""
        steps = int(distance_mm * DRIVETRAIN_STEPS_PER_REV
                    / (DRIVETRAIN_WHEEL_DIAMETER * math.pi))
        await self.step(steps, -steps)
        return steps

    async def turn(self, degrees):
        """Rotate in place. Positive = clockwise (right turn)."""
        arc = degrees * math.pi / 360.0 * DRIVETRAIN_WHEEL_DISTANCE
        steps = int(arc * DRIVETRAIN_STEPS_PER_REV
                    / (DRIVETRAIN_WHEEL_DIAMETER * math.pi))
        await self.step(steps, steps)
        return steps
