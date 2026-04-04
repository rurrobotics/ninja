use embassy_rp::{
    Peri,
    gpio::{Input, Level, Output, Pin, Pull},
};
use embassy_time::{Instant, Timer};

pub struct Ultrasonic<'d> {
    trig: Output<'d>,
    echo: Input<'d>,
}

impl<'d> Ultrasonic<'d> {
    pub fn new(trig: Peri<'d, impl Pin>, echo: Peri<'d, impl Pin>) -> Self {
        Self {
            trig: Output::new(trig, Level::Low),
            echo: Input::new(echo, Pull::None),
        }
    }

    pub async fn get_distance(&mut self) -> u32 {
        self.trig.set_high();
        Timer::after_micros(10).await;
        self.trig.set_low();
        while self.echo.is_low() {}

        let start = Instant::now();
        while self.echo.is_high() {}
        let us = start.elapsed().as_micros();

        us as u32 * 34 / 200
    }
}
