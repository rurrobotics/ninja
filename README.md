# Ninja

> Note: Hybrid SIMA firmware is available in the [hybrid](https://github.com/rurrobotics/ninja/tree/hybrid) branch. 

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
Stepper EN: |    2W   | Stepper 3:
STP1 - GP10 |         |      - 7V4
STP2 - GP12 |         |  GND - GND
STP3 - GP13 |         | GP20 - 3V3
            |         | GP18 - DIR
Servo 2:    |         | GP17 - STEP
PWM  - GP15 |         |
7V4  -      |         | Battery:
GND  - GND  |         | 7V4 - 7V4
            |         | GND - GND
            |---------|
```

Always `ON` pins: `GP3`, `GP20`.  
Ultrasonic sensor is has `ECHO` and `TRIG` tied to `GP19` and `GP21`.

## Controller
- [Download latest build](https://nightly.link/rurrobotics/ninja/workflows/build/master?preview)
- [Direct link for linux](https://nightly.link/rurrobotics/ninja/workflows/build/master/controller-linux.zip)
- [Direct link for rpi5](https://nightly.link/rurrobotics/ninja/workflows/build/master/controller-rpi5.zip)
- [Direct link for windows](https://nightly.link/rurrobotics/ninja/workflows/build/master/controller-windows.zip)

### Usage
```
Encode robot control packets via postcard/serde

Usage: controller --address <ADDRESS> <COMMAND>

Commands:
  game            
  action          
  custom          
  test-extension  
  test-rotation   
  test-square     
  test-line       
  help            Print this message or the help of the given subcommand(s)

Options:
  -a, --address <ADDRESS>  
  -h, --help               Print help
  -V, --version            Print version
```

## TODO
- ~~Blocking motor calls (irq?)~~
- ~~Accelerate/Decelerate~~
- ~~Parallel motor calls~~
- ~~Homing stutters~~
