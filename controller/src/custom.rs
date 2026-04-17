use serde::Deserialize;

use crate::{Action, Color};

#[derive(Debug, Deserialize)]
#[serde(tag = "name", deny_unknown_fields)]
pub enum JsonAction {
    GripperOpen,
    GripperClose,
    ExtensionPush,
    ExtensionPull,
    Drive { speed: i32 },
    Turn { angle: i32 },
    SetExtensionFrequency { frequency: u32 },
    SetProximityEnable { enable: bool },
    SetProximityThreshold { threshold: u32 },
    SetDrivetrainEnable { enable: bool },
    SetExtensionEnable { enable: bool },
    SetColor { color: Color },
    SetAcceleration { acceleration: f64 },
    SetMaxSpeed { max_speed: f64 },
    StopDrivetrain,
    StartDrivetrain,
    // SetPCoefficient { p: f64 },
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
            JsonAction::SetAcceleration { acceleration } => Action::SetAcceleration(acceleration),
            JsonAction::SetMaxSpeed { max_speed } => Action::SetMaxSpeed(max_speed),
            JsonAction::StopDrivetrain => Action::StopDrivetrain,
            JsonAction::StartDrivetrain => Action::StartDrivetrain,
            // JsonAction::SetPCoefficient { p } => Action::SetPCoefficient(p),
        }
    }
}
