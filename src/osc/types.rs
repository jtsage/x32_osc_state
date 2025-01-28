use std::{fmt, time::{Duration, SystemTime, UNIX_EPOCH}};

use super::super::enums;
use super::Buffer;

// MARK: OSCType
/// OSC Basic Types
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Type {
    /// 4-byte padded string (s:0x73)
    String(String),
    /// Type list, sent as (,:0x2c) prefixed string
    TypeList(Vec<char>),
    /// 32-bit BE integer (i:0x69)
    Integer(i32),
    /// Time Tag
    TimeTag(TimeTag),
    /// 64-bit BE integer (h:0x68)
    LongInteger(i64),
    /// 32-bit BE floating point (f:0x66)
    Float(f32),
    /// 64-bit BE floating point (d:0x64)
    Double(f64),
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
    /// Blob type
    Blob(Vec<u8>),
    /// Generic error type when others fail
    Unknown()
}

impl Default for Type {
    fn default() -> Self { Self::Unknown() }
}
impl Default for &Type {
    fn default() -> Self { &Type::Unknown() }
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
            type Error = enums::Error;

            fn try_into(self) -> Result<$ty, Self::Error> {
                match self {
                    Type::$variant(v) => Ok(v),
                    _ => Err(enums::Error::OSC(enums::OSCError::InvalidTypeConversion))
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
    (TimeTag, TimeTag)
}

// MARK: Types->Buffer
#[expect(clippy::from_over_into)]
impl Into<Buffer> for Type {
    fn into(self) -> Buffer { Buffer::from(<Self as Into<Vec<u8>>>::into(self)) }
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
    let buffer = padded_string_buffer(v);
    let len_tot = buffer.len();
    let mut string = String::from_utf8(buffer).expect("invalid string").replace(char::from(0), "•");
    string.push_str(&format!("[{len_tot}]"));
    string
}

// MARK: Types -> Vec<u8>
#[expect(clippy::from_over_into)]
impl Into<Vec<u8>> for Type {
    fn into(self) -> Vec<u8> {
        match self {
            Self::Integer(v)     => v.to_be_bytes().to_vec(),
            Self::LongInteger(v) => v.to_be_bytes().to_vec(),
            Self::Float(v)       => v.to_be_bytes().to_vec(),
            Self::Double(v)      => v.to_be_bytes().to_vec(),

            Self::Color(v) => v.to_vec(),
            Self::Char(v) => (v as u32).to_be_bytes().to_vec(),
            Self::String(v) => padded_string_buffer(&v),
            Self::TimeTag(v) => v.into(),
            Self::TypeList(v) => {
                if v.is_empty() {
                    vec![]
                } else {
                    padded_string_buffer(&format!(",{}", v.into_iter().collect::<String>()))
                }
            },
            Self::Blob(v) => {
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
            },
            _ => vec![],
        }
    }
}

// MARK: Types -> String
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let type_flag= self.get_type_char().unwrap_or('*');

        let type_string:String = match &self {
            Self::Float(v) => v.to_string(),
            Self::Double(v) => v.to_string(),

            Self::Integer(v) => v.to_string(),
            Self::LongInteger(v) => v.to_string(),

            Self::Char(v) => v.to_string(),
            Self::Color(v) => format!("[{}, {}, {}, {}]", v[0], v[1], v[2], v[3]),

            Self::Bang() | Self::Null() | Self::Boolean(_) | Self::Unknown() => String::new(),
            
            Self::TimeTag(v) => format!("[{}, {}]", v.seconds, v.fractional),
            Self::String(v)=> padded_string(v),

            Self::TypeList(v) => {
                if v.is_empty() {
                    String::new()
                } else {
                    padded_string(&format!(",{}", String::from_iter(v)))
                }
            },
            Self::Blob(v) => format!("[~b:{}~]", v.len())
        };

        write!(f, "|{type_flag}:{type_string}|")
    }
}

// MARK:([u8],ch) -> Types
impl TryFrom<(&[u8], char)> for Type {
    type Error = enums::Error;
    
