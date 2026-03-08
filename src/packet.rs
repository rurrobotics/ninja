use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum StepperDirection {
    FORWARD,
    BACKWARD,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StepperRequest {
    pub distance: u32,
    pub direction: StepperDirection,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestPacket {
    pub stepper1: StepperRequest,
    pub stepper2: StepperRequest,
    pub stepper3: StepperRequest,
    pub servo1: Option<u32>,
    pub servo2: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponsePacket {
    pub status: bool,
}
