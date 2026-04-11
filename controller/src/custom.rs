use serde::Deserialize;

use crate::{Color, Action};

#[derive(Debug, Deserialize)]
#[serde(tag = "name", deny_unknown_fields)]
pub enum JsonAction {
    GripperOpen,
    GripperClose,
    ExtensionPush,
    ExtensionPull,
    Drive { speed: i32 },
    Turn { angle: i32 },
    SetDrivetrainFrequency { frequency: u32 },
    SetExtensionFrequency { frequency: u32 },
    SetProximityEnable { enable: bool },
    SetProximityThreshold { threshold: u32 },
    SetDrivetrainEnable { enable: bool },
    SetExtensionEnable { enable: bool },
    SetColor { color: Color },
}

impl From<JsonAction> for Action {
    fn from(a: JsonAction) -> Self {
        match a {
            JsonAction::GripperOpen => Action::GripperOpen,
            JsonAction::GripperClose => Action::GripperClose,
            JsonAction::ExtensionPush => Action::ExtensionPush,
            JsonAction::ExtensionPull => Action::ExtensionPull,
            JsonAction::Drive { speed } => Action::Drive(speed),
            JsonAction::Turn { angle } => Action::Turn(angle),
            JsonAction::SetDrivetrainFrequency { frequency } => {
                Action::SetDrivetrainFrequency(frequency)
            }
            JsonAction::SetExtensionFrequency { frequency } => {
                Action::SetExtensionFrequency(frequency)
            }
            JsonAction::SetProximityEnable { enable } => Action::SetProximityEnable(enable),
            JsonAction::SetProximityThreshold { threshold } => {
                Action::SetProximityThreshold(threshold)
            }
            JsonAction::SetDrivetrainEnable { enable } => Action::SetDrivetrainEnable(enable),
            JsonAction::SetExtensionEnable { enable } => Action::SetExtensionEnable(enable),
            JsonAction::SetColor { color } => Action::SetColor(color),
        }
    }
}
