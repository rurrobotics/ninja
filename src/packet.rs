use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestPacket {
    Game,
    GripperOpen,
    GripperClose,
    ExtensionPush,
    ExtensionPull,
    Drive(i32),
    Turn(i32),
    TestExtension,
    TestRotation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponsePacket {
    pub status: bool,
}
