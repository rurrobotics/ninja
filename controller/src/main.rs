use anyhow::{Context, Result};
use clap::Parser;
use std::io::Write;
use std::net::TcpStream;

mod cli;
mod custom;
mod packet;

use crate::{
    cli::Cli,
    packet::{Action, Color, RequestPacket, ResponsePacket},
};

use std::io::Read;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let packet = RequestPacket::try_from(cli.command)?;
    let bytes = postcard::to_allocvec(&packet).context("Failed to serialize packet")?;

    let mut stream = TcpStream::connect(&cli.address)?;
    stream.write_all(&bytes)?;

    // TODO: Make this match firmware code
    let mut buf = [0u8; 4];
    stream.read(&mut buf)?;
    let response: ResponsePacket =
        postcard::from_bytes(&buf).context("Failed to deserialize response")?;

    println!("{:#?}", response);

    Ok(())
}
