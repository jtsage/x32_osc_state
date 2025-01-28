use std::fmt;
// use lazy_static::lazy_static;
use std::sync::LazyLock;
use regex::Regex;

/// Pull fader level from node string
static LVL_STRING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?<level>[+\-0-9.]+)").expect("unable to compile pattern")
});

/// Split node string on whitespace, skipping quoted items
pub static NODE_STRING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"[^\s"]+|"([^"]*)""#).expect("unable to compile pattern")
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
    #[must_use]
    #[inline]
    pub fn from_int(v : i32) -> Self {
        match v {
            1 => Self::Scenes,
            2 => Self::Snippets,
            _ => Self::Cues
        }
    }

    /// Get from a string
    #[must_use]
    #[inline]
    pub fn from_const(v : &str) -> Self {
        match v {
            "SCENES" => Self::Scenes,
            "SNIPPETS" => Self::Snippets,
            _ => Self::Cues
        }
    }
}

/// Show cue structure
#[derive(Debug, Clone)]
pub struct ShowCue {
    /// Displayed cue number
    pub cue_number : String,
    /// Cue name
    pub name : String,
    /// associated snippet (or None)
    pub snippet : Option<usize>,
    /// associated scene (or None)
    pub scene : Option<usize>,
}

#[derive(Debug, Default, PartialEq, PartialOrd, Clone, Eq, Ord)]
/// Types of faders
pub enum FaderIndex {
    /// auxin's, 1-8 (last 2 are USB typically)
    Aux(usize),
    /// Matrix sends, 1-6
    Matrix(usize),
    /// Main = 1, Mono/M/C = 2
    Main(usize),
    /// Channels, 1-32
    Channel(usize),
    /// DCA, 1-8
    Dca(usize),
    /// Mix Bus, 1-16
    Bus(usize),
    /// Unknown fader type
    #[default]
    Unknown
}

impl FaderIndex {
    /// Get index (1-based) of the fader
    #[must_use]
    pub fn get_index(&self) -> usize {
        match self {
            Self::Aux(v) | Self::Matrix(v) | Self::Bus(v) |
            Self::Main(v) | Self::Channel(v) | Self::Dca(v) => *v,
            Self::Unknown => 0,
        }
    }

    /// Get the default label for this fader
    #[must_use]
    pub fn default_label(&self) -> String {
        match self {
            Self::Aux(v) => format!("Aux{v:02}"),
            Self::Matrix(v) => format!("Mtx{v:02}"),
            Self::Main(v) => if *v == 2 { String::from("M/C") } else { String::from("Main") },
            Self::Channel(v) => format!("Ch{v:02}",),
            Self::Dca(v) => format!("DCA{v}"),
            Self::Bus(v) => format!("MixBus{v:02}"),
            Self::Unknown => String::new(),
        }
    }

    /// Get the X32 address for this fader
    #[must_use]
    pub fn get_x32_address(&self) -> String {
        match self {
            Self::Unknown => String::new(),
            Self::Aux(v) => format!("/auxin/{v:02}"),
            Self::Matrix(v) => format!("/mtx/{v:02}"),
            Self::Main(v) => if *v == 2 { String::from("/main/m") } else { String::from("/main/st") },
            Self::Channel(v) => format!("/ch/{v:02}"),
            Self::Dca(v) => format!("/dca/{v}"),
            Self::Bus(v) => format!("/bus/{v:02}"),
        }
    }

    /// Get the VOR output address for this fader
    #[must_use]
    pub fn get_vor_address(&self) -> String {
        match self {
            Self::Main(v) => format!("/main/{v:02}"),
            _ => self.get_x32_address(),
        }
    }

    /// Get a vector of OSC messages that will force
    /// the X32 to update this fader
    #[must_use]
    pub fn get_x32_update(&self) -> Vec<super::osc::Buffer> {
        let address = self.get_x32_address();
        match self {
            Self::Unknown => vec![super::osc::Buffer::default()],
            Self::Dca(_) => vec![
                super::osc::Message::new_string("/node", &address).try_into().unwrap_or_default(),
                super::osc::Message::new_string("/node", &format!("{address}/config")).try_into().unwrap_or_default(),
            ],
            _ => vec![
                super::osc::Message::new_string("/node", &format!("{address}/mix")).try_into().unwrap_or_default(),
                super::osc::Message::new_string("/node", &format!("{address}/config")).try_into().unwrap_or_default(),
            ],
        }
    }
}

/// Fader index - string type, integer index (1-based)
type FaderIndexStrInt = (String, i32);
/// Fader index - string type, string index (1-based)
type FaderIndexStrStr = (String, String);

