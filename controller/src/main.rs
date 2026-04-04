use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::net::TcpStream;
use std::path::PathBuf;
use std::{io::Write, net::SocketAddr};

mod packet;

use crate::packet::{Action, RequestPacket};

#[derive(Debug, Deserialize)]
#[serde(tag = "name", deny_unknown_fields)]
enum JsonAction {
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
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about = "Encode robot control packets via postcard/serde")]
struct Cli {
    #[arg(long, short)]
    address: SocketAddr,

    #[command(subcommand)]
    command: PacketCommand,
}

#[derive(Subcommand, Debug)]
enum PacketCommand {
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

#[derive(Subcommand, Debug)]
enum ActionCommand {
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
            ActionCommand::SetDrivetrainFrequency { frequency } => {
                Action::SetDrivetrainFrequency(frequency)
            }
            ActionCommand::SetExtensionFrequency { frequency } => {
                Action::SetExtensionFrequency(frequency)
            }
            ActionCommand::SetProximityEnable { enable } => Action::SetProximityEnable(enable),
            ActionCommand::SetProximityThreshold { threshold } => {
                Action::SetProximityThreshold(threshold)
            }
        }
    }
}

impl TryFrom<PacketCommand> for RequestPacket {
    type Error = anyhow::Error;

    fn try_from(cmd: PacketCommand) -> Result<Self> {
        let packet = match cmd {
            PacketCommand::Game => RequestPacket::Game,
            PacketCommand::Action { action } => RequestPacket::Action(Action::from(action)),
            PacketCommand::Custom { file } => {
                let content = std::fs::read_to_string(&file)?;
                let json_actions: Vec<JsonAction> = serde_json::from_str(&content)?;
                anyhow::ensure!(
                    json_actions.len() <= 64,
                    "{} actions provided, maximum is 64",
                    json_actions.len()
                );
                let mut actions: heapless::Vec<Action, 64> = heapless::Vec::new();
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    let packet = RequestPacket::try_from(cli.command)?;
    let bytes = postcard::to_allocvec(&packet).context("Failed to serialize packet")?;

    let mut stream = TcpStream::connect(&cli.address)?;

    stream.write_all(&bytes)?;

    Ok(())
}
