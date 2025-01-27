use std::{fmt, time::{Duration, SystemTime, UNIX_EPOCH}};

use super::{Buffer, TypeError, BufferError};

// MARK: OSCType
/// OSC Basic Types
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Type {
    /// 32-bit BE integer (i:0x69)
    Integer(i32),
    /// 64-bit BE integer (h:0x68)
    LongInteger(i64),
    /// 32-bit BE floating point (f:0x66)
    Float(f32),
    /// 64-bit BE floating point (d:0x64)
    Double(f64),
    /// 4-byte padded string (s:0x73)
    String(String),
    /// Type list, sent as (,:0x2c) prefixed string
    TypeList(Vec<char>),
    /// Bool (T:0x54, F:0x46) (empty)
    Boolean(bool),
    /// Null (N:0x49) (empty)
    Null(),
    /// Bang (I:0x4e) (empty)
    Bang(),
    /// Color type (r:0x72)
    Color([u8;4]),
    /// Character type (c:0x63)
    Char(char),
    /// Time Tag
    TimeTag(TimeTag),
    /// Generic error type when others fail
    Error(TypeError),
    /// Blob type
    Blob(Vec<u8>),
}

impl Default for Type {
    fn default() -> Self { Type::Error(TypeError::UnknownType) }
}
impl Default for &Type {
    fn default() -> Self { &Type::Error(TypeError::UnknownType) }
}

/// generate `From<T>` and `TryInto<T>` for `Type`
macro_rules! value_impl {
    ($(($variant:ident, $ty:ty)),*) => {
        $(
        impl From<$ty> for Type {
            fn from(v: $ty) -> Self {
                Type::$variant(v)
            }
        }
        impl TryInto<$ty> for Type {
            type Error = TypeError;

            fn try_into(self) -> Result<$ty, TypeError> {
                match self {
                    Type::$variant(v) => Ok(v),
                    _ => Err(TypeError::InvalidTypeConversion)
                }
            }
        }
        )*
    }
}

value_impl! {
    (Integer, i32),
    (LongInteger, i64),
    (Float, f32),
    (Double, f64),
    (String, String),
    (TypeList, Vec<char>),
    (Boolean, bool),
    (Char, char),
    (Color, [u8;4]),
    (TimeTag, TimeTag),
    (Error, TypeError)
}

// MARK: Types->Buffer
#[expect(clippy::from_over_into)]
impl Into<Buffer> for Type {
    fn into(self) -> Buffer { Buffer::from(<Type as Into<Vec<u8>>>::into(self)) }
}

/// Pad a string buffer (`Vec<u8>`)
fn padded_string_buffer(v: &String) -> Vec<u8> {
    let mut buffer = v.as_bytes().to_vec();
    let len_act = buffer.len();
    let len_pad = 4 - (len_act % 4);
    buffer.resize(len_act + len_pad, 0_u8);
    buffer
}

/// Pad a string "string••\[8\]"
fn padded_string(v: &String) -> String {
    let mut buffer = v.as_bytes().to_vec();
    let len_act = buffer.len();
    let len_pad = 4 - (len_act % 4);
    let len_tot = len_act + len_pad;
    buffer.resize(len_tot, 0_u8);
    let mut string = String::from_utf8(buffer).expect("invalid string").replace(char::from(0), "•");
    string.push_str(&format!("[{len_tot}]"));
    string
}

// MARK: Types -> Vec<u8>
#[expect(clippy::from_over_into)]
impl Into<Vec<u8>> for Type {
    fn into(self) -> Vec<u8> {
        match self {
            Type::Error(_) | Type::Bang() | Type::Null() | Type::Boolean(_) => vec![],

            Type::Integer(v)     => v.to_be_bytes().to_vec(),
            Type::LongInteger(v) => v.to_be_bytes().to_vec(),
            Type::Float(v)       => v.to_be_bytes().to_vec(),
            Type::Double(v)      => v.to_be_bytes().to_vec(),

            Type::Color(v) => v.to_vec(),
            Type::Char(v) => (v as u32).to_be_bytes().to_vec(),
            Type::String(v) => padded_string_buffer(&v),
            Type::TimeTag(v) => v.into(),
            Type::TypeList(v) => {
                if v.is_empty() {
                    vec![]
                } else {
                    padded_string_buffer(&format!(",{}", v.into_iter().collect::<String>()))
                }
            },
            Type::Blob(v) => {
                let mut buffer = vec![];
                #[expect(clippy::cast_possible_truncation)]
                #[expect(clippy::cast_possible_wrap)]
                let size = v.len() as i32;

                buffer.extend(size.to_be_bytes().to_vec());
                buffer.extend(v);

                let len_act = buffer.len();
                let len_tot = if len_act % 4 == 0 { len_act} else { len_act + (4 - (len_act % 4)) };
                buffer.resize(len_tot, 0_u8);

                buffer
            }
        }
    }
}

