from machine import Pin
import utime

dir1 = Pin(5, Pin.OUT)
step1 = Pin(6, Pin.OUT, value=0)

dir2 = Pin(18, Pin.OUT)
step2 = Pin(17, Pin.OUT, value=0)

def move(forward, steps=400):
    dir1.value(1 if forward else 0)
    dir2.value(0 if forward else 1)  # reversed for opposite side
    for i in range(steps):
        step1.high(); step2.high()
        utime.sleep_us(10)
        step1.low(); step2.low()
        utime.sleep_ms(2)

move(True)   # forward
utime.sleep_ms(500)
move(False)  # backward
