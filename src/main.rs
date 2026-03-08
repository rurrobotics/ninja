#![no_std]
#![no_main]

pub mod config;
pub mod interrupts;
pub mod packet;
pub mod tasks;
pub mod wrappers;

use cyw43::JoinOptions;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use panic_halt as _;

use crate::packet::RequestPacket;

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(config::NAME),
    embassy_rp::binary_info::rp_program_description!(config::DESCRIPTION),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

static COMMAND_CHANNEL: Channel<CriticalSectionRawMutex, RequestPacket, 1> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    spawner.must_spawn(tasks::logger(p.USB));

    // Stepper driver
    let _gp12 = Output::new(p.PIN_12, Level::High);
    let _gp3 = Output::new(p.PIN_3, Level::High);
    let _gp20 = Output::new(p.PIN_20, Level::High);

    let (net_device, mut control, runner, clm) =
        wrappers::cyw43(p.PIN_23, p.PIN_24, p.PIN_25, p.PIN_29, p.DMA_CH0, p.PIO0).await;
    spawner.must_spawn(tasks::cyw43(runner));

    let (stack, runner) = wrappers::wifi(net_device).await;
    spawner.must_spawn(tasks::net(runner));

    while let Err(err) = control
        .join(
            config::WIFI_NETWORK,
            JoinOptions::new(config::WIFI_PASSWORD.as_bytes()),
        )
        .await
    {
        log::info!("join failed with status={:?}", err);
    }

    log::info!("waiting for link...");
    stack.wait_link_up().await;

    log::info!("waiting for DHCP...");
    stack.wait_config_up().await;

    log::info!("{:?}", stack.config_v4());

    spawner.must_spawn(tasks::receiver(control, stack));
    spawner.must_spawn(tasks::actuator(
        p.PWM_SLICE5,
        p.PWM_SLICE7,
        p.PIN_10,
        p.PIN_15,
    ));
}
