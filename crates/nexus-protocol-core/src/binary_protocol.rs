//! Binary serialization for high-performance messaging.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryMessage {
    pub version: u16,
    pub msg_type: u8,
    pub payload: Vec<u8>,
    pub checksum: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BinaryMsgType {
    Handshake = 0x01,
    HandshakeAck = 0x02,
    Execute = 0x03,
    ExecutionReady = 0x04,
    ExecuteInSandbox = 0x05,
    Stdout = 0x06,
    Stderr = 0x07,
    Exit = 0x08,
    ExecutionResult = 0x09,
    OllamaConnect = 0x10,
    OllamaConnected = 0x11,
    OllamaGenerate = 0x12,
    OllamaToken = 0x13,
    OllamaDone = 0x14,
    Error = 0xFF,
}

impl BinaryMsgType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(Self::Handshake),
            0x02 => Some(Self::HandshakeAck),
            0x03 => Some(Self::Execute),
            0x04 => Some(Self::ExecutionReady),
            0x05 => Some(Self::ExecuteInSandbox),
            0x06 => Some(Self::Stdout),
            0x07 => Some(Self::Stderr),
            0x08 => Some(Self::Exit),
            0x09 => Some(Self::ExecutionResult),
            0x10 => Some(Self::OllamaConnect),
            0x11 => Some(Self::OllamaConnected),
            0x12 => Some(Self::OllamaGenerate),
            0x13 => Some(Self::OllamaToken),
            0x14 => Some(Self::OllamaDone),
            0xFF => Some(Self::Error),
            _ => None,
        }
    }
}

pub const BINARY_PROTOCOL_VERSION: u16 = 0x0100;

pub fn calculate_checksum(payload: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for byte in payload {
        crc = crc.wrapping_mul(0xEDB88320).wrapping_add(*byte as u32);
    }
    crc ^ 0xFFFFFFFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum() {
        let data = b"Hello, World!";
        let crc = calculate_checksum(data);
        assert_ne!(crc, 0);
    }

    #[test]
    fn test_msg_type_conversion() {
        assert_eq!(BinaryMsgType::from_u8(0x01), Some(BinaryMsgType::Handshake));
        assert_eq!(BinaryMsgType::from_u8(0xFF), Some(BinaryMsgType::Error));
        assert_eq!(BinaryMsgType::from_u8(0x99), None);
    }
}