#![no_std]
#![no_main]

pub mod conf;
pub mod packet;

use cyw43::{Control, JoinOptions};
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use embassy_executor::Spawner;
use embassy_net::{Config, Stack, StackResources, tcp::TcpSocket};
use embassy_rp::{
    bind_interrupts,
    clocks::RoscRng,
    gpio::{Level, Output},
    peripherals::{DMA_CH0, PIN_10, PIN_15, PIO0, PWM_SLICE5, PWM_SLICE7, USB},
    pio::{self, Pio},
    pwm::{Config as PwmConfig, Pwm},
    usb,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Duration;
use embedded_io_async::Write;
use panic_halt as _;
use static_cell::StaticCell;

use crate::packet::{RequestPacket, ResponsePacket};

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(conf::NAME),
    embassy_rp::binary_info::rp_program_description!(conf::DESCRIPTION),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
});

static COMMAND_CHANNEL: Channel<CriticalSectionRawMutex, RequestPacket, 1> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    spawner.must_spawn(logger_task(p.USB));

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
    spawner.must_spawn(cyw43_task(runner));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::Performance)
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

    spawner.must_spawn(net_task(runner));

    while let Err(err) = control
        .join(
            conf::WIFI_NETWORK,
            JoinOptions::new(conf::WIFI_PASSWORD.as_bytes()),
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

    spawner.must_spawn(receiver_task(control, stack));
}

#[embassy_executor::task]
async fn actuator_task(
    pwm5: embassy_rp::Peri<'static, PWM_SLICE5>,
    gp10: embassy_rp::Peri<'static, PIN_10>,
    gp15: embassy_rp::Peri<'static, PIN_15>,
    pwm7: embassy_rp::Peri<'static, PWM_SLICE7>,
) {
    let mut config1 = PwmConfig::default();
    config1.top = 20000;
    config1.divider = 125.into();
    let servo1 = Pwm::new_output_a(pwm5, gp10, config1);

    let mut config2 = PwmConfig::default();
    config2.top = 20000;
    config2.divider = 125.into();
    let servo2 = Pwm::new_output_b(pwm7, gp15, config2);

    fn angle_to_duty(angle: u32) -> u16 {
        let pulse_us = 1000 + (angle.min(180) * 1000 / 180);
        pulse_us as u16
    }

    loop {
        let cmd = COMMAND_CHANNEL.receive().await;

        log::info!("Actuator received: {:?}", cmd);

        if let Some(angle) = cmd.servo1 {
            let duty = angle_to_duty(angle);
            servo1.set_counter(duty);
        }

        if let Some(angle) = cmd.servo2 {
            let duty = angle_to_duty(angle);
            servo2.set_counter(duty);
        }

        // TODO: Stepper
    }
}

#[embassy_executor::task]
async fn receiver_task(mut control: Control<'static>, stack: Stack<'static>) {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_keep_alive(Some(Duration::from_secs(10)));

        control.gpio_set(0, false).await;
        log::info!("Listening on TCP:1234...");
        if let Err(e) = socket.accept(1234).await {
            log::warn!("accept error: {:?}", e);
            continue;
        }

        log::info!("Received connection from {:?}", socket.remote_endpoint());
        control.gpio_set(0, true).await;

        loop {
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    log::warn!("read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    log::warn!("read error: {:?}", e);
                    break;
                }
            };

            let req: RequestPacket = match postcard::from_bytes(&buf[..n]) {
                Ok(rp) => rp,
                Err(e) => {
                    log::warn!("format is invalid: {:?}", e);
                    break;
                }
            };

            log::info!("{:?}", req);
            COMMAND_CHANNEL.send(req).await;

            let resp = match postcard::to_vec::<_, 4>(&ResponsePacket { status: true }) {
                Ok(rp) => rp,
                Err(e) => {
                    log::warn!("response error: {:?}", e);
                    break;
                }
            };

            match socket.write_all(&resp).await {
                Ok(()) => {}
                Err(e) => {
                    log::warn!("write error: {:?}", e);
                    break;
                }
            };
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) {
    runner.run().await
}

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) {
    runner.run().await
}

#[embassy_executor::task]
async fn logger_task(usb: embassy_rp::Peri<'static, embassy_rp::peripherals::USB>) {
    let driver = embassy_rp::usb::Driver::new(usb, Irqs);

    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}
