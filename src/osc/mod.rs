/// OSC Decoder and Encoder
///
/// Supported types
/// s :: string - string value (String padded to 32 bit block with nulls)
/// f :: float - numeric value (`f32`)
/// d :: double - numeric value (`f64`)
/// i :: integer - numeric value (`i32`)
/// h :: big int - numeric value (`i64`)
/// T :: true - no value (0 bits)
/// F :: false - no value (0 bits)
/// N :: null - no value (0 bits)
/// I :: bang - no value (0 bits)
/// r :: color - rgbA as an array [R(0-255),G,B,A] (`[u8;4]`)
/// c :: char - Character (`char` -> `u32`  - 32 bits)
/// t :: time tag - numeric value (date -> `u32` x2)
/// 
/// Unsupported types
/// 
/// b :: blob (error)
/// [] :: arrays (ignored)


use std::fmt;
use std::fmt::Write;

/// `OSCType` definitions
mod types;
/// `OSCPacket` definitions
mod packet;

pub use types::{Type, TimeTag};
pub use packet::{Packet, Bundle, Message};

/// #bundle tag
pub const BUNDLE_TAG:[u8;8] = [0x23, 0x62, 0x75, 0x6e, 0x64, 0x6c, 0x65, 0x0];

// MARK: BufferError
/// OSC Error Types
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum BufferError {
    /// buffer is not 4-byte aligned
    NotFourByte,
    /// buffer does not end with 1 or more nulls
    UnterminatedString,
    /// buffer not large enough for operation
    BufferUnderrun,
}

// MARK: BufferError->String
impl fmt::Display for BufferError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "osc::BufferError::{self:?}")
    }
}

// MARK: TypeError
/// OSC Error Types
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum TypeError {
    /// String from bytes failed
    ConvertFromString,
    /// Address is not valid
    AddressContent,
    /// Buffer not a multiple of 4 (string)
    MisalignedBuffer,
    /// Buffer not exactly 4 (int, float)
    MisalignedNumberBuffer,
    /// Unknown OSC type
    UnknownType,
    /// Invalid type conversion (named type)
    InvalidTypeFlag,
    /// Invalid type conversion (type -> primitive
    InvalidTypeConversion,
    /// Invalid packet (from buffer)
    InvalidPacket,
    /// Time underflow
    InvalidTimeUnderflow,
    /// Time overflow
    InvalidTimeOverflow,
}

// MARK: TypeError->String
impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "osc::TypeError::{self:?}")
    }
}


// MARK: Buffer
/// Buffer with extra methods
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Buffer {
    /// Internal vector data
    data : Vec<u8>,
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string:String = self.data
            .iter()
            .enumerate()
            .fold(String::new(), | mut output, (i, item)| {
                let nl_sp = if i != 0 && i % 4 == 3 { "\n" } else { " | " };
                let ch:char = match item {
                    0 => '•',
                    32..=126 => *item as char,
                    _ => '�'
                };
            
                let _ = write!(output, "{item:#04x} '{ch}'{nl_sp}");
                output
        });

        write!(f, "{display_string}")
    }
}
// MARK: Vec<u8>->Buffer
impl From<Vec<u8>> for Buffer {
    fn from(data: Vec<u8>) -> Self { Buffer { data } }
}

// MARK: Vec<ch>->Buffer
impl From<Vec<char>> for Buffer {
    fn from(data: Vec<char>) -> Self {
        let data:Vec<u8> = data.into_iter().map(|v| (v as u8)).collect();
        Buffer { data }
    }
}

// MARK: Iter<Type>->Buffer
impl FromIterator<types::Type> for Buffer {
    fn from_iter<T: IntoIterator<Item = types::Type>>(iter: T) -> Self {
        let mut buffer:Vec<u8> = vec![];

        for i in iter {
            buffer.extend(<Type as Into<Vec<u8>>>::into(i));
        }

        Buffer::from(buffer)
    }
}

impl FromIterator<types::Type> for String {
    fn from_iter<T: IntoIterator<Item = types::Type>>(iter: T) -> Self {
        let mut return_string = String::new();

        for i in iter {
            return_string.push_str(&i.to_string());
        }

        return_string
    }
}

// MARK: Buffer impl
impl Buffer {
    /// get length
    #[must_use]
    pub fn len(&self) -> usize { self.data.len() }

    /// check if buffer has a valid length
    #[must_use]
    pub fn is_valid(&self) -> bool { self.data.len() % 4 == 0 }

    /// check if buffer is empty
    #[must_use]
    pub fn is_empty(&self) -> bool { self.data.is_empty() }

