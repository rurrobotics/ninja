use std::{net::SocketAddr, path::PathBuf};

use crate::{Action, Color, RequestPacket, custom::JsonAction};
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use json_comments::StripComments;

use crate::packet;

#[derive(Parser, Debug)]
#[command(version, about = "Encode robot control packets via postcard/serde")]
pub struct Cli {
    #[arg(long, short)]
    pub address: SocketAddr,

    #[command(subcommand)]
    pub command: PacketCommand,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum CliColor {
    Yellow,
    Blue,
}

impl From<CliColor> for Color {
    fn from(c: CliColor) -> Self {
        match c {
            CliColor::Yellow => Color::Yellow,
            CliColor::Blue => Color::Blue,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum PacketCommand {
    Game,
    Action {
        #[command(subcommand)]
        action: ActionCommand,
    },
    Custom {
        file: PathBuf,
    },
    TestExtension {
        times: u32,
    },
    TestRotation {
        times: u32,
    },
    TestSquare {
        times: u32,
        distance: u32,
    },
    TestLine {
        times: u32,
        distance: u32,
    },
}

impl TryFrom<PacketCommand> for RequestPacket {
    type Error = anyhow::Error;

    fn try_from(cmd: PacketCommand) -> Result<Self> {
        let packet = match cmd {
            PacketCommand::Game => RequestPacket::Game,
            PacketCommand::Action { action } => RequestPacket::Action(Action::from(action)),
            PacketCommand::Custom { file } => {
                let content = std::fs::read_to_string(&file)?;
                let stripped = StripComments::new(content.as_bytes());
                let json_actions: Vec<JsonAction> = serde_json::from_reader(stripped)?;
                anyhow::ensure!(
                    json_actions.len() <= packet::COMMAND_LENGTH_LIMIT,
                    "{} actions provided, maximum is {}",
                    json_actions.len(),
                    packet::COMMAND_LENGTH_LIMIT,
                );
                let mut actions: heapless::Vec<Action, { packet::COMMAND_LENGTH_LIMIT }> =
                    heapless::Vec::new();
                for a in json_actions {
                    actions.push(a.into()).unwrap();
                }
                RequestPacket::Custom(actions)
            }
            PacketCommand::TestExtension { times } => RequestPacket::TestExtension(times),
            PacketCommand::TestRotation { times } => RequestPacket::TestRotation(times),
            PacketCommand::TestSquare { times, distance } => {
                RequestPacket::TestSquare(times, distance)
            }
            PacketCommand::TestLine { times, distance } => RequestPacket::TestLine(times, distance),
        };
        Ok(packet)
    }
}

#[derive(Subcommand, Debug)]
pub enum ActionCommand {
    GripperOpen,
    GripperClose,
    ExtensionPush,
    ExtensionPull,
    Drive { speed: i32 },
    Turn { angle: i32 },
    SetExtensionFrequency { frequency: u32 },
    SetProximityThreshold { threshold: u32 },
    EnableProximity,
    DisableProximity,
    EnableDrivetrain,
    DisableDrivetrain,
    EnableExtension,
    DisableExtension,
    SetColor { color: CliColor },
    SetAcceleration { acceleration: f64 },
    SetMaxSpeed { max_speed: f64 },
    // SetPCoefficient { p: f64 },
}

impl From<ActionCommand> for Action {
    fn from(cmd: ActionCommand) -> Self {
        match cmd {
            ActionCommand::GripperOpen => Action::GripperOpen,
            ActionCommand::GripperClose => Action::GripperClose,
            ActionCommand::ExtensionPush => Action::ExtensionPush,
            ActionCommand::ExtensionPull => Action::ExtensionPull,
            ActionCommand::Drive { speed } => Action::Drive(speed),
            ActionCommand::Turn { angle } => Action::Turn(angle),
            ActionCommand::SetExtensionFrequency { frequency } => {
                Action::SetExtensionFrequency(frequency)
            }
            ActionCommand::SetProximityThreshold { threshold } => {
                Action::SetProximityThreshold(threshold)
            }
            ActionCommand::EnableProximity => Action::SetProximityEnable(true),
            ActionCommand::DisableProximity => Action::SetProximityEnable(false),
            ActionCommand::EnableDrivetrain => Action::SetDrivetrainEnable(true),
            ActionCommand::DisableDrivetrain => Action::SetDrivetrainEnable(false),
            ActionCommand::EnableExtension => Action::SetExtensionEnable(true),
            ActionCommand::DisableExtension => Action::SetExtensionEnable(false),
            ActionCommand::SetColor { color } => Action::SetColor(color.into()),
            ActionCommand::SetAcceleration { acceleration } => {
                Action::SetAcceleration(acceleration)
            }
            ActionCommand::SetMaxSpeed { max_speed } => Action::SetMaxSpeed(max_speed),
            // ActionCommand::SetPCoefficient { p } => Action::SetPCoefficient(p),
        }
    }
}
