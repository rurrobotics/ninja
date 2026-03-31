# Ninja robot - MicroPython entry point for Raspberry Pi Pico W
#
# Startup sequence:
#   1. Enable stepper driver enable pins
#   2. Connect to WiFi (DHCP)
#   3. Home the extension arm, close gripper
#   4. Start TCP server on port 1234
#   5. Dispatch incoming RequestPackets to actuators

import asyncio
from machine import Pin
from config import PIN_ENABLE_1, PIN_ENABLE_2, PIN_ENABLE_3
from drivetrain import Drivetrain
from gripper import Gripper


async def main():
    for pin_num in (PIN_ENABLE_1, PIN_ENABLE_2, PIN_ENABLE_3):
        Pin(pin_num, Pin.OUT, value=1)

    drivetrain = Drivetrain()
    gripper = Gripper()
    gripper.open()
    
    #gripper.close()
    
    await drivetrain.drive(160.0)
    await drivetrain.turn(-90.0)
    await gripper.open()

    await drivetrain.drive(180.0)
    

    await gripper.close()
    await asyncio.sleep_ms(1000)

    await drivetrain.drive(40.0)


    await drivetrain.drive(-210.0)
    await drivetrain.turn(90.0)
    await drivetrain.drive(250.0)
    await drivetrain.turn(-90.0)
    await drivetrain.drive(140.0)
    await drivetrain.turn(90.0)
    await drivetrain.drive(210.0)
    await drivetrain.drive(-230.0)
    await gripper.open()
    await drivetrain.drive(-20.0)
    await drivetrain.drive(-100.0)
    await drivetrain.turn(90.0)
    await drivetrain.drive(70.0)
    await drivetrain.turn(-90.0)
    await drivetrain.drive(350.0)






    


asyncio.run(main())
