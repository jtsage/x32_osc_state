#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![warn(clippy::allow_attributes)]
#![warn(clippy::default_trait_access)]
#![warn(clippy::derive_partial_eq_without_eq)]
#![warn(clippy::equatable_if_let)]
#![warn(clippy::from_iter_instead_of_collect)]
#![warn(clippy::if_not_else)]
#![warn(clippy::if_then_some_else_none)]
#![warn(clippy::implicit_clone)]
#![warn(clippy::inefficient_to_string)]
#![warn(clippy::manual_is_variant_and)]
#![warn(clippy::manual_let_else)]
#![warn(clippy::manual_ok_or)]
#![warn(clippy::map_unwrap_or)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::needless_collect)]
#![warn(clippy::needless_pass_by_ref_mut)]
#![warn(clippy::option_if_let_else)]
#![warn(clippy::or_fun_call)]
#![warn(clippy::partial_pub_fields)]
#![warn(clippy::pub_use)]
#![warn(clippy::redundant_type_annotations)]
#![warn(clippy::renamed_function_params)]
#![warn(clippy::return_self_not_must_use)]
#![warn(clippy::single_call_fn)]
#![warn(clippy::str_to_string)]
#![warn(clippy::string_to_string)]
#![warn(clippy::suspicious_operation_groupings)]
#![warn(clippy::unseparated_literal_suffix)]
#![warn(clippy::unwrap_in_result)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::use_self)]
// #![warn(clippy::non_std_lazy_statics)]

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
