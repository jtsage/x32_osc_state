use std::fmt;
use std::sync::LazyLock;
use regex::Regex;
use super::osc;

/// Pull fader level from node string
static LVL_STRING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?<level>[+\-0-9.]+)").expect("unable to compile pattern")
});

/// Split node string on whitespace, skipping quoted items
pub static NODE_STRING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"[^\s"]+|"([^"]*)""#).expect("unable to compile pattern")
});

/// bundle tag, "#bundle", 8-byte
pub const BUNDLE_TAG:[u8;8] = [0x23, 0x62, 0x75, 0x6e, 0x64, 0x6c, 0x65, 0x0];
/// simple ignored node message - "-prefs/name", 24-byte
pub const X32_KEEP_ALIVE:[u8;24] = [0x2f, 0x6e, 0x6f, 0x64, 0x65, 0x0, 0x0, 0x0, 0x2c, 0x73, 0x0, 0x0, 0x2d, 0x70, 0x72, 0x65, 0x66, 0x73, 0x2f, 0x6e, 0x61, 0x6d, 0x65, 0x0];
/// X32 remote command "/xremote", 12-byte
pub const X32_XREMOTE:[u8;12] = [0x2f, 0x78, 0x72, 0x65, 0x6d, 0x6f, 0x74, 0x65, 0x0, 0x0, 0x0, 0x0];

// MARK: Error
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Packet(v) => write!(f, "buffer error: {v}"),
            Self::OSC(v) => write!(f, "osc error: {v}"),
            Self::X32(v) => write!(f, "x32 error: {v}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Packet(v) => Some(v),
            Self::OSC(v) => Some(v),
            Self::X32(v) => Some(v),
        }
    }
}

// MARK: PacketError
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
    InvalidBuffer,
    /// Invalid original message
    InvalidMessage,
    /// Type conversion failed
    InvalidTypesForMessage,
    
}

impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Self::NotFourByte => "not 4-byte aligned",
            Self::UnterminatedString => "string not terminated with 0x0 null",
            Self::Underrun => "buffer not large enough for operation",
            Self::InvalidBuffer => "buffer contains invalid data",
            Self::InvalidMessage => "message conversion invalid",
            Self::InvalidTypesForMessage => "type conversion invalid",
        })
    }
}

impl std::error::Error for PacketError { }

// MARK: OSCError
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

impl fmt::Display for OSCError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Self::ConvertFromString => "string conversion failed",
            Self::AddressContent => "address is not ascii",
            Self::UnknownType => "unknown OSC type",
            Self::InvalidTypeFlag => "unknown OSC type flag",
            Self::InvalidTypeConversion => "type conversion invalid",
            Self::InvalidTimeUnderflow => "time too early to represent",
            Self::InvalidTimeOverflow => "time too late to represent",
        })
    }
}

impl std::error::Error for OSCError { }

// MARK: X32Error
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

impl fmt::Display for X32Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Self::InvalidFader => "invalid fader",
            Self::UnimplementedPacket => "unhandled message",
            Self::MalformedPacket => "packet format invalid - not enough arguments",
        })
    }
}

impl std::error::Error for X32Error { }


// MARK: ShowMode
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
            Self::Aux(v) => format!("auxin/{v:02}"),
            Self::Matrix(v) => format!("mtx/{v:02}"),
            Self::Main(v) => if *v == 2 { String::from("main/m") } else { String::from("main/st") },
            Self::Channel(v) => format!("ch/{v:02}"),
            Self::Dca(v) => format!("dca/{v}"),
            Self::Bus(v) => format!("bus/{v:02}"),
        }
    }

    /// Get the VOR output address for this fader
    #[must_use]
    pub fn get_vor_address(&self) -> String {
        match self {
            Self::Main(v) => format!("/main/{v:02}"),
            _ => format!("/{}", self.get_x32_address()),
        }
    }

    /// Get a vector of OSC messages that will force
    /// the X32 to update this fader
    #[must_use]
    pub fn get_x32_update(&self) -> Vec<osc::Buffer> {
        let address = self.get_x32_address();
        match self {
            Self::Unknown => vec![osc::Buffer::default()],
            Self::Dca(_) => vec![
                osc::Buffer::try_from(osc::Message::new_with_string("/node", &address)).unwrap_or_default(),
                osc::Buffer::try_from(osc::Message::new_with_string("/node", &format!("{address}/config"))).unwrap_or_default(),
            ],
            _ => vec![
                osc::Buffer::try_from(osc::Message::new_with_string("/node", &format!("{address}/mix"))).unwrap_or_default(),
                osc::Buffer::try_from(osc::Message::new_with_string("/node", &format!("{address}/config"))).unwrap_or_default(),
            ],
        }
    }
}

// MARK: FaderIndexParse
/// Fader Index parsers
pub enum FaderIndexParse {
    /// String name, integer index (1-based)
    Integer(String, i32),
    /// String name, string index (1-based)
    String(String, String),
}

