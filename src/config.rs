use core::ffi::CStr;

use core::time::Duration as CoreDuration;
use cyw43::PowerManagementMode;
use embassy_time::Duration;

pub const NAME: &CStr = unsafe {
    CStr::from_bytes_with_nul_unchecked(concat!(env!("CARGO_PKG_NAME"), "\0").as_bytes())
};

pub const DESCRIPTION: &CStr = unsafe {
    CStr::from_bytes_with_nul_unchecked(concat!(env!("CARGO_PKG_DESCRIPTION"), "\0").as_bytes())
};

pub const WIFI_NETWORK: &'static str = env!("WIFI_NETWORK");
pub const WIFI_PASSWORD: &'static str = env!("WIFI_PASSWORD");

pub const RECEIVER_BUFFER_SIZE: usize = 64;
pub const RECEIVER_KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(10);

pub const CYW43_POWER_MANAGEMENT_MODE: PowerManagementMode = PowerManagementMode::Performance;

pub const STEPPER_DEFAULT_FREQUENCY: u32 = 1000;
pub const STEPPER_DEFAULT_START_DELAY: u32 = 100;
pub const STEPPER_DEFAULT_ACCELERATION: u32 = 2;
pub const STEPPER_MAX_ACCELERATION_STEPS: usize = 256;


pub const EXTENSION_HOME_FREQUENCY: u32 = 200;
pub const EXTENSION_HOME_OFFSET: i32 = 4;
pub const EXTENSION_PULL_OFFSET: i32 = 2;
pub const EXTENSION_FREQUENCY: u32 = 700;

pub const SERVO_DEFAULT_MIN_PULSE_WIDTH: CoreDuration = CoreDuration::from_micros(1000);
pub const SERVO_DEFAULT_MAX_PULSE_WIDTH: CoreDuration = CoreDuration::from_micros(2000);
pub const SERVO_DEFAULT_REFRESH_INTERVAL: CoreDuration = CoreDuration::from_micros(20000);
pub const SERVO_MAX_DEGREE_ROTATION: u64 = 180;

pub const GRIPPER_MIN_PULSE_WIDTH: CoreDuration = CoreDuration::from_micros(320);
pub const GRIPPER_MAX_PULSE_WIDTH: CoreDuration = CoreDuration::from_micros(1200);
pub const GRIPPER_REFRESH_INTERVAL: CoreDuration = CoreDuration::from_micros(1786);
pub const GRIPPER_MIN_ANGLE: u64 = 27;
pub const GRIPPER_MAX_ANGLE: u64 = 135;
// 0.04s per 60, 0.005s margin
pub const GRIPPER_ACTUATE_TIME: Duration = Duration::from_nanos(
    Duration::from_millis(40 + 5).as_nanos() * (GRIPPER_MAX_ANGLE - GRIPPER_MIN_ANGLE) / 60,
);

pub const DRIVETRAIN_WHEEL_DIAMETER: f64 = 55.0;
pub const DRIVETRAIN_WHEEL_DISTANCE: f64 = 137.0;
pub const DRIVETRAIN_STEPS_PER_REVOLUTION: u32 = 400;
pub const DRIVETRAIN_FREQUENCY: u32 = 200;