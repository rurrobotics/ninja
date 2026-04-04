use embassy_rp::{Peri, gpio::Pin};
use embassy_time::Timer;

use crate::{
    config::{PROXIMITY_DEFAULT_ENABLE, PROXIMITY_DEFAULT_THRESHOLD},
    sensors::Ultrasonic,
};

pub struct Proximity<'d> {
    ultrasonic: Ultrasonic<'d>,
    pub threshold: u32,
    pub enable: bool,
}

impl<'d> Proximity<'d> {
    pub fn new(trig: Peri<'d, impl Pin>, echo: Peri<'d, impl Pin>) -> Self {
        Self {
            ultrasonic: Ultrasonic::new(trig, echo),
            threshold: PROXIMITY_DEFAULT_THRESHOLD,
            enable: PROXIMITY_DEFAULT_ENABLE,
        }
    }

    pub async fn is_close(&mut self) -> bool {
        let dist = self.ultrasonic.get_distance().await;
        return dist < self.threshold;
    }

    pub async fn wait_for_proximity(&mut self) {
        if !self.enable {
            core::future::pending::<()>().await;
        }
        while !self.is_close().await {
            Timer::after_millis(60).await;
        }
    }
}
