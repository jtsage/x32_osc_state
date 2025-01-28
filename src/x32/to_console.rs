use crate::osc::{Message, Buffer};
use super::super::enums::FaderIndex;
// use super::util;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
/// Get info from the console
pub enum ConsoleRequest {
    /// Matrix with index
    Fader(FaderIndex),
    /// Cue, Scene, and Snippet list
    ShowInfo(),
    /// Show mode
    ShowMode(),
    /// Current cue index
    CurrentCue(),
    /// /xremote command
    KeepAlive(),
}

impl ConsoleRequest {
    /// Full update of all tracked data request
    #[must_use]
    pub fn full_update() -> Vec<Buffer> {
        let mut buffers:Vec<Buffer> = vec![];

        buffers.extend(Self::ShowInfo());
        buffers.extend(Self::ShowMode());
        buffers.extend(Self::CurrentCue());
        buffers.extend(Self::Fader(FaderIndex::Main(1)));
        buffers.extend(Self::Fader(FaderIndex::Main(2)));

        let aux:Vec<Buffer> = (1..=8).flat_map(|i|Self::Fader(FaderIndex::Aux(i))).collect();
        let mtx:Vec<Buffer> = (1..=6).flat_map(|i|Self::Fader(FaderIndex::Matrix(i))).collect();
        let bus:Vec<Buffer> = (1..=16).flat_map(|i|Self::Fader(FaderIndex::Bus(i))).collect();
        let dca:Vec<Buffer> = (1..=8).flat_map(|i|Self::Fader(FaderIndex::Dca(i))).collect();
        let ch:Vec<Buffer>  = (1..=32).flat_map(|i|Self::Fader(FaderIndex::Channel(i))).collect();

        buffers.extend(aux);
        buffers.extend(mtx);
        buffers.extend(bus);
        buffers.extend(dca);
        buffers.extend(ch);
        buffers
    }
}


impl IntoIterator for ConsoleRequest {
    type Item = Buffer;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        <Self as Into<Vec<Buffer>>>::into(self).into_iter()
    }
}

impl From<ConsoleRequest> for Vec<Buffer> {
    fn from(value: ConsoleRequest) -> Self {
        match value {
            ConsoleRequest::Fader(v) => v.get_x32_update(),
            ConsoleRequest::ShowInfo() => vec![
                Message::new("/showdata").try_into().unwrap_or_default()
            ],
            ConsoleRequest::ShowMode() => vec![
                Message::new_string("/node", "-prefs/show_control").try_into().unwrap_or_default()
            ],
            ConsoleRequest::CurrentCue() => vec![
                Message::new_string("/node", "-show/prepos/current").try_into().unwrap_or_default()
            ],
            ConsoleRequest::KeepAlive() => vec![
                Message::new("/xremote").try_into().unwrap_or_default()
            ],
        }
    }
}