use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestPacket {
    Game,                        // 0
    GripperOpen,                 // 1
    GripperClose,                // 2
    ExtensionPush,               // 3
    ExtensionPull,               // 4
    Drive(i32),                  // 5
    Turn(i32),                   // 6
    SetDrivetrainFrequency(u32), // 7
    SetExtensionFrequency(u32),  // 8
    TestExtension,               // 9
    TestRotation,                // 10
    TestSquare(u32),             // 11
    TestLine(u32),               // 12
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponsePacket {
    pub status: bool,
}
