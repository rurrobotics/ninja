# Motion profiles for stepper acceleration.
# Returns a list of inter-step delay values in milliseconds.

from config import STEPPER_START_FREQUENCY, STEPPER_RAMP_STEPS


def trapezoid_delays(steps, target_freq, start_freq=None, ramp_steps=None):
    """
    Compute per-step delay schedule (ms) for a trapezoidal motion profile.

    steps       : total number of steps (positive)
    target_freq : cruise frequency in Hz
    start_freq  : starting frequency in Hz (default: STEPPER_START_FREQUENCY)
    ramp_steps  : max ramp length in steps (default: STEPPER_RAMP_STEPS)

    Returns a list of float delay values (ms), one per step.
    """
    if start_freq is None:
        start_freq = STEPPER_START_FREQUENCY
    if ramp_steps is None:
        ramp_steps = STEPPER_RAMP_STEPS

    ramp = min(steps // 2, ramp_steps)
    cruise = steps - 2 * ramp

    delays = []

    # Acceleration ramp
    for i in range(ramp):
        if ramp > 1:
            freq = start_freq + (target_freq - start_freq) * i / (ramp - 1)
        else:
            freq = target_freq
        delays.append(1000.0 / freq)

    # Cruise
    cruise_delay = 1000.0 / target_freq
    for _ in range(cruise):
        delays.append(cruise_delay)

    # Deceleration ramp (mirror of acceleration)
    for i in range(ramp - 1, -1, -1):
        if ramp > 1:
            freq = start_freq + (target_freq - start_freq) * i / (ramp - 1)
        else:
            freq = target_freq
        delays.append(1000.0 / freq)

    return delays
