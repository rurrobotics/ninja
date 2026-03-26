mod scurve;
mod trapezoid;

pub use scurve::SCurveProfile;
pub use trapezoid::TrapezoidProfile;

use crate::config::STEPPER_MAX_ACCELERATION_STEPS;

pub trait MotionProfile {
    fn delays(
        &self,
        steps: u32,
    ) -> (
        heapless::Vec<u32, STEPPER_MAX_ACCELERATION_STEPS>,
        usize,
        heapless::Vec<u32, STEPPER_MAX_ACCELERATION_STEPS>,
    );
}
