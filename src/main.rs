#![no_std]
#![no_main]

pub mod config;
pub mod interrupts;
pub mod packet;
pub mod tasks;

use cyw43::JoinOptions;
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use embassy_executor::Spawner;
use embassy_net::{Config, StackResources};
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::pio::Pio;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use panic_halt as _;
use static_cell::StaticCell;

use crate::config::CYW43_POWER_MANAGEMENT_MODE;
use crate::interrupts::Irqs;
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

    let fw = include_bytes!("../firmware/43439A0.bin");
    let clm = include_bytes!("../firmware/43439A0_clm.bin");

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        // SPI communication won't work if the speed is too high, so we use a divider larger than `DEFAULT_CLOCK_DIVIDER`.
        // See: https://github.com/embassy-rs/embassy/issues/3960.
        RM2_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    spawner.must_spawn(tasks::cyw43(runner));

    control.init(clm).await;
    control
        .set_power_management(CYW43_POWER_MANAGEMENT_MODE)
        .await;

    let config = Config::dhcpv4(Default::default());

    let seed = RoscRng.next_u64();

    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(
        net_device,
        config,
        RESOURCES.init(StackResources::new()),
        seed,
    );
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

    spawner.must_spawn(tasks::actuator(
        p.PWM_SLICE5,
        p.PWM_SLICE7,
        p.PIN_10,
        p.PIN_15,
    ));
    spawner.must_spawn(tasks::receiver(stack));

    core::future::pending::<()>().await;
}
