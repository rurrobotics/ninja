#![no_std]
#![no_main]

pub mod actuators;
pub mod config;
pub mod interrupts;
pub mod packet;
pub mod profiles;
pub mod sensors;
pub mod strategy;
pub mod tasks;

use core::net::Ipv4Addr;
use core::str::FromStr;

use cyw43::JoinOptions;
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use embassy_executor::Spawner;
use embassy_net::{Config, Ipv4Cidr, StackResources, StaticConfigV4};
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::pio::Pio;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use heapless::Vec;
use panic_halt as _;
use static_cell::StaticCell;

use crate::config::{
    CYW43_POWER_MANAGEMENT_MODE, WIFI_STATIC_GATEWAY, WIFI_STATIC_IPV4CIDR, WIFI_USE_STATIC,
};
use crate::interrupts::Irqs;
use crate::packet::{RequestPacket, ResponsePacket};

#[unsafe(link_section = ".boot_loader")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

static COMMAND_CHANNEL: Signal<CriticalSectionRawMutex, RequestPacket> = Signal::new();
static RESPONSE_CHANNEL: Signal<CriticalSectionRawMutex, ResponsePacket> = Signal::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    spawner.must_spawn(tasks::logger(p.USB));

    // Stepper driver
    let _gp13 = Output::new(p.PIN_13, Level::High);
    let _gp21 = Output::new(p.PIN_21, Level::High);
    let _gp8 = Output::new(p.PIN_8, Level::High);

    let fw = include_bytes!("../../binary/43439A0.bin");
    let clm = include_bytes!("../../binary/43439A0_clm.bin");

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio0 = Pio::new(p.PIO0, Irqs);
    let pio1 = Pio::new(p.PIO1, Irqs);
    let spi = PioSpi::new(
        &mut pio0.common,
        pio0.sm0,
        // SPI communication won't work if the speed is too high, so we use a divider larger than `DEFAULT_CLOCK_DIVIDER`.
        // See: https://github.com/embassy-rs/embassy/issues/3960.
        RM2_CLOCK_DIVIDER,
        pio0.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    spawner.must_spawn(tasks::actuator(
        pio0.common,
        pio1.common,
        pio0.sm2,
        (pio1.sm0, pio1.irq0, p.DMA_CH1),
        (pio1.sm1, pio1.irq1, p.DMA_CH2),
        (p.PIN_6, p.PIN_7),
        (p.PIN_18, p.PIN_20),
        (p.PIN_11, p.PIN_12),
        (p.PIN_19, p.PIN_10),
        p.PIN_15,
        p.PIN_27,
    ));

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    spawner.must_spawn(tasks::cyw43(runner));

    control.init(clm).await;
    control
        .set_power_management(CYW43_POWER_MANAGEMENT_MODE)
        .await;

    let config = match WIFI_USE_STATIC {
        true => Config::ipv4_static(StaticConfigV4 {
            address: Ipv4Cidr::from_str(WIFI_STATIC_IPV4CIDR).unwrap(),
            gateway: Some(Ipv4Addr::from_str(WIFI_STATIC_GATEWAY).unwrap()),
            dns_servers: Vec::new(),
        }),
        false => Config::dhcpv4(Default::default()),
    };

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

    spawner.must_spawn(tasks::receiver(stack));
    control.gpio_set(0, true).await;

    core::future::pending::<()>().await;
}
