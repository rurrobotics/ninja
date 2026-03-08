use core::ffi::CStr;

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