impl TryFrom<FaderIndexStrInt> for FaderIndex {
    type Error = Error;

    fn try_from(value: FaderIndexStrInt) -> Result<Self, Self::Error> {
        if let Ok(index) = usize::try_from(value.1) {
            match value.0.as_str() {
                _ if index == 0 => Err(Error::X32(X32Error::InvalidFader)),
                "mtx" if index <= 6 => Ok(Self::Matrix(index)),
                "auxin" if index <= 8 => Ok(Self::Aux(index)),
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

/* */
/// Internal fader tracking
#[derive(Debug, Clone, PartialEq, PartialOrd)]
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

    /// Get the vor update message for this fader
    #[must_use]
    pub fn vor_message(&self) -> super::osc::Message {
        super::osc::Message::new_string(
            &self.source.get_vor_address(),
            &format!("[{:02}] {:>3} {:>8} {}",
                self.source.get_index(),
                self.is_on().1,
                self.level().1,
                self.name()
            )
        )
    }

    /// update fader from OSC data
    pub fn update(&mut self, update : super::x32::updates::FaderUpdate) {
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
    #[inline]
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


/// Full tracked fader banks
#[derive(Debug, Clone)]
pub struct FaderBank {
    /// main and mono
    main : [Fader;2],
    /// matrix (6)
    matrix : [Fader;6],
    /// aux in (8)
    aux : [Fader;8],
    /// DCA (8)
    dca : [Fader;8],
    /// mix bus (16)
    bus : [Fader;16],
    /// channels (32)
    channel : [Fader;32],
}


impl FaderBank {
    /// create new fader bank
    #[must_use]
    pub fn new() -> Self {
        Self {
            main    : core::array::from_fn(|i| Fader::new(FaderIndex::Main(i+1))),
            matrix  : core::array::from_fn(|i| Fader::new(FaderIndex::Matrix(i+1))),
            bus     : core::array::from_fn(|i| Fader::new(FaderIndex::Bus(i+1))),
            channel : core::array::from_fn(|i| Fader::new(FaderIndex::Channel(i+1))),
            aux     : core::array::from_fn(|i| Fader::new(FaderIndex::Aux(i+1))),
            dca     : core::array::from_fn(|i| Fader::new(FaderIndex::Dca(i+1))),
        }
    }

    /// Reset faders
    pub fn reset(&mut self) {
        let update = crate::x32::updates::FaderUpdate {
            label: Some(String::new()),
            level: Some(0_f32),
            is_on: Some(false),
            ..Default::default() };

        self.main.iter_mut().for_each(|f| f.update(update.clone()));
        self.aux.iter_mut().for_each(|f| f.update(update.clone()));
        self.bus.iter_mut().for_each(|f| f.update(update.clone()));
        self.dca.iter_mut().for_each(|f| f.update(update.clone()));
        self.channel.iter_mut().for_each(|f| f.update(update.clone()));
        self.matrix.iter_mut().for_each(|f| f.update(update.clone()));
    }

    /// Update a fader
    pub fn update(&mut self, update : crate::x32::updates::FaderUpdate) {
        if let Some(fader) = self.get_mut(&update.source) {
            fader.update(update);
        }
    }

    /// Get a mutable fader, zero based index
    pub fn get_mut(&mut self, f_type: &FaderIndex) -> Option<&mut Fader> {
        let index = f_type.get_index() - 1;
        match f_type {
            FaderIndex::Aux(_) => self.aux.get_mut(index),
            FaderIndex::Matrix(_) => self.matrix.get_mut(index),
            FaderIndex::Main(_) => self.main.get_mut(index),
            FaderIndex::Channel(_) => self.channel.get_mut(index),
            FaderIndex::Dca(_) => self.dca.get_mut(index),
            FaderIndex::Bus(_) => self.bus.get_mut(index),
            FaderIndex::Unknown => None,
        }
    }

    /// Get a fader, zero based index
    #[must_use]
    pub fn get(&self, f_type: &FaderIndex) -> Option<Fader> {
        let index = f_type.get_index() - 1;
        match f_type {
            FaderIndex::Aux(_) => self.aux.get(index).cloned(),
            FaderIndex::Matrix(_) => self.matrix.get(index).cloned(),
            FaderIndex::Main(_) => self.main.get(index).cloned(),
            FaderIndex::Channel(_) => self.channel.get(index).cloned(),
            FaderIndex::Dca(_) => self.dca.get(index).cloned(),
            FaderIndex::Bus(_) => self.bus.get(index).cloned(),
            FaderIndex::Unknown => None,
        }
    }
}

impl Default for FaderBank {
    fn default() -> Self { Self::new() }
}
