# Hybrid

> Note: Ninja SIMA firmware is available in the [master](https://github.com/rurrobotics/ninja/tree/master) branch. 

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
