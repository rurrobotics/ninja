use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestPacket {
    pub stepper1: i64,
    pub stepper2: i64,
    pub stepper3: i64,
    pub servo1: Option<u64>,
    pub servo2: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponsePacket {
    pub status: bool,
}