// MARK: Types -> String
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let type_flag= self.get_type_char().unwrap_or('*');

        let type_string:String = match &self {
            Type::Float(v) => v.to_string(),
            Type::Double(v) => v.to_string(),

            Type::Integer(v) => v.to_string(),
            Type::LongInteger(v) => v.to_string(),

            Type::Char(v) => v.to_string(),
            Type::Color(v) => format!("[{}, {}, {}, {}]", v[0], v[1], v[2], v[3]),

            Type::Bang() | Type::Null() | Type::Boolean(_) => String::new(),
            
            Type::TimeTag(v) => format!("[{}, {}]", v.seconds, v.fractional),
            Type::Error(v) => v.to_string(),

            Type::String(v)=> padded_string(v),

            Type::TypeList(v) => {
                if v.is_empty() {
                    String::new()
                } else {
                    padded_string(&format!(",{}", String::from_iter(v)))
                }
            },
            Type::Blob(v) => format!("[~b:{}~]", v.len())
        };

        write!(f, "|{type_flag}:{type_string}|")
    }
}

// MARK:([u8],ch) -> Types
impl From<(&[u8], char)> for Type {
    fn from(value: (&[u8], char)) -> Self {
        if value.0.len() % 4 != 0 { return TypeError::MisalignedBuffer.into() }
        match (value.1, value.0.len()) {
            ('T', 0) => true.into(),
            ('F', 0) => false.into(),
            ('N', 0) => Type::Null(),
            ('I', 0) => Type::Bang(),
            (',', 0) => Type::TypeList(vec![]),

            (_, 0) => TypeError::MisalignedBuffer.into(),

            ('i', 4) => {
                let v = &value.0[0..4].try_into().expect("invalid buffer");
                i32::from_be_bytes(*v).into()
            },

            ('f', 4) => {
                let v = &value.0[0..4].try_into().expect("invalid buffer");
                f32::from_be_bytes(*v).into()
            },

            ('h', 8) => {
                let v = &value.0[0..8].try_into().expect("invalid buffer");
                i64::from_be_bytes(*v).into()
            },

            ('d', 8) => {
                let v = &value.0[0..8].try_into().expect("invalid buffer");
                f64::from_be_bytes(*v).into()
            },

            ('i' | 'f' | 'h' | 'd', _) => TypeError::MisalignedNumberBuffer.into(),
            
            ('t', 8) => {
                let s = &value.0[0..4].try_into().expect("invalid buffer");
                let f = &value.0[4..8].try_into().expect("invalid buffer");
                let time_tag:TimeTag = (s, f).into();
                time_tag.into()
            }
            ('c', 4) => {
                let v = &value.0[0..4].try_into().expect("invalid buffer");
                match char::from_u32(u32::from_be_bytes(*v)) {
                    Some(v) => v.into(),
                    None => TypeError::ConvertFromString.into()
                }
            }

            ('r', 4) => {
                let v:[u8;4] = value.0[0..4].try_into().expect("invalid buffer");
                v.into()
            }
            ('c' | 'r' | 't', _) => TypeError::MisalignedBuffer.into(),

            ('s', _,) => {
                match std::str::from_utf8(value.0) {
                    Ok(v) => v.trim_end_matches(char::from(0)).to_owned().into(),
                    Err(_) => TypeError::ConvertFromString.into()
                }
            },

            (',', _) => {
                let mut type_list:Vec<char> = vec![];
                for i in &value.0[1..] {
                    if i != &0_u8 { type_list.push(*i as char); }
                }
                type_list.into()
            }

            ('b', _) => {
                let v:&[u8;4] = value.0[0..4].try_into().expect("invalid buffer");
                
                #[expect(clippy::cast_sign_loss)]
                let real_size = i32::from_be_bytes(*v) as usize;
                let end_idx = real_size + 4;

                if value.0.len() >= end_idx {
                    Type::Blob(value.0[4..end_idx].to_vec())
                } else {
                    TypeError::MisalignedBuffer.into()
                }
            }

            _ => TypeError::InvalidTypeFlag.into()
        }
    }
}

// MARK: Types impl
impl Type {
    /// is error type? (bool)
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(&self, Type::Error(_))
    }

    /// Decode a buffer into an `Option`
    /// 
    /// # Errors
    /// fails on invalid packets or unknown type or invalid type conversion
    pub fn decode_buffer(item : Result<Vec<u8>, BufferError>, type_flag : char ) -> Result<Self, TypeError> {
        match item {
            Err(_) => Err(TypeError::InvalidPacket),
            Ok(item) => {
                match (item.as_slice(), type_flag).into() {
                    Type::Error(v) => Err(v),
                    v => Ok(v)
                }
            }
        }
    }

    /// create new `OSCType` from `[u8]` of the passed type
    #[must_use]
    pub fn from_type(item: &Vec<u8>, type_flag:char) -> Self {
        (item.as_slice(), type_flag).into()
    }

    /// get character type association
    ///
    /// # Errors
    /// fails on invalid type 
    pub fn get_type_char(&self) -> Result<char, TypeError> {
        match &self {
            Type::Bang()         => Ok('I'),
            Type::Blob(_)        => Ok('b'),
            Type::Char(_)        => Ok('c'),
            Type::Color(_)       => Ok('r'),
            Type::Double(_)      => Ok('d'),
            Type::Float(_)       => Ok('f'),
            Type::Integer(_)     => Ok('i'),
            Type::LongInteger(_) => Ok('h'),
            Type::Null()         => Ok('N'),
            Type::String(_)      => Ok('s'),
            Type::TimeTag(_)     => Ok('t'),
            Type::TypeList(_)    => Ok(','),
            Type::Boolean(v) => if *v { Ok('T') } else { Ok('F') },
            Type::Error(v) => Err(v.clone()),
        }
    }
}




