use heapless::Vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Action {
    GripperOpen,   // 0
    GripperClose,  // 1
    ExtensionPush, // 2
    ExtensionPull, // 3
    Drive(i32),    // 4
    Turn(i32),     // 5

    SetDrivetrainFrequency(u32), // 6
    SetExtensionFrequency(u32),  // 7
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestPacket {
    Game,                    // 0
    Action(Action),          // 1
    Custom(Vec<Action, 64>), // 2

    // Number of times, distance/angle
    TestExtension(u32),   // 3
    TestRotation(u32),    // 4
    TestSquare(u32, u32), // 5
    TestLine(u32, u32),   // 6
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponsePacket {
    pub status: bool,
}
