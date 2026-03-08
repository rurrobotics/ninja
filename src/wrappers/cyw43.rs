use crate::{config::CYW43_POWER_MANAGEMENT_MODE, interrupts::Irqs};

use cyw43::{Control, NetDriver};
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use embassy_rp::{
    Peri,
    gpio::{Level, Output},
    peripherals::{DMA_CH0, PIN_23, PIN_24, PIN_25, PIN_29, PIO0},
    pio::Pio,
};
use static_cell::StaticCell;

pub async fn wrapper(
    pin23: Peri<'static, PIN_23>,
    pin24: Peri<'static, PIN_24>,
    pin25: Peri<'static, PIN_25>,
    pin29: Peri<'static, PIN_29>,
    dmach0: Peri<'static, DMA_CH0>,
    pio0: Peri<'static, PIO0>,
) -> (
    NetDriver<'static>,
    Control<'static>,
    cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
    &'static [u8],
) {
    let fw = include_bytes!("../../firmware/43439A0.bin");
    let clm = include_bytes!("../../firmware/43439A0_clm.bin");

    let pwr = Output::new(pin23, Level::Low);
    let cs = Output::new(pin25, Level::High);
    let mut pio = Pio::new(pio0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        // SPI communication won't work if the speed is too high, so we use a divider larger than `DEFAULT_CLOCK_DIVIDER`.
        // See: https://github.com/embassy-rs/embassy/issues/3960.
        RM2_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        pin24,
        pin29,
        dmach0,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;

    control.init(clm).await;
    control
        .set_power_management(CYW43_POWER_MANAGEMENT_MODE)
        .await;

    (net_device, control, runner, clm)
}
