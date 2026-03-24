# Ninja

## Wiring

```
Stepper 1:  |---------| Stepper 2:
7V4  -      |         |      - 7V4
GND  - GND  |         |  GND - GND
3V3  - GP3  |         |  3V3 - 3V3
DIR  - GP5  |         | GP28 - DIR
STEP - GP6  |         | GP27 - STEP
            |         |
HOME - GP8  |         |
            |   PICO  |
Servo 1:    |    2W   | Stepper 3:
PWM  - GP10 |         |      - 7V4
3V3  - GP12 |         |  GND - GND
GND  - GND  |         | GP20 - 3V3
            |         | GP18 - DIR
Servo 2:    |         | GP17 - STEP
PWM  - GP15 |         |
7V4  -      |         | Battery:
GND  - GND  |         | 7V4 - 7V4
            |         | GND - GND
            |---------|
```

Always on pins: `GP3`, `GP12`, `GP20`.

## TODO
- ~~Blocking motor calls (irq?)~~
- Accelerate/Decelerate
- Parallel motor calls
- ~~Homing stutters~~