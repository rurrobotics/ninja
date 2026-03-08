use embassy_rp::{
    bind_interrupts,
    peripherals::{PIO0, USB},
    pio, usb,
};

bind_interrupts!(pub struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
});