    fn try_from(value: (&[u8], char)) -> Result<Self, Self::Error> {
        if value.0.len() % 4 != 0 { return Err(enums::Error::Packet(enums::PacketError::NotFourByte)) }
        match (value.1, value.0.len()) {
            ('T', 0) => Ok(true.into()),
            ('F', 0) => Ok(false.into()),
            ('N', 0) => Ok(Self::Null()),
            ('I', 0) => Ok(Self::Bang()),
            (',', 0) => Ok(Self::TypeList(vec![])),

            ('i', 4) => {
                let v = &value.0[0..4].try_into().map_err(|_| enums::Error::Packet(enums::PacketError::Underrun))?;
                Ok(i32::from_be_bytes(*v).into())
            },

            ('f', 4) => {
                let v = &value.0[0..4].try_into().map_err(|_| enums::Error::Packet(enums::PacketError::Underrun))?;
                Ok(f32::from_be_bytes(*v).into())
            },

            ('h', 8) => {
                let v = &value.0[0..8].try_into().map_err(|_| enums::Error::Packet(enums::PacketError::Underrun))?;
                Ok(i64::from_be_bytes(*v).into())
            },

            ('d', 8) => {
                let v = &value.0[0..8].try_into().map_err(|_| enums::Error::Packet(enums::PacketError::Underrun))?;
                Ok(f64::from_be_bytes(*v).into())
            },

            ('t', 8) => {
                let s = &value.0[0..4].try_into().map_err(|_| enums::Error::Packet(enums::PacketError::Underrun))?;
                let f = &value.0[4..8].try_into().map_err(|_| enums::Error::Packet(enums::PacketError::Underrun))?;
                let time_tag:TimeTag = (s, f).into();
                Ok(time_tag.into())
            }
            ('c', 4) => {
                let v = &value.0[0..4].try_into().map_err(|_| enums::Error::Packet(enums::PacketError::Underrun))?;
                char::from_u32(u32::from_be_bytes(*v)).map_or(Err(enums::Error::OSC(enums::OSCError::ConvertFromString)), |v| Ok(v.into()))
            }

            ('r', 4) => {
                let v:[u8;4] = value.0[0..4].try_into().map_err(|_| enums::Error::Packet(enums::PacketError::Underrun))?;
                Ok(v.into())
            }

            ('i' | 'f' | 'h' | 'd' | 'c' | 'r' | 't', _) | (_, 0) => Err(enums::Error::Packet(enums::PacketError::Underrun)),

            ('s', _,) => {
                let v = std::str::from_utf8(value.0).map_err(|_| enums::Error::OSC(enums::OSCError::ConvertFromString))?;
                Ok(v.trim_end_matches(char::from(0)).to_owned().into())
            },

            (',', _) => {
                let mut type_list:Vec<char> = vec![];
                for i in &value.0[1..] {
                    if i != &0_u8 { type_list.push(*i as char); }
                }
                Ok(type_list.into())
            }

            ('b', _) => {
                let v:&[u8;4] = value.0[0..4].try_into().map_err(|_| enums::Error::Packet(enums::PacketError::Underrun))?;
                
                #[expect(clippy::cast_sign_loss)]
                let real_size = i32::from_be_bytes(*v) as usize;
                let end_idx = real_size + 4;

                if value.0.len() >= end_idx {
                    Ok(Self::Blob(value.0[4..end_idx].to_vec()))
                } else {
                    Err(enums::Error::Packet(enums::PacketError::Underrun))
                }
            }

            _ => Err(enums::Error::OSC(enums::OSCError::InvalidTypeFlag))
        }
    }
}

// MARK: Types impl
impl Type {
    /// is error type? (bool)
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(&self, Self::Unknown())
    }

    /// Decode a buffer into an `Option`
    /// 
    /// # Errors
    /// fails on invalid packets or unknown type or invalid type conversion
    #[inline]
    pub fn decode_buffer(item : Result<Vec<u8>, enums::Error>, type_flag : char ) -> Result<Self, enums::Error> {
        match item {
            Err(v) => Err(v),
            Ok(item) => (item.as_slice(), type_flag).try_into()
        }
    }

    /// create new `OSCType` from `[u8]` of the passed type
    ///
    /// # Errors
    /// fails on invalid packets or unknown type or invalid type conversion
    #[inline]
    pub fn try_from_type(item: &Vec<u8>, type_flag:char) -> Result<Self, enums::Error> {
        (item.as_slice(), type_flag).try_into()
    }

    /// get character type association
    ///
    /// # Errors
    /// fails on invalid type 
    pub fn get_type_char(&self) -> Result<char, enums::Error> {
        match &self {
            Self::String(_)      => Ok('s'),
            Self::Integer(_)     => Ok('i'),
            Self::TypeList(_)    => Ok(','),
            Self::TimeTag(_)     => Ok('t'),

            Self::Bang()         => Ok('I'),
            Self::Blob(_)        => Ok('b'),
            Self::Char(_)        => Ok('c'),
            Self::Color(_)       => Ok('r'),
            Self::Double(_)      => Ok('d'),
            Self::Float(_)       => Ok('f'),
            Self::LongInteger(_) => Ok('h'),
            Self::Null()         => Ok('N'),
            Self::Boolean(v) => if *v { Ok('T') } else { Ok('F') },
            Self::Unknown() => Err(enums::Error::OSC(enums::OSCError::UnknownType)),
        }
    }
}




