use crate::osc::{Message, Buffer};
use super::util;

#[derive(Debug, PartialEq, PartialOrd)]
/// Get info from the console
pub enum ConsoleRequest {
    /// Matrix with index
    Matrix(u8),
    /// Bus with index
    Bus(u8),
    /// DCA with index
    Dca(u8),
    /// Channel with index
    Channel(u8),
    /// Aux with index
    Aux(u8),
    /// Main with index, 1 = ST, 2 = M
    Main(u8),
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

        buffers.extend(ConsoleRequest::ShowInfo());
        buffers.extend(ConsoleRequest::ShowMode());
        buffers.extend(ConsoleRequest::CurrentCue());
        buffers.extend(ConsoleRequest::Main(1));
        buffers.extend(ConsoleRequest::Main(2));

        let aux:Vec<Buffer> = (1..=6).flat_map(ConsoleRequest::Aux).collect();
        let mtx:Vec<Buffer> = (1..=6).flat_map(ConsoleRequest::Matrix).collect();
        let bus:Vec<Buffer> = (1..=16).flat_map(ConsoleRequest::Bus).collect();
        let dca:Vec<Buffer> = (1..=8).flat_map(ConsoleRequest::Dca).collect();
        let ch:Vec<Buffer>  = (1..=32).flat_map(ConsoleRequest::Channel).collect();

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
        <ConsoleRequest as Into<Vec<Buffer>>>::into(self).into_iter()
    }
}

impl From<ConsoleRequest> for Vec<Buffer> {
    fn from(value: ConsoleRequest) -> Self {
        match value {
            ConsoleRequest::Bus(v) => vec![
                util::new_node_buffer(format!("bus/{v:02}/mix")),
                util::new_node_buffer(format!("bus/{v:02}/config")),
            ],
            ConsoleRequest::Dca(v) => vec![
                util::new_node_buffer(format!("dca/{v}/")),
                util::new_node_buffer(format!("dca/{v}/config")),
            ],
            ConsoleRequest::Channel(v) => vec![
                util::new_node_buffer(format!("ch/{v:02}/mix")),
                util::new_node_buffer(format!("ch/{v:02}/config")),
            ],
            ConsoleRequest::Aux(v) => vec![
                util::new_node_buffer(format!("auxin/{v:02}/mix")),
                util::new_node_buffer(format!("auxin/{v:02}/config")),
            ],
            ConsoleRequest::Matrix(v) => vec![
                util::new_node_buffer(format!("mtx/{v:02}/mix")),
                util::new_node_buffer(format!("mtx/{v:02}/config")),
            ],
            ConsoleRequest::ShowInfo() => vec![
                Message::new("/showdata").try_into().unwrap_or_default()
            ],
            ConsoleRequest::ShowMode() => vec![
                util::new_node_buffer(String::from("-prefs/show_control"))
            ],
            ConsoleRequest::CurrentCue() => vec![
                util::new_node_buffer(String::from("-show/prepos/current"))
            ],
            ConsoleRequest::KeepAlive() => vec![
                Message::new("/xremote").try_into().unwrap_or_default()
            ],
            ConsoleRequest::Main(v) => vec![
                util::new_node_buffer(format!("main/{}/mix", if v == 1 { "st" } else { "m"} )),
                util::new_node_buffer(format!("main/{}/config", if v == 1 { "st" } else { "m"} ))
            ],
        }
    }
}