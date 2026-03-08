use cyw43::Runner;
use cyw43_pio::PioSpi;
use embassy_rp::gpio::Output;
use embassy_rp::peripherals::{DMA_CH0, PIO0};

#[embassy_executor::task]
pub async fn task(
    runner: Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) {
    runner.run().await
}
