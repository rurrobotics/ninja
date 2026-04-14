mod trapezoid;
mod power;

use crate::config::STEPPER_WITHACC_STEPS_LIMIT;
use heapless::Vec;

pub use trapezoid::TrapezoidProfile;
pub use power::PowerProfile;

pub type Profile = Vec<u32, STEPPER_WITHACC_STEPS_LIMIT>;

pub trait MotionProfile {
    fn delays(&self, steps: u32) -> Profile;
}
