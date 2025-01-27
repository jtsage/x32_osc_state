use std::fmt;
// use lazy_static::lazy_static;
use std::sync::LazyLock;
use regex::Regex;

static LVL_STRING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?<level>[+\-0-9.]+)").expect("unable to compile pattern")
});

// lazy_static! {
//     static ref LVL_STRING: Regex = Regex::new(r"^(?<level>[+\-0-9.]+)").expect("unable to compile pattern");
//     static ref NODE_STRING: Regex = Regex::new(r#"[^\s"]+|"([^"]*)""#).expect("unable to compile pattern");
// }

/// bundle tag, "#bundle", 8-byte
pub const BUNDLE_TAG:[u8;8] = [0x23, 0x62, 0x75, 0x6e, 0x64, 0x6c, 0x65, 0x0];
/// simple ignored node message - "-prefs/name", 24-byte
pub const X32_KEEP_ALIVE:[u8;24] = [0x2f, 0x6e, 0x6f, 0x64, 0x65, 0x0, 0x0, 0x0, 0x2c, 0x73, 0x0, 0x0, 0x2d, 0x70, 0x72, 0x65, 0x66, 0x73, 0x2f, 0x6e, 0x61, 0x6d, 0x65, 0x0];
/// X32 remote command "/xremote", 12-byte
pub const X32_XREMOTE:[u8;12] = [0x2f, 0x78, 0x72, 0x65, 0x6d, 0x6f, 0x74, 0x65, 0x0, 0x0, 0x0, 0x0];

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
/// Error type for crate
pub enum Error {
    /// Packet / buffer errors
    Packet(PacketError),
    /// OSC type errors
    OSC(OSCError),
    /// X32 state errors
    X32(X32Error)
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
/// Packet (buffer) Errors
pub enum PacketError {
    /// buffer is not 4-byte aligned
    NotFourByte,
    /// buffer does not end with 1 or more nulls
    UnterminatedString,
    /// buffer not large enough for operation
    Underrun,
    /// Invalid original message
    Invalid
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
/// OSC Type conversion errors
pub enum OSCError {
    /// String from bytes failed
    ConvertFromString,
    /// Address is not valid
    AddressContent,
    /// Unknown OSC type
    UnknownType,
    /// Invalid type conversion (named type)
    InvalidTypeFlag,
    /// Invalid type conversion (type -> primitive
    InvalidTypeConversion,
    /// Time underflow
    InvalidTimeUnderflow,
    /// Time overflow
    InvalidTimeOverflow,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
/// X32 state errors
pub enum X32Error {
    /// Fader does not exist
    InvalidFader,
    /// Packet was not understood
    UnimplementedPacket,
    /// Packet was poorly formed (missing data?)
    MalformedPacket
}

// MARK: BufferError->String
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Show Control Mode
pub enum ShowMode {
    /// Tracking cues
    Cues,
    /// Tracking scenes
    Scenes,
    /// Tracking snippets
    Snippets
}

impl ShowMode {
    /// Get from an integer
    pub fn from_int(v : i32) -> Self {
        match v {
            1 => Self::Scenes,
            2 => Self::Snippets,
            _ => Self::Cues
        }
    }

    /// Get from a string
    pub fn from_str(v : &str) -> Self {
        match v {
            "SCENES" => Self::Scenes,
            "SNIPPETS" => Self::Snippets,
            _ => Self::Cues
        }
    }
}

#[derive(Debug, Default, PartialEq, PartialOrd, Clone, Eq, Ord)]
/// Types of faders
pub enum FaderIndex {
    /// auxin's, 1-8 (last 2 are USB typically)
    Aux(u8),
    /// Matrix sends, 1-6
    Matrix(u8),
    /// Main = 1, Mono/M/C = 2
    Main(u8),
    /// Channels, 1-32
    Channel(u8),
    /// DCA, 1-8
    Dca(u8),
    /// Mix Bus, 1-16
    Bus(u8),
    /// Unknown fader type
    #[default]
    Unknown
}

impl FaderIndex {
    pub fn default_label(&self) -> String {
        match self {
            Self::Aux(v) => format!("Aux{v:02}"),
            Self::Matrix(v) => format!("Mtx{v:02}"),
            Self::Main(v) => if *v == 2_u8 { String::from("M/C") } else { String::from("Main") },
            Self::Channel(v) => format!("Ch{v:02}",),
            Self::Dca(v) => format!("DCA{v}"),
            Self::Bus(v) => format!("MixBus{v:02}"),
            Self::Unknown => String::new(),
        }
    }
    pub fn get_address(&self) -> String {
        match self {
            Self::Unknown => String::new(),
            Self::Aux(v) => format!("/aux/{v:02}"),
            Self::Matrix(v) => format!("/mtx/{v:02}"),
            Self::Main(v) => if *v == 2_u8 { String::from("/main/m") } else { String::from("/main/st") },
            Self::Channel(v) => format!("/ch/{v:02}"),
            Self::Dca(v) => format!("/dca/{v}"),
            Self::Bus(v) => format!("/bus/{v:02}"),
        }
    }
}

type FaderIndexStrInt = (String, i32);
type FaderIndexStrStr = (String, String);

impl TryFrom<FaderIndexStrInt> for FaderIndex {
    type Error = Error;

