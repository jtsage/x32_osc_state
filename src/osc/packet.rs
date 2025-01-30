/// OSC Packet definitions - messages and bundles, and `OSCData` container
use std::fmt;

use super::super::enums;
use super::types::TimeTag;
use super::types::Type;
use super::Buffer;


// MARK: Message
/// OSC Single Message
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Message {
    /// Address bit
    pub address : String,
    /// Arguments vector
    pub args : Vec<Type>,
    /// Force empty argument list output
    pub force_empty_args : bool,
}

// MARK: Bundle
/// OSC Bundle
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Bundle {
    /// Time tag for message
    pub time : TimeTag,
    /// Messages vector
    pub messages : Vec<Packet>
}

// MARK: Packet
/// OSC Data Enum
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Packet {
    /// Message Type
    Message(Message),
    /// Bundle type (can be nested)
    Bundle(Bundle)
}

impl From<Message> for Packet {
    fn from(v: Message) -> Self { Self::Message(v) }
}

impl From<Bundle> for Packet {
    fn from(v: Bundle) -> Self { Self::Bundle(v) }
}

// MARK: Bundle impl
impl Bundle {
    /// Make a new bundle
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            time : TimeTag::now(),
            messages : vec![]
        }
    }

    /// Make a new bundle with pre-built messages
    #[must_use]
    pub fn new_with_messages<T: Into<Packet>>(msgs: Vec<T>) -> Self {
        let mut messages:Vec<Packet> = vec![];
        for v in msgs { messages.push(v.into()); }
        Self {
            time : TimeTag::now(),
            messages
        }
    }

    /// Make a new future bundle (add "ms" to now)
    #[must_use]
    #[inline]
    pub fn new_with_future(ms : u64) -> Self {
        Self {
            time : TimeTag::future(ms),
            messages : vec![]
        }
    }

    /// Add message or nested bundle to bundle
    pub fn add<T: Into<Packet>>(&mut self, v : T) {
        let v = v.into();
        self.messages.push(v);
    }
}

impl Default for Bundle {
    fn default() -> Self { Self::new() }
}

// MARK: Message impl
impl Message {
    /// New message, relaxed addressing
    #[must_use]
    pub fn new(address: &str) -> Self {
        Self {
            address : address.to_owned(),
            args : vec![],
            force_empty_args : false
        }
    }

    /// Create a new message with a single string argument
    #[must_use]
    pub fn new_with_string(address: &str, data: &str) -> Self {
        Self {
            address : address.to_owned(),
            args : vec![Type::String(data.to_owned())],
            force_empty_args : false
        }
    }

    /// Get the first argument, with a sane default
    /// Note that type is determined by the type of the default
    pub fn first_default<T>(&self, default: T) -> T  where 
        T: TryFrom<Type>
    {
        if let Some(a) = self.args.first() {
            a.clone().default_value(default)
        } else {
            default
        }
    }

    /// Boolean is message valid
    #[must_use]
    pub fn is_valid(&self) -> bool {
        if self.address.is_ascii() && !self.address.is_empty() {
            !self.args.clone().iter().any(|s| matches!(s, Type::Unknown()))
        } else {
            false
        }
    }

    /// Add a known type to the message
    pub fn add_item<T>(&mut self, item : T) -> &mut Self where
        Type: std::convert::From<T>
    {
        self.args.push(Type::from(item));
        self
    }

    /// Get the type list as an `OSCType(TypeList)`
    fn type_list(&self) -> Type {
        let list:Vec<char> = self.args
            .clone()
            .into_iter()
            .filter_map(|x| x.as_type_char().ok())
            .collect();
        
        list.into()
    }
}

// MARK: Message->String
impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Type::String(self.address.clone()))?;

        if self.force_empty_args && self.args.is_empty() {
            write!(f, "|,:,•••[4]|")?;
        } else {
            write!(f, "{}", &self.type_list())?;
        }

        write!(f, "{}", String::from_iter(self.args.clone()))
    }
}

// MARK: Message->Buffer
impl TryFrom<Message> for Buffer {
    type Error = enums::Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        if !value.is_valid() { return Err(enums::Error::Packet(enums::PacketError::InvalidMessage)); }

        let mut osc_buffer = <Type as Into<Self>>::into(Type::String(value.address.clone()));//.into();

        if value.force_empty_args && value.args.is_empty() {
            osc_buffer.extend(&Self::from(vec![0x2c, 0x0, 0x0, 0x0]));
        } else {
            osc_buffer.extend(&<Type as Into<Self>>::into(value.type_list()));
        }
        osc_buffer.extend(&value.args.clone().into_iter().collect());

        Ok(osc_buffer)
    }
}

// MARK: Buffer->Message
impl TryFrom<Buffer> for Message {
    type Error = enums::Error;

