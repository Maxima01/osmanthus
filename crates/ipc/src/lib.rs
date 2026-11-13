//! Protocol contracts only; no named pipe or privileged server is exposed.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

pub const PROTOCOL_VERSION: u16 = 1;
pub const MAX_FRAME_BYTES: usize = 64 * 1024;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub version: u16,
    pub request_id: u64,
    pub request: Query,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Query {
    Health,
    History,
    Session { id: String },
}

pub fn write_frame(mut output: impl Write, frame: &Frame) -> Result<()> {
    ensure!(
        frame.version == PROTOCOL_VERSION,
        "Unsupported protocol version"
    );
    let bytes = serde_json::to_vec(frame)?;
    ensure!(bytes.len() <= MAX_FRAME_BYTES, "Frame exceeds size limit");
    output.write_all(&(bytes.len() as u32).to_le_bytes())?;
    output.write_all(&bytes)?;
    Ok(())
}

pub fn read_frame(mut input: impl Read) -> Result<Frame> {
    let mut prefix = [0; 4];
    input.read_exact(&mut prefix)?;
    let size = u32::from_le_bytes(prefix) as usize;
    ensure!(size > 0 && size <= MAX_FRAME_BYTES, "Invalid frame size");
    let mut bytes = vec![0; size];
    input.read_exact(&mut bytes)?;
    let frame: Frame = serde_json::from_slice(&bytes)?;
    ensure!(
        frame.version == PROTOCOL_VERSION,
        "Unsupported protocol version"
    );
    Ok(frame)
}