    /// check if buffer if a bundle
    #[must_use]
    pub fn is_bundle(&self) -> bool { self.data.starts_with(&BUNDLE_TAG) }

    /// extend buffer with another buffer
    pub fn extend(&mut self, item : &Buffer) {
        self.data.extend(item.as_vec());
    }

    /// get buffer as a `&[u8]` slice
    #[must_use]
    pub fn as_slice(&self) -> &[u8] { self.data.as_slice() }

    /// get buffer as a vector
    #[must_use]
    pub fn as_vec(&self) -> Vec<u8> { self.data.clone() }

    /// get next string (until null)
    /// 
    /// # Errors
    /// - empty buffer
    /// - buffer length is 0
    /// - buffer is not a 4-byte multiple
    pub fn get_string(&mut self) -> Result<Vec<u8>, BufferError> {
        if self.is_empty() {
            Err(BufferError::BufferUnderrun)
        } else if !self.is_valid() {
            Err(BufferError::NotFourByte)
        } else {
            let mut this_buffer = vec![];
            while this_buffer.last() != Some(&0_u8) {
                if self.data.len() < 4 {
                    return Err(BufferError::UnterminatedString);
                }
                this_buffer.extend(self.data[0..4].to_vec());
                self.data = self.data[4 .. ].to_vec();
            }
            Ok(this_buffer)
        }
    }

    /// get bytes
    /// 
    /// # Errors
    /// - empty buffer
    /// - buffer length is 0
    /// - buffer is not a 4-byte multiple
    pub fn get_bytes(&mut self, length: usize) -> Result<Vec<u8>, BufferError> {
        if length == 0 {
            Ok(vec![])
        } else if self.is_empty() {
            Err(BufferError::BufferUnderrun)
        } else if !self.is_valid() || length % 4 != 0 {
            Err(BufferError::NotFourByte)
        } else if self.len() < length {
            Err(BufferError::BufferUnderrun)
        } else {
            let mut this_buffer = vec![];
            self.data[0..length].clone_into(&mut this_buffer);
            self.data = self.data[length..].to_vec();
            Ok(this_buffer)
        }
    }

    /// get sized byte block (include size in return)
    /// 
    /// # Errors
    /// - empty buffer
    /// - buffer length is less than 4 (4 = zero length buffer, maybe valid?)
    /// - buffer is not a 4-byte multiple
    pub fn get_next_byte_block(&mut self) -> Result<Vec<u8>, BufferError> {
        if self.len() < 4 {
            Err(BufferError::BufferUnderrun)
        } else if !self.is_valid() {
            Err(BufferError::NotFourByte)
        } else {
            let len_act_buff = self.data.clone()[0..4].try_into().map_err(|_| BufferError::BufferUnderrun)?;

            #[expect(clippy::cast_sign_loss)]
            let len_act = i32::from_be_bytes(len_act_buff) as usize;
            let len_tot = if len_act % 4 == 0 { len_act } else { len_act + (4 - (len_act % 4)) };
            let chunk_tot = len_tot + 4;

            if self.data.len() < ( chunk_tot ) {
                Err(BufferError::BufferUnderrun)
            } else {
                let mut this_buffer = vec![];
                self.data[0..chunk_tot].clone_into(&mut this_buffer);
                self.data = self.data[chunk_tot..].to_vec();
                Ok(this_buffer)
            }
        }
    }

    /// get sized byte block (drop size)
    /// 
    /// # Errors
    /// - empty buffer
    /// - buffer length is less than 4 (4 = zero length buffer, maybe valid?)
    /// - buffer is not a 4-byte multiple
    pub fn get_next_block(&mut self) -> Result<Buffer, BufferError> {
        if self.len() < 4 {
            Err(BufferError::BufferUnderrun)
        } else if !self.is_valid() {
            Err(BufferError::NotFourByte)
        } else {
            let len_act_buff = self.data.clone()[0..4].try_into().map_err(|_| BufferError::BufferUnderrun)?;

            #[expect(clippy::cast_sign_loss)]
            let chunk_tot = (i32::from_be_bytes(len_act_buff) as usize) + 4;

            if self.data.len() < ( chunk_tot ) {
                Err(BufferError::BufferUnderrun)
            } else {
                let mut this_buffer = vec![];
                self.data[4..chunk_tot].clone_into(&mut this_buffer);
                self.data = self.data[chunk_tot..].to_vec();
                Ok(Buffer::from(this_buffer))
            }
        }
    }
}

/// MARK: Buffer default
impl Default for Buffer {
    fn default() -> Self { Buffer { data : vec![] } }
}