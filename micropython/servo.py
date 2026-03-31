# Generic PWM servo controller using machine.PWM.

from machine import Pin, PWM
from config import (
    SERVO_MIN_PW_US, SERVO_MAX_PW_US,
    SERVO_REFRESH_US, SERVO_MAX_DEGREES,
)


class Servo:
    """
    PWM servo motor.

    pwm_pin     : GPIO pin number
    min_pw_us   : pulse width in µs for 0°
    max_pw_us   : pulse width in µs for max_degrees°
    refresh_us  : PWM period in µs (1/freq)
    max_degrees : full range of motion
    """

    def __init__(self, pwm_pin,
                 min_pw_us=SERVO_MIN_PW_US,
                 max_pw_us=SERVO_MAX_PW_US,
                 refresh_us=SERVO_REFRESH_US,
                 max_degrees=SERVO_MAX_DEGREES):
        self.pwm = PWM(Pin(pwm_pin))
        self.pwm.freq(int(1_000_000 / refresh_us))
        self.min_pw_us = min_pw_us
        self.max_pw_us = max_pw_us
        self.refresh_us = refresh_us
        self.max_degrees = max_degrees
        self._stopped = True

    def _duty_for_pulse(self, pulse_us):
        """Convert a pulse width in µs to a 16-bit duty cycle value."""
        return int(pulse_us * 65535 / self.refresh_us)

    def rotate(self, degrees):
        """Set servo position to the given angle (0 to max_degrees)."""
        degrees = max(0, min(degrees, self.max_degrees))
        pw = self.min_pw_us + (degrees / self.max_degrees) * (self.max_pw_us - self.min_pw_us)
        self.pwm.duty_u16(self._duty_for_pulse(pw))
        self._stopped = False

    def stop(self):
        """Disable PWM output (servo holds last position or goes slack)."""
        self.pwm.duty_u16(0)
        self._stopped = True

    def start(self):
        """Re-enable PWM output without changing the angle."""
        if self._stopped:
            self.rotate(0)
