use embassy_rp::{Peri, gpio::Pin};
use embassy_time::Timer;

use crate::{
    config::{PROXIMITY_ALPHA, PROXIMITY_DEFAULT_ENABLE, PROXIMITY_DEFAULT_THRESHOLD},
    sensors::Ultrasonic,
};

pub struct Proximity<'d> {
    ultrasonic: Ultrasonic<'d>,
    pub threshold: f64,
    pub enable: bool,
    last_value: Option<f64>,
}

impl<'d> Proximity<'d> {
    pub fn new(trig: Peri<'d, impl Pin>, echo: Peri<'d, impl Pin>) -> Self {
        Self {
            ultrasonic: Ultrasonic::new(trig, echo),
            threshold: PROXIMITY_DEFAULT_THRESHOLD,
            enable: PROXIMITY_DEFAULT_ENABLE,
            last_value: None,
        }
    }

    pub async fn is_close(&mut self) -> bool {
        let dist = self.ultrasonic.get_distance().await;

        self.last_value = Some(match self.last_value {
            Some(lv) => PROXIMITY_ALPHA * dist as f64 + (1.0 - PROXIMITY_ALPHA) * lv,
            None => dist as f64,
        });
        log::info!("{:?}", self.last_value);
        return self.last_value.unwrap() < self.threshold;
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
