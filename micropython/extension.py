# Extension arm stepper with PIO-based homing.
#
# Homing uses a PIO program that counts steps while the home sensor (GP8,
# active-low with pull-up) reads HIGH, then stops when it reads LOW.
# The step count is read back from the RX FIFO to compute the travel range.
#
# Push/pull use software stepping (same as Stepper class).

import asyncio
import rp2
import utime
from machine import Pin
from config import (
    PIN_STP3_STEP, PIN_STP3_DIR, PIN_STP3_HOME,
    EXTENSION_HOME_FREQUENCY,
    EXTENSION_FREQUENCY,
    EXTENSION_HOME_OFFSET,
    EXTENSION_PULL_OFFSET,
)
from stepper import Stepper


# PIO homing program:
#   - Steps the motor while the JMP_PIN (home sensor) reads HIGH (not home)
#   - When pin goes LOW (sensor triggered), stops and pushes step count to RX FIFO
#   - X starts at 0 and wraps/decrements so final X = 0xFFFFFFFF - (N-1)
#   - Python recovers N = (0xFFFFFFFF - X + 1)
@rp2.asm_pio(set_init=rp2.PIO.OUT_LOW)
def _homing_prog():
    set(x, 0)
    jmp(pin, "loop")    # if pin already LOW, nothing to do
    jmp("end")
    label("loop")
    set(pins, 1) [31]   # STEP high (32 cycles)
    set(pins, 0) [31]   # STEP low  (32 cycles)
    jmp(x_dec, "decr")  # always taken (post-decrement; wraps 0→0xFFFFFFFF on first step)
    label("decr")
    jmp(pin, "loop")    # keep stepping while pin is HIGH
    label("end")
    in_(x, 32)
    push(block)


class Extension:
    """
    Extension arm controlled by stepper 3.

    home() must be called once at startup before push()/pull().
    """

    def __init__(self):
        self.stepper = Stepper(PIN_STP3_STEP, PIN_STP3_DIR, EXTENSION_FREQUENCY)
        self.home_pin = Pin(PIN_STP3_HOME, Pin.IN, Pin.PULL_UP)
        self.max_steps = 0

    async def home(self):
        """
        Run the homing sequence using PIO, then step back to offset position.
        """
        print("Extension: homing...")

        # PIO state machine frequency: EXTENSION_HOME_FREQUENCY * 66 cycles per step
        # (32 set high + 32 set low + 2 overhead ≈ 66 cycles)
        pio_freq = EXTENSION_HOME_FREQUENCY * 66

        sm = rp2.StateMachine(
            2,                          # SM index 2 (stepper 3 uses SM2 on PIO1 in Rust)
            _homing_prog,
            freq=pio_freq,
            set_base=Pin(PIN_STP3_STEP),
            jmp_pin=Pin(PIN_STP3_HOME),
        )
        sm.active(1)

        # Block until PIO pushes the step count
        raw = sm.get()
        sm.active(0)

        # raw is the X register value (may be returned as signed int in MicroPython)
        if raw < 0:
            raw = raw + 0x100000000
        n_steps = (0xFFFFFFFF - raw + 1)
        self.max_steps = n_steps - EXTENSION_HOME_OFFSET
        print(f"Extension: homed, max_steps={self.max_steps}")

        # Step back to home offset position
        await self.stepper.step(EXTENSION_HOME_OFFSET)

    async def push(self):
        """Extend arm to maximum position."""
        await self.stepper.step(self.max_steps)

    async def pull(self):
        """Retract arm to home position."""
        await self.stepper.step(EXTENSION_PULL_OFFSET - self.max_steps)
