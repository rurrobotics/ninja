# Software-driven async stepper motor.
# Generates step pulses in an asyncio coroutine so other tasks can run
# between steps. Uses utime.sleep_us for the short pulse itself (blocking
# for ~10 µs is negligible) and asyncio.sleep_ms for inter-step delays.

import asyncio
import utime
from machine import Pin
from profiles import trapezoid_delays
from config import STEPPER_DEFAULT_FREQUENCY


class Stepper:
    """
    Single stepper motor controlled via DIR + STEP GPIO pins.

    step_pin : GPIO number for the STEP signal
    dir_pin  : GPIO number for the DIR signal
    freq     : cruise stepping frequency in Hz (default 400)
    """

    PULSE_US = 10  # step pulse high time in microseconds

    def __init__(self, step_pin, dir_pin, freq=STEPPER_DEFAULT_FREQUENCY):
        self.stp = Pin(step_pin, Pin.OUT, value=0)
        self.dir = Pin(dir_pin, Pin.OUT, value=0)
        self.freq = freq

    def set_frequency(self, freq):
        self.freq = freq

    async def step(self, steps):
        """Step |steps| times. Positive = forward (DIR high), negative = backward."""
        if steps == 0:
            return
        self.dir.value(1 if steps > 0 else 0)
        n = abs(steps)
        delays = trapezoid_delays(n, self.freq)
        for d_ms in delays:
            self.stp.high()
            utime.sleep_us(self.PULSE_US)
            self.stp.low()
            wait = max(1, int(d_ms - self.PULSE_US / 1000))
            await asyncio.sleep_ms(wait)
