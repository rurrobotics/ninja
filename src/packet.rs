use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestPacket {
    GripperOpen,
    GripperClose,
    ExtensionPush,
    ExtensionPull,
    LeftStep(i32),
    RightStep(i32),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponsePacket {
    pub status: bool,
}
