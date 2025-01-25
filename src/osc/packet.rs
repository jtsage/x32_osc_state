/// OSC Packet definitions - messages and bundles, and `OSCData` container
use std::fmt;
use super::{Buffer, TypeError, Type, TimeTag, BUNDLE_TAG};



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
    time : TimeTag,
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
    pub fn new() -> Self {
        Bundle {
            time : TimeTag::now(),
            messages : vec![]
        }
    }

    /// Make a new future bundle (add "ms" to now)
    #[must_use]
    pub fn future(ms : u64) -> Self {
        Bundle {
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
        Message {
            address : address.to_owned(),
            args : vec![],
            force_empty_args : false
        }
    }

    /// Boolean is message valid
    #[must_use]
    pub fn is_valid(&self) -> bool {
        if self.address.is_ascii() && !self.address.is_empty() {
            !self.args.clone().iter().any(|s| matches!(s, Type::Error(_)))
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
            .filter_map(|x| x.get_type_char().ok())
            .collect();
        
        list.into()
    }

    /// Pack the arguments into an `OSCBuffer`
    fn pack_args(&self) -> Buffer {
        if self.args.is_empty() {
            Buffer::default()
        } else {
            self.args.clone().into_iter().collect()
        }
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
impl TryInto<Buffer> for Message {
    type Error = TypeError;

    fn try_into(self) -> Result<Buffer, Self::Error> {
        if !self.is_valid() { return Err(TypeError::InvalidPacket); }

        let mut osc_buffer:Buffer = Type::String(self.address.clone()).into();

        if self.force_empty_args && self.args.is_empty() {
            osc_buffer.extend(&Buffer::from(vec![0x2c, 0x0, 0x0, 0x0]));
        } else {
            osc_buffer.extend(&<Type as Into<Buffer>>::into(self.type_list()));
        }
        osc_buffer.extend(&self.pack_args());

        Ok(osc_buffer)
    }
}

// MARK: Buffer->Message
impl TryFrom<Buffer> for Message {
    type Error = TypeError;

    fn try_from(mut data: Buffer) -> Result<Self, Self::Error> {
        if !data.is_valid() {
            Err(TypeError::MisalignedBuffer)
        } else if let Ok(Type::String(osc_address)) = Type::decode_buffer(data.get_string(), 's') {
            let mut force_empty_args = false;
            let mut osc_payload:Vec<Type> = vec![];

            if let Ok(Type::TypeList(osc_types)) = Type::decode_buffer(data.get_string(), ',') {
                if osc_types.is_empty() { force_empty_args = true }

                let type_input_length= osc_types.len();

                osc_payload = osc_types.into_iter().filter_map(|type_flag| match type_flag {
                    'i' | 'f' | 'c' | 'r' => Type::decode_buffer(data.get_bytes(4), type_flag),
                    'h' | 'd' | 't' => Type::decode_buffer(data.get_bytes(8), type_flag),
                    'T' | 'F' => Ok(Type::Boolean(type_flag == 'T')),
                    'N' => Ok(Type::Null()),
                    'I' => Ok(Type::Bang()),
                    's' => Type::decode_buffer(data.get_string(), 's'),
                    'b' => Type::decode_buffer(data.get_next_byte_block(), 'b'),
                    _ => Err(TypeError::UnknownType)
                }.ok()).collect();

                if osc_payload.len() != type_input_length {
                    return Err(TypeError::InvalidPacket)
                }
            }

            Ok(Message {
                address : osc_address,
                args : osc_payload,
                force_empty_args
            })
        } else {
            Err(TypeError::InvalidPacket)
        }
    }
}

// MARK: Bundle->Buffer
impl TryInto<Buffer> for Bundle {
    type Error = TypeError;

    fn try_into(self) -> Result<Buffer, Self::Error> {
        let mut buffer = Buffer::from(BUNDLE_TAG.clone().to_vec());

        buffer.extend(&Type::TimeTag(self.time).into());
        
        for item in self.messages.clone() {
            let item_buffer:Buffer = item.try_into()?;
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
        write!(f, "|#bundle•|")?;
        write!(f, "{}", Type::TimeTag(self.time.clone()))?;

        for item in self.messages.clone() {
            write!(f, "M[{item}]")?;
        }
        Ok(())
    }
}

// MARK: Buffer->Bundle
impl TryFrom<Buffer> for Bundle {
    type Error = TypeError;

    fn try_from(mut data: Buffer) -> Result<Self, Self::Error> {
        if !data.is_valid() {
            Err(TypeError::MisalignedBuffer)
        } else if Ok(BUNDLE_TAG.clone().to_vec()) == data.get_string() {
            let time_tag = Type::decode_buffer(data.get_bytes(8), 't')?;
            let time = time_tag.try_into()?;

            let mut messages:Vec<Packet> = vec![];

            while ! data.is_empty() {
                match data.get_next_block() {
                    Ok(buffer) => {
                        match buffer.try_into() {
                            Ok(msg) => messages.push(msg),
                            Err(_) => { return Err(TypeError::InvalidPacket); }
                        }
                    },
                    Err(_) => { return Err(TypeError::InvalidPacket); }
                }
            }

            Ok(Bundle {
                time,
                messages,
            })
        } else {
            Err(TypeError::InvalidPacket)
        }
    }
}

// MARK: Packet->Buffer
impl TryInto<Buffer> for Packet {
    type Error = TypeError;

    fn try_into(self) -> Result<Buffer, Self::Error> {
        match self {
            Packet::Message(v) => v.try_into(),
            Packet::Bundle(v) => v.try_into(),
        }
    }
}

// MARK: Packet->String
impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Packet::Message(v) => write!(f, "{v}"),
            Packet::Bundle(v) => write!(f, "{v}")
        }
    }
}

// MARK: Buffer->Packet
impl TryFrom<Buffer> for Packet {
    type Error = TypeError;

    fn try_from(data: Buffer) -> Result<Self, Self::Error> {
        if !data.is_valid() {
            Err(TypeError::MisalignedBuffer)
        } else if data.is_bundle() {
            match data.try_into() {
                Ok(v) => Ok(Packet::Bundle(v)),
                Err(v) => Err(v)
            }
        } else {
            match data.try_into() {
                Ok(v) => Ok(Packet::Message(v)),
                Err(v) => Err(v)
            }
        }
    }
}
