#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::allow_attributes)]
#![warn(clippy::if_then_some_else_none)]
#![warn(clippy::redundant_type_annotations)]
#![warn(clippy::str_to_string)]
#![warn(clippy::string_to_string)]
#![warn(clippy::unseparated_literal_suffix)]
#![warn(clippy::unwrap_in_result)]
#![warn(clippy::unwrap_used)]

/// Low-level OSC message handling
pub mod osc;
/// X32 Types and OSC Reflections
pub mod x32;
/// X32 state machine
pub mod state;

pub use state::X32Console;
pub use state::X32Fader;
pub use x32::FaderType;
pub use x32::ConsoleRequest;
pub use osc::Buffer as OSCBuffer;
pub use osc::Packet as OSCPacket;
pub use osc::Bundle as OSCBundle;
pub use osc::Message as OSCMessage;

/// /xremote command
// #[expect(dead_code)]
pub const XREMOTE:[u8;12] = [0x2f, 0x78, 0x72, 0x65, 0x6d, 0x6f, 0x74, 0x65, 0x0, 0x0, 0x0, 0x0];
