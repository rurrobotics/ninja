#![no_std]
#![no_main]

use core::ffi::CStr;

use embassy_executor::Spawner;
use embassy_rp::{peripherals::USB, usb};
use embassy_time::Timer;
use panic_halt as _;

const NAME: &CStr = unsafe {
    CStr::from_bytes_with_nul_unchecked(concat!(env!("CARGO_PKG_NAME"), "\0").as_bytes())
};

const DESCRIPTION: &CStr = unsafe {
    CStr::from_bytes_with_nul_unchecked(concat!(env!("CARGO_PKG_DESCRIPTION"), "\0").as_bytes())
};

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(NAME),
    embassy_rp::binary_info::rp_program_description!(DESCRIPTION),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

embassy_rp::bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    spawner.must_spawn(logger_task(p.USB));

    let mut i: u8 = 0;
    loop {
        i = i.wrapping_add(1);
        log::info!("USB says: {}", i);

        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn logger_task(usb: embassy_rp::Peri<'static, embassy_rp::peripherals::USB>) {
    let driver = embassy_rp::usb::Driver::new(usb, Irqs);

    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}