impl TryFrom<FaderIndexParse> for FaderIndex {
    type Error = Error;

    fn try_from(value: FaderIndexParse) -> Result<Self, Self::Error> {
        let invalid_fader = Error::X32(X32Error::InvalidFader);

        let index = match &value {
            FaderIndexParse::Integer(_, d) => usize::try_from(*d).map_err(|_| invalid_fader)?,
            FaderIndexParse::String(s, d) => {
                if s.as_str() == "main" {
                    if d.as_str() == "m" { 2 } else { 1 }
                } else {
                    d.parse::<usize>().map_err(|_| invalid_fader)?
                }
            },
        };

        match value {
            FaderIndexParse::Integer(s, _) |
            FaderIndexParse::String(s, _) => {
                match s.as_str() {
                    _ if index == 0 => Err(invalid_fader),
                    "mtx" if index <= 6 => Ok(Self::Matrix(index)),
                    "auxin" if index <= 8 => Ok(Self::Aux(index)),
                    "dca" if index <= 8 => Ok(Self::Dca(index)),
                    "main" if index <= 2 => Ok(Self::Main(index)),
                    "ch" if index <= 32 => Ok(Self::Channel(index)),
                    "bus" if index <= 16 => Ok(Self::Bus(index)),
                    _ => Err(invalid_fader)
                }
            },
        }
    }
}


/// Fader color
#[expect(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum FaderColor {
    Off,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    #[default]
    White,
    RedInverted,
    GreenInverted,
    YellowInverted,
    BlueInverted,
    MagentaInverted,
    CyanInverted,
    WhiteInverted,
}

impl FaderColor {
    /// Get color from index
    #[must_use]
    pub fn parse_int(v: i32) -> Self {
        match v {
            1 => Self::Red,
            2 => Self::Green,
            3 => Self::Yellow,
            4 => Self::Blue,
            5 => Self::Magenta,
            6 => Self::Cyan,
            7 => Self::White,
            9 => Self::RedInverted,
            10 => Self::GreenInverted,
            11 => Self::YellowInverted,
            12 => Self::BlueInverted,
            13 => Self::MagentaInverted,
            14 => Self::CyanInverted,
            15 => Self::WhiteInverted,
            _ => Self::Off,
        }
    }
    /// Read from pre-defined color string
    #[must_use]
    pub fn parse_str(v: &str) -> Self {
        match v {
            "OFF" | "OFFi" => Self::Off, 
            "RD" => Self::Red,
            "GN" => Self::Green,
            "YE" => Self::Yellow,
            "BL" => Self::Blue,
            "MG" => Self::Magenta,
            "CY" => Self::Cyan,
            "RDi" => Self::RedInverted,
            "GNi" => Self::GreenInverted,
            "YEi" => Self::YellowInverted,
            "BLi" => Self::BlueInverted,
            "MGi" => Self::MagentaInverted,
            "CYi" => Self::CyanInverted,
            "WHi" => Self::WhiteInverted,
            _ => Self::White,
        }
    }
}

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
    is_on : bool,
    /// Fader color
    color : FaderColor,
}


impl Fader {
    /// create new fader
    #[must_use]
    pub fn new(source : FaderIndex) -> Self {
        Self {
            source,
            color : FaderColor::default(),
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

    /// Get color
    #[must_use]
    pub fn color(&self) -> FaderColor {
        self.color
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
    pub fn vor_message(&self) -> super::osc::Packet {
        super::osc::Packet::Message(super::osc::Message::new_with_string(
            &self.source.get_vor_address(),
            &format!("[{:02}] {:>3} {:>8} {}",
                self.source.get_index(),
                self.is_on().1,
                self.level().1,
                self.name()
            )
        ))
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

        if let Some(new_color) = update.color {
            self.color = new_color;
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

/// Keys to the fader banks
pub enum FaderBankKey {
    /// main (2)
    Main,
    /// matrix (6)
    Matrix,
    /// aux (8)
    Aux,
    /// bus (16)
    Bus,
    /// DCA (8)
    Dca,
    /// Channel (32)
    Channel
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

    /// Get vor messages for an entire bank
    pub fn vor_bundle(&self, key : &FaderBankKey) -> Vec<super::osc::Packet> {
        let a = match key {
            FaderBankKey::Main => self.main.to_vec(),
            FaderBankKey::Matrix => self.matrix.to_vec(),
            FaderBankKey::Aux => self.aux.to_vec(),
            FaderBankKey::Bus => self.bus.to_vec(),
            FaderBankKey::Dca => self.dca.to_vec(),
            FaderBankKey::Channel => self.channel.to_vec(),
        };

        a.iter().map(Fader::vor_message).collect()
    }

    /// Reset faders
    pub fn reset(&mut self) {
        let update = crate::x32::updates::FaderUpdate {
            label: Some(String::new()),
            level: Some(0_f32),
            is_on: Some(false),
            color: Some(FaderColor::White),
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
