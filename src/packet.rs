use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestPacket {
    pub stepper1: i32,
    pub stepper2: i32,
    pub stepper3: i32,
    pub servo1: Option<u64>,
    pub servo2: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponsePacket {
    pub status: bool,
}
