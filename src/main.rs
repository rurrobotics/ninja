#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_time::Timer;
use gpio::{Level, Output};
use {panic_halt as _};

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Ninja SIMA"),
    embassy_rp::binary_info::rp_program_description!(c"Marko Samardzija"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);

    loop {
        led.set_high();
        Timer::after_millis(250).await;

        led.set_low();
        Timer::after_millis(250).await;
    }
}
