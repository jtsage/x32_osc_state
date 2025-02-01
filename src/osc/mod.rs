/// OSC Decoder and Encoder
///
/// Supported types
/// s :: string - string value (String padded to 32 bit block with nulls)
/// f :: float - numeric value
/// d :: double - numeric value
/// i :: integer - numeric value
/// h :: big int - numeric value
/// T :: true - no value (0 bits)
/// F :: false - no value (0 bits)
/// N :: null - no value (0 bits)
/// I :: bang - no value (0 bits)
/// r :: color - rgbA as an array [R(0-255),G,B,A] (`[u8;4]`)
/// c :: char - Character
/// t :: time tag - numeric value (date -> `[u32;2]`)
/// 
/// Unsupported types
/// 
/// b :: blob (error)
/// [] :: arrays (ignored)


use std::fmt;
use std::fmt::Write;

/// [`Type`] definitions
mod types;
/// [`Packet`] definitions
mod packet;

use super::enums;

pub use types::Type;
pub use packet::{Packet, Bundle, Message};


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
    fn from(data: Vec<u8>) -> Self { Self { data } }
}

// MARK: Vec<ch>->Buffer
impl From<Vec<char>> for Buffer {
    fn from(data: Vec<char>) -> Self {
        let data:Vec<u8> = data.into_iter().map(|v| (v as u8)).collect();
        Self { data }
    }
}

// MARK: Iter<Type>->Buffer
impl FromIterator<types::Type> for Buffer {
    fn from_iter<T: IntoIterator<Item = types::Type>>(iter: T) -> Self {
        let mut buffer:Vec<u8> = vec![];

        for i in iter {
            buffer.extend(<types::Type as Into<Vec<u8>>>::into(i));
        }

        Self::from(buffer)
    }
}

impl FromIterator<types::Type> for String {
    fn from_iter<T: IntoIterator<Item = types::Type>>(iter: T) -> Self {
        let mut return_string = Self::new();

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
    pub fn is_bundle(&self) -> bool { self.data.starts_with(&enums::BUNDLE_TAG) }

    /// extend buffer with another buffer
    pub fn extend(&mut self, item : &Self) {
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
    pub fn next_string(&mut self) -> Result<Vec<u8>, enums::Error> {
        if self.is_empty() {
            Err(enums::Error::Packet(enums::PacketError::Underrun))
        } else if !self.is_valid() {
            Err(enums::Error::Packet(enums::PacketError::NotFourByte))
        } else {
            let mut this_buffer = vec![];
            while this_buffer.last() != Some(&0_u8) {
                if self.data.len() < 4 {
                    return Err(enums::Error::Packet(enums::PacketError::UnterminatedString));
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
    pub fn next_bytes(&mut self, length: usize) -> Result<Vec<u8>, enums::Error> {
        if length == 0 {
            Ok(vec![])
        } else if self.is_empty() {
            Err(enums::Error::Packet(enums::PacketError::Underrun))
        } else if !self.is_valid() || length % 4 != 0 {
            Err(enums::Error::Packet(enums::PacketError::NotFourByte))
        } else if self.len() < length {
            Err(enums::Error::Packet(enums::PacketError::Underrun))
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
    pub fn next_block_with_size(&mut self) -> Result<Vec<u8>, enums::Error> {
        if self.len() < 4 {
            Err(enums::Error::Packet(enums::PacketError::Underrun))
        } else if !self.is_valid() {
            Err(enums::Error::Packet(enums::PacketError::NotFourByte))
        } else {
            let len_act_buff = [self.data[0], self.data[1], self.data[2], self.data[3]];
            
            #[expect(clippy::cast_sign_loss)]
            let len_act = i32::from_be_bytes(len_act_buff) as usize;
            let len_tot = if len_act % 4 == 0 { len_act } else { len_act + (4 - (len_act % 4)) };
            let chunk_tot = len_tot + 4;

            if self.data.len() < ( chunk_tot ) {
                Err(enums::Error::Packet(enums::PacketError::Underrun))
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
    pub fn next_block(&mut self) -> Result<Self, enums::Error> {
        if self.len() < 4 {
            Err(enums::Error::Packet(enums::PacketError::Underrun))
        } else if !self.is_valid() {
            Err(enums::Error::Packet(enums::PacketError::NotFourByte))
        } else {
            let len_act_buff = [self.data[0], self.data[1], self.data[2], self.data[3]];

            #[expect(clippy::cast_sign_loss)]
            let chunk_tot = (i32::from_be_bytes(len_act_buff) as usize) + 4;

            if self.data.len() < ( chunk_tot ) {
                Err(enums::Error::Packet(enums::PacketError::Underrun))
            } else {
                let mut this_buffer = vec![];
                self.data[4..chunk_tot].clone_into(&mut this_buffer);
                self.data = self.data[chunk_tot..].to_vec();
                Ok(Self::from(this_buffer))
            }
        }
    }
}

/// MARK: Buffer default
impl Default for Buffer {
    fn default() -> Self { Self { data : vec![] } }
}