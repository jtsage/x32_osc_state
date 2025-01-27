use super::enums::Error;


/// X32 Utility functions
pub mod util;
/// `osc::Message` to the console
mod to_console;
/// `osc::Message` from the console
mod from_console;
mod faders;

pub use faders::FaderUpdate;

pub use to_console::ConsoleRequest;
pub use from_console::{ConsoleMessage, CueUpdate, SceneUpdate, SnippetUpdate};


