/// [`crate::osc::Message`] to the console
mod to_console;
/// [`crate::osc::Message`] from the console
mod from_console;
/// Update packets for state
pub mod updates;

pub use to_console::ConsoleRequest;
pub use from_console::ConsoleMessage;