    fn try_from(mut data: Buffer) -> Result<Self, Self::Error> {
        if !data.is_valid() {
            Err(enums::Error::Packet(enums::PacketError::NotFourByte))
        } else if let Ok(Type::String(osc_address)) = Type::try_from_buffer(data.next_string(), 's') {
            let mut force_empty_args = false;
            let mut osc_payload:Vec<Type> = vec![];

            if let Ok(Type::TypeList(osc_types)) = Type::try_from_buffer(data.next_string(), ',') {
                if osc_types.is_empty() { force_empty_args = true }

                let type_input_length= osc_types.len();

                osc_payload = osc_types.into_iter().filter_map(|type_flag| match type_flag {
                    'i' | 'f' | 'c' | 'r' => Type::try_from_buffer(data.next_bytes(4), type_flag),
                    'h' | 'd' | 't' => Type::try_from_buffer(data.next_bytes(8), type_flag),
                    'T' | 'F' => Ok(Type::Boolean(type_flag == 'T')),
                    'N' => Ok(Type::Null()),
                    'I' => Ok(Type::Bang()),
                    's' => Type::try_from_buffer(data.next_string(), 's'),
                    'b' => Type::try_from_buffer(data.next_block_with_size(), 'b'),
                    _ => Err(enums::Error::OSC(enums::OSCError::UnknownType))
                }.ok()).collect();

                if osc_payload.len() != type_input_length {
                    return Err(enums::Error::Packet(enums::PacketError::InvalidTypesForMessage))
                }
            }

            Ok(Self {
                address : osc_address,
                args : osc_payload,
                force_empty_args
            })
        } else {
            Err(enums::Error::Packet(enums::PacketError::InvalidMessage))
        }
    }
}

// MARK: Bundle->Buffer
impl TryFrom<Bundle> for Buffer {
    type Error = enums::Error;

    fn try_from(value: Bundle) -> Result<Self, Self::Error> {
        let mut buffer = Self::from(enums::BUNDLE_TAG.to_vec());

        buffer.extend(&Type::TimeTag(value.time).into());
        
        for item in value.messages.clone() {
            let item_buffer = Self::try_from(item)?;
            let item_length = item_buffer.len();

            #[expect(clippy::cast_possible_truncation)]
            #[expect(clippy::cast_possible_wrap)]
            buffer.extend(&Type::Integer(item_length as i32).into());
            buffer.extend(&item_buffer);
        }
        Ok(buffer)
    }
}

// MARK: Bundle->String
impl fmt::Display for Bundle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "|#bundle•|{}", Type::TimeTag(self.time))?;

        for item in self.messages.clone() {
            write!(f, "M[{item}]")?;
        }
        Ok(())
    }
}

// MARK: Buffer->Bundle
impl TryFrom<Buffer> for Bundle {
    type Error = enums::Error;

    fn try_from(mut data: Buffer) -> Result<Self, Self::Error> {
        if !data.is_valid() {
            Err(enums::Error::Packet(enums::PacketError::NotFourByte))
        } else if Ok(enums::BUNDLE_TAG.to_vec()) == data.next_string() {
            let time_tag = Type::try_from_buffer(data.next_bytes(8), 't')?;
            let time = time_tag.try_into()?;

            let mut messages:Vec<Packet> = vec![];

            while ! data.is_empty() {
                match data.next_block() {
                    Ok(buffer) => {
                        match buffer.try_into() {
                            Ok(msg) => messages.push(msg),
                            Err(_) => { return Err(enums::Error::Packet(enums::PacketError::InvalidBuffer)); }
                        }
                    },
                    Err(_) => { return Err(enums::Error::Packet(enums::PacketError::InvalidBuffer)); }
                }
            }

            Ok(Self { time, messages })
        } else {
            Err(enums::Error::Packet(enums::PacketError::InvalidBuffer))
        }
    }
}

// MARK: Packet->Buffer
impl TryFrom<Packet> for Buffer {
    type Error = enums::Error;

    fn try_from(value: Packet) -> Result<Self, Self::Error> {
        match value {
            Packet::Message(v) => v.try_into(),
            Packet::Bundle(v) => v.try_into(),
        }
    }
}

// MARK: Packet->String
impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Message(v) => write!(f, "{v}"),
            Self::Bundle(v) => write!(f, "{v}")
        }
    }
}

// MARK: Buffer->Packet
impl TryFrom<Buffer> for Packet {
    type Error = enums::Error;

    fn try_from(data: Buffer) -> Result<Self, Self::Error> {
        if !data.is_valid() {
            Err(enums::Error::Packet(enums::PacketError::NotFourByte))
        } else if data.is_bundle() {
            match data.try_into() {
                Ok(v) => Ok(Self::Bundle(v)),
                Err(v) => Err(v)
            }
        } else {
            match data.try_into() {
                Ok(v) => Ok(Self::Message(v)),
                Err(v) => Err(v)
            }
        }
    }
}
