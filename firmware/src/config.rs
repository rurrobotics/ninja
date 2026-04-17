use core::ffi::CStr;

use core::time::Duration as CoreDuration;
use cyw43::PowerManagementMode;
use embassy_time::Duration;

use crate::profiles::PowerProfile;

pub const NAME: &CStr = unsafe {
    CStr::from_bytes_with_nul_unchecked(concat!(env!("CARGO_PKG_NAME"), "\0").as_bytes())
};

pub const DESCRIPTION: &CStr = unsafe {
    CStr::from_bytes_with_nul_unchecked(concat!(env!("CARGO_PKG_DESCRIPTION"), "\0").as_bytes())
};

pub const WIFI_NETWORK: &'static str = env!("WIFI_NETWORK");
pub const WIFI_PASSWORD: &'static str = env!("WIFI_PASSWORD");
pub const WIFI_USE_STATIC: bool = {
    let s = env!("WIFI_USE_STATIC").as_bytes();
    matches!(s, b"true")
};
pub const WIFI_STATIC_IPV4CIDR: &'static str = "192.168.4.10/24";
pub const WIFI_STATIC_GATEWAY: &'static str = "192.168.4.1";

pub const RECEIVER_BUFFER_SIZE: usize = 1024;
pub const RECEIVER_KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(10);

pub const CYW43_POWER_MANAGEMENT_MODE: PowerManagementMode = PowerManagementMode::Performance;

pub const STEPPER_WITHACC_STEPS_LIMIT: usize = 4096;
pub const STEPPER_WITHACC_TIMER_FREQUENCY: u32 = 1_000_000;

pub const PROXIMITY_DEFAULT_ENABLE: bool = true;
pub const PROXIMITY_DEFAULT_THRESHOLD: f64 = 60.0; // in mm
pub const PROXIMITY_ALPHA: f64 = 0.2; // PROXIMITY_ALPHA * value + (1 - PROXIMITY_ALPHA) * last

pub const EXTENSION_HOME_FREQUENCY: u32 = 400;
pub const EXTENSION_HOME_OFFSET: i32 = 120;
pub const EXTENSION_PULL_OFFSET: i32 = 2;
pub const EXTENSION_FREQUENCY: u32 = 700;

pub const SERVO_DEFAULT_MIN_PULSE_WIDTH: CoreDuration = CoreDuration::from_micros(1000);
pub const SERVO_DEFAULT_MAX_PULSE_WIDTH: CoreDuration = CoreDuration::from_micros(2000);
pub const SERVO_DEFAULT_REFRESH_INTERVAL: CoreDuration = CoreDuration::from_micros(20000);
pub const SERVO_MAX_DEGREE_ROTATION: u64 = 180;

pub const GRIPPER_MIN_PULSE_WIDTH: CoreDuration = CoreDuration::from_micros(320);
pub const GRIPPER_MAX_PULSE_WIDTH: CoreDuration = CoreDuration::from_micros(1200);
pub const GRIPPER_REFRESH_INTERVAL: CoreDuration = CoreDuration::from_micros(1786);
pub const GRIPPER_MIN_ANGLE: u64 = 50;
pub const GRIPPER_MAX_ANGLE: u64 = 153;
// 0.04s per 60, 0.005s margin
pub const GRIPPER_ACTUATE_TIME: Duration = Duration::from_nanos(
    Duration::from_millis(40 + 5).as_nanos() * (GRIPPER_MAX_ANGLE - GRIPPER_MIN_ANGLE) / 60,
);

pub const DRIVETRAIN_WHEEL_DIAMETER: f64 = 56.0;
pub const DRIVETRAIN_WHEEL_DISTANCE: f64 = 159.0;
pub const DRIVETRAIN_STEPS_PER_REVOLUTION: u32 = 400;
pub type DrivetrainProfile = PowerProfile;

pub const TRAPEZOID_DEFAULT_MAX_SPEED: f64 = 400.0 / (DRIVETRAIN_WHEEL_DIAMETER / 2.0);
pub const TRAPEZOID_DEFAULT_ACCELERATION: f64 = TRAPEZOID_DEFAULT_MAX_SPEED * 3.0;
pub const TRAPEZOID_DEFAULT_STEPS_PER_REVOLUTION: u32 = DRIVETRAIN_STEPS_PER_REVOLUTION;

pub const POWER_DEFAULT_MAX_SPEED: f64 = TRAPEZOID_DEFAULT_MAX_SPEED;
pub const POWER_DEFAULT_ACCELERATION: f64 = TRAPEZOID_DEFAULT_ACCELERATION;
pub const POWER_DEFAULT_P: f64 = 3.0;
pub const POWER_DEFAULT_STEPS_PER_REVOLUTION: u32 = TRAPEZOID_DEFAULT_STEPS_PER_REVOLUTION;