// MARK: OSCTimeTag
/// OSC Time tag structure
#[derive(Debug, PartialEq, PartialOrd, Clone, Default, Eq, Ord)]
pub struct TimeTag {
    /// seconds since epoch
    seconds: u32,
    /// fractional seconds
    fractional : u32
}

// MARK:(u32*2)->TimeTag
impl From<(u32, u32)> for TimeTag {
    fn from(value: (u32, u32)) -> Self {
        TimeTag { seconds: value.0, fractional: value.1 }
    }
}

// MARK:([u8;4]*2)->TimeTag
impl From<(&[u8;4], &[u8;4])> for TimeTag {
    fn from(value: (&[u8;4], &[u8;4])) -> Self {
        (
            u32::from_be_bytes(*value.0),
            u32::from_be_bytes(*value.1)
        ).into()
    }
}

// MARK: TimeTag->Vec<u8>
impl From<TimeTag> for Vec<u8> {
    fn from(v: TimeTag) -> Self {
        let mut buffer = v.seconds.to_be_bytes().to_vec();

        buffer.extend(v.fractional.to_be_bytes());
        buffer
    }
}

//  MARK: TimeTag impl
impl TimeTag {
    /// get a now time tag
    #[must_use]
    pub fn now() -> Self {
        SystemTime::now().try_into().unwrap_or_default()
    }

    /// get a future time tag (now + ms)
    #[must_use]
    pub fn future(ms : u64) -> Self {
        let now = SystemTime::now();
        let adder = Duration::from_millis(ms);

        match now.checked_add(adder) {
            Some(v) => v.try_into().unwrap_or_default(),
            None => TimeTag::default()
        }
    }

    /// From RFC 5905
    const UNIX_OFFSET: u64 = 2_208_988_800;
    /// Number of bits in a `u32`
    const TWO_POW_32: f64 = (u32::MAX as f64) + 1.0;
    /// One over the number of bits
    const ONE_OVER_TWO_POW_32: f64 = 1.0 / TimeTag::TWO_POW_32;
    /// Nanoseconds in a second
    const NANO_SEC_PER_SECOND: f64 = 1.0e9;
    /// Seconds in a nanosecond (fractional)
    const SECONDS_PER_NANO: f64 = 1.0 / TimeTag::NANO_SEC_PER_SECOND;
}

// MARK: SysTime -> Types
impl TryFrom<SystemTime> for Type {
    type Error = TypeError;

    fn try_from(value: SystemTime) -> Result<Self, TypeError> {
        match TimeTag::try_from(value) {
            Ok(v) => Ok(v.into()),
            Err(v) => Err(v)
        }
    }
}

// MARK: SysTime -> TimeTag
impl TryFrom<SystemTime> for TimeTag {
    type Error = TypeError;

    fn try_from(time: SystemTime) -> Result<Self, TypeError> {
        let duration_since_epoch = time
            .duration_since(UNIX_EPOCH)
            .map_err(|_| TypeError::InvalidTimeUnderflow)?
            + Duration::new(TimeTag::UNIX_OFFSET, 0);

        let seconds = u32::try_from(duration_since_epoch.as_secs())
            .map_err(|_| TypeError::InvalidTimeOverflow)?;

        #[expect(clippy::cast_lossless)]
        let nano_sec = duration_since_epoch.subsec_nanos() as f64;

        #[expect(clippy::cast_possible_truncation)]
        #[expect(clippy::cast_sign_loss)]
        let fractional = (nano_sec * TimeTag::SECONDS_PER_NANO * TimeTag::TWO_POW_32).round() as u32;

        Ok((seconds, fractional).into())
    }
}

// MARK : Types -> SysTime
impl TryFrom<Type> for SystemTime {
    type Error = TypeError;

    fn try_from(value: Type) -> Result<Self, TypeError> {
        match value {
            Type::TimeTag(v) => Ok(v.into()),
            _ => Err(TypeError::InvalidTypeConversion)
        }
    }
}

// MARK : TimeTag -> SysTime
impl From<TimeTag> for SystemTime {
    fn from(time: TimeTag) -> Self {
        let nano_secs =
            f64::from(time.fractional) * TimeTag::ONE_OVER_TWO_POW_32 * TimeTag::NANO_SEC_PER_SECOND;

        #[expect(clippy::cast_possible_truncation)]
        #[expect(clippy::cast_sign_loss)]
        let duration_since_osc_epoch = Duration::new(u64::from(time.seconds), nano_secs.round() as u32);
        let duration_since_unix_epoch =
            duration_since_osc_epoch - Duration::new(TimeTag::UNIX_OFFSET, 0);
        UNIX_EPOCH + duration_since_unix_epoch
    }
}