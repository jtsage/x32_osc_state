use std::fmt;
use std::str;

/// X32 Utility functions
pub mod util;
/// `osc::Message` to the console
mod to_console;
/// `osc::Message` from the console
mod from_console;

pub use to_console::ConsoleRequest;
pub use from_console::{ConsoleMessage, CueUpdate, SceneUpdate, SnippetUpdate};

#[derive(Debug, PartialEq, PartialOrd)]
/// Errors on incoming messages
pub enum Error {
    /// Fader is not covered, or out-of-bounds
    InvalidFader,
    /// Packet was not understood
    UnimplementedPacket,
    /// Packet was poorly formed
    MalformedPacket
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
/// Show Control Mode
pub enum ShowMode {
    /// Tracking cues
    Cues,
    /// Tracking scenes
    Scenes,
    /// Tracking snippets
    Snippets
}

#[derive(Debug, Default, PartialEq, PartialOrd, Clone)]
/// Types of faders
pub enum FaderType {
    /// auxin's, 1-8 (last 2 are USB typically)
    Aux,
    /// Matrix sends, 1-6
    Matrix,
    /// Main = 1, Mono/M/C = 2
    Main,
    /// Channels, 1-32
    Channel,
    /// DCA, 1-8
    Dca,
    /// Mix Bus, 1-16
    Bus,
    /// Unknown fader type
    #[default]
    Unknown
}

impl FaderType {
    /// Check is fader index (not number!) is within bounds
    #[must_use]
    pub fn check_bounds(&self, index : usize) -> bool {
        match self {
            FaderType::Matrix => index < 6,
            FaderType::Aux | FaderType::Dca => index < 8,
            FaderType::Main => index < 2,
            FaderType::Channel => index < 32,
            FaderType::Bus => index < 16,
            FaderType::Unknown => false,
        }
    }
}
impl str::FromStr for FaderType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auxin" => Ok(FaderType::Aux),
            "mtx"   => Ok(FaderType::Matrix),
            "main"  => Ok(FaderType::Main),
            "ch"    => Ok(FaderType::Channel),
            "dca"   => Ok(FaderType::Dca),
            "bus"   => Ok(FaderType::Bus),
            _       => Err(Error::InvalidFader)
        }
    }
}

impl fmt::Display for FaderType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            FaderType::Aux     => write!(f, "auxin"),
            FaderType::Matrix  => write!(f, "mtx"),
            FaderType::Main    => write!(f, "main"),
            FaderType::Channel => write!(f, "ch"),
            FaderType::Dca     => write!(f, "dca"),
            FaderType::Bus     => write!(f, "bus"),
            FaderType::Unknown => write!(f, ""),
        }
    }
}

/// Fader update processed
#[derive(Debug, PartialEq, PartialOrd)]
pub struct FaderUpdate {
    /// Type of fader
    pub source : FaderType,
    /// index of fader
    pub index : usize,
    /// scribble strip label
    pub label : Option<String>,
    /// level of fader, as number
    pub level : Option<f32>,
    /// mute status, as bool
    pub is_on : Option<bool>
}

impl Default for FaderUpdate {
    fn default() -> Self { FaderUpdate {
        source : FaderType::Unknown,
        index : 0,
        label : None,
        level : None,
        is_on : None
    } }
}

/// mix message from node s~... osc message
type NodeMixMessage = (String, String, String, String);
/// fader level from /fader/... message
type StdFaderMessage = (String, String, f32);
/// mute from /fader/... message
type StdMuteMessage = (String, String, i32);
/// name from /fader/ or node s~ message
type NameMessage = (String, String, String);

impl TryFrom<StdMuteMessage> for FaderUpdate {
    type Error = Error;

    fn try_from(v: StdMuteMessage) -> Result<Self, Self::Error> {
        if let Ok(source) = v.0.parse::<FaderType>() {
            let index = util::fader_num_to_idx(v.1.as_str());
            if source.check_bounds(index) {
                Ok(FaderUpdate {
                    source, index,
                    is_on : Some(v.2 == 1),
                    ..Default::default()
                })
            } else {
                Err(Error::InvalidFader)
            }
        } else {
            Err(Error::InvalidFader)
        }
    }
}

impl TryFrom<StdFaderMessage> for FaderUpdate {
    type Error = Error;

    fn try_from(v: StdFaderMessage) -> Result<Self, Self::Error> {
        if let Ok(source) = v.0.parse::<FaderType>() {
            let index = util::fader_num_to_idx(v.1.as_str());
            if source.check_bounds(index) {
                Ok(FaderUpdate {
                    source, index,
                    level : Some(v.2),
                    ..Default::default()
                })
            } else {
                Err(Error::InvalidFader)
            }
        } else {
            Err(Error::InvalidFader)
        }
    }
}

impl TryFrom<NodeMixMessage> for FaderUpdate {
    type Error = Error;

    fn try_from(v: NodeMixMessage) -> Result<Self, Self::Error> {
        if let Ok(source) = v.0.parse::<FaderType>() {
            let index = util::fader_num_to_idx(v.1.as_str());
            if source.check_bounds(index) {
                Ok(FaderUpdate {
                    source, index,
                    level : Some(util::level_from_string(&v.3)),
                    is_on : Some(util::is_on_from_string(&v.2)),
                    ..Default::default()
                })
            } else {
                Err(Error::InvalidFader)
            }
        } else {
            Err(Error::InvalidFader)
        }
    }
}

impl TryFrom<NameMessage> for FaderUpdate {
    type Error = Error;

    fn try_from(v: NameMessage) -> Result<Self, Self::Error> {
        if let Ok(source) = v.0.parse::<FaderType>() {
            let index = util::fader_num_to_idx(v.1.as_str());
            if source.check_bounds(index) {
                Ok(FaderUpdate {
                    source, index,
                    label : Some(v.2.clone()),
                    ..Default::default()
                })
            } else {
                Err(Error::InvalidFader)
            }
        } else {
            Err(Error::InvalidFader)
        }
    }
}
