use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestPacket {
    Game, // = 0,
    GripperOpen, // = 1,
    GripperClose, // = 2,
    ExtensionPush, // = 3,
    ExtensionPull, // = 4,
    Drive(i32), // = 5,
    Turn(i32), // = 6,
    TestExtension, // = 7,
    TestRotation, // = 8,
    TestSquare(u32), // = 9,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponsePacket {
    pub status: bool,
}
