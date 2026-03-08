use embassy_rp::Peri;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_usb_logger::run;

use crate::interrupts::Irqs;

#[embassy_executor::task]
pub async fn task(usb: Peri<'static, USB>) {
    let driver = Driver::new(usb, Irqs);

    run!(1024, log::LevelFilter::Info, driver);
}