    fn try_from(value: FaderIndexStrInt) -> Result<Self, Self::Error> {
        if let Ok(index) = u8::try_from(value.1) {
            match value.0.as_str() {
                _ if index == 0 => Err(Error::X32(X32Error::InvalidFader)),
                "mtx" if index <= 6 => Ok(Self::Matrix(index)),
                "aux" if index <= 8 => Ok(Self::Aux(index)),
                "dca" if index <= 8 => Ok(Self::Dca(index)),
                "main" if index <= 2 => Ok(Self::Main(index)),
                "ch" if index <= 32 => Ok(Self::Channel(index)),
                "bus" if index <= 16 => Ok(Self::Bus(index)),
                _ => Err(Error::X32(X32Error::InvalidFader))
            }
        } else {
            Err(Error::X32(X32Error::InvalidFader))
        }
    }
}

impl TryFrom<FaderIndexStrStr> for FaderIndex {
    type Error = Error;

    fn try_from(value: FaderIndexStrStr) -> Result<Self, Self::Error> {
        if value.0 == "main" {
            if value.1 == "m" {
                (value.0, 2).try_into()
            } else {
                (value.0, 1).try_into()
            }
        } else if let Ok(num) = value.1.parse::<i32>() {
            (value.0, num).try_into()
        } else {
            Err(Error::X32(X32Error::InvalidFader))
        }
    }
}


#[derive(Debug, Clone)]
pub struct Fader {
    /// fader index, with type. 
    source : FaderIndex,
    /// scribble strip label
    label : String,
    /// level of fader, as number
    level : f32,
    /// mute status, as bool
    is_on : bool
}


impl Fader {
    /// create new fader
    #[must_use]
    pub fn new(source : FaderIndex) -> Self {
        Self {
            source,
            label : String::new(),
            level : 0_f32,
            is_on : false
        }
    }

    /// get fader label or default name
    #[must_use]
    pub fn name(&self) -> String {
        if self.label.is_empty() {
            self.source.default_label()
        } else {
            self.label.clone()
        }
    }

    /// get fader level
    #[must_use]
    pub fn level(&self) -> (f32, String) {
        ( self.level, Self::level_to_string(self.level) )
    }

    /// get fader mute status
    #[must_use]
    pub fn is_on(&self) -> (bool, String) {
        ( self.is_on, String::from(if self.is_on { "ON" } else { "OFF" }) )
    }

    /// update fader from OSC data
    pub fn update(&mut self, update : super::x32::FaderUpdate) {
        if let Some(new_level) = update.level {
            self.level = new_level;
        }

        if let Some(new_is_on) = update.is_on {
            self.is_on = new_is_on;
        }

        if let Some(new_label) = update.label {
            self.label = new_label;
        }
    }

    
    /// Get is on property from ON/OFF
    #[must_use]
    pub fn is_on_from_string(v : &str) -> bool { v == "ON" }

    /// Get string level from float
    #[must_use]
    pub fn level_to_string(v : f32) -> String {
        let c_value = match v {
            d if d >= 0.5 => v * 40_f32 - 30_f32,
            d if d >= 0.25 => v * 80_f32 - 50_f32,
            d if d >= 0.0625 => v * 160_f32 - 70_f32,
            _ => v * 480_f32 - 90_f32
        };

        match c_value {
            d if (-0.05..=0.05).contains(&d)  => String::from("+0.0 dB"),
            d if d <= -89.9 => String::from("-oo dB"),
            d if d < 0_f32   => format!("{c_value:.1} dB"),
            _ => format!("+{c_value:.1} dB")
        }
    }

    /// get level as float from String
    #[must_use]
    pub fn level_from_string(input : &str) -> f32 {
        if input.starts_with("-oo") {
            0_f32
        } else if let Some(caps) = LVL_STRING.captures(input) {
            let lvl = match caps["level"].parse::<f32>() {
                Ok(d) if d < -60.0_f32 => (d + 90.0_f32) / 480.0_f32,
                Ok(d) if d < -30.0_f32 => (d + 70.0_f32) / 160.0_f32,
                Ok(d) if d < -10.0_f32 => (d + 50.0_f32) / 80.0_f32,
                Ok(d) => (d + 30.0_f32) / 40.0_f32,
                Err(_) => 0_f32
            };
            let f_lvl = (lvl * 1023.5).trunc() / 1023.0;
            (f_lvl * 10000.0).round() / 10000.0
        } else {
            0_f32
        }
    }
}