// MARK: OSCTimeTag
/// OSC Time tag structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct TimeTag {
    /// seconds since epoch
    seconds: u32,
    /// fractional seconds
    fractional : u32
}

// MARK:(u32*2)->TimeTag
impl From<(u32, u32)> for TimeTag {
    fn from(value: (u32, u32)) -> Self {
        Self { seconds: value.0, fractional: value.1 }
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
    #[inline]
    #[must_use]
    pub fn now() -> Self {
        SystemTime::now().try_into().unwrap_or_default()
    }

    /// get a future time tag (now + ms)
    #[expect(clippy::single_call_fn)]
    #[inline]
    #[must_use]
    pub fn future(ms : u64) -> Self {
        let now = SystemTime::now();
        let adder = Duration::from_millis(ms);

        now.checked_add(adder).map_or_else(Self::default, |v| v.try_into().unwrap_or_default())
    }

    /// From RFC 5905
    const UNIX_OFFSET: u64 = 2_208_988_800;
    /// Number of bits in a `u32`
    const TWO_POW_32: f64 = (u32::MAX as f64) + 1.0;
    /// One over the number of bits
    const ONE_OVER_TWO_POW_32: f64 = 1.0 / Self::TWO_POW_32;
    /// Nanoseconds in a second
    const NANO_SEC_PER_SECOND: f64 = 1.0e9;
    /// Seconds in a nanosecond (fractional)
    const SECONDS_PER_NANO: f64 = 1.0 / Self::NANO_SEC_PER_SECOND;
}

// MARK: SysTime -> Types
impl TryFrom<SystemTime> for Type {
    type Error = enums::Error;

    fn try_from(value: SystemTime) -> Result<Self, Self::Error> {
        match TimeTag::try_from(value) {
            Ok(v) => Ok(v.into()),
            Err(v) => Err(v)
        }
    }
}

// MARK: SysTime -> TimeTag
impl TryFrom<SystemTime> for TimeTag {
    type Error = enums::Error;

    fn try_from(time: SystemTime) -> Result<Self, Self::Error> {
        let duration_since_epoch = time
            .duration_since(UNIX_EPOCH)
            .map_err(|_| enums::Error::OSC(enums::OSCError::InvalidTimeUnderflow))?
            + Duration::new(Self::UNIX_OFFSET, 0);

        let seconds = u32::try_from(duration_since_epoch.as_secs())
            .map_err(|_| enums::Error::OSC(enums::OSCError::InvalidTimeOverflow))?;

        #[expect(clippy::cast_lossless)]
        let nano_sec = duration_since_epoch.subsec_nanos() as f64;

        #[expect(clippy::cast_possible_truncation)]
        #[expect(clippy::cast_sign_loss)]
        let fractional = (nano_sec * Self::SECONDS_PER_NANO * Self::TWO_POW_32).round() as u32;

        Ok((seconds, fractional).into())
    }
}

// MARK : Types -> SysTime
impl TryFrom<Type> for SystemTime {
    type Error = enums::Error;

    fn try_from(value: Type) -> Result<Self, enums::Error> {
        match value {
            Type::TimeTag(v) => Ok(v.into()),
            _ => Err(enums::Error::OSC(enums::OSCError::InvalidTypeConversion))
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

#[cfg(test)]
mod time_tag_test {
    use super::TimeTag;
    use std::time::SystemTime;
    
    #[test]
    fn time_future_test() {
        let now = TimeTag::now();
        let future = TimeTag::future(5000);

        let bad_future = TimeTag::future(u64::MAX);
        assert_eq!(bad_future, TimeTag::default());

        let now_sys:SystemTime = now.into();
        let future_sys:SystemTime = future.into();

        let duration = future_sys.duration_since(now_sys).expect("clock drift");
        let seconds = duration.as_secs_f64();

        assert!(seconds > 4.0 && seconds < 6.0);
    }
}