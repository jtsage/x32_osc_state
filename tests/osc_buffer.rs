use x32_osc_state::osc::Buffer;
use x32_osc_state::enums::{Error, PacketError, OSCError, X32Error};

mod buffer_common;
use buffer_common::*;

#[test]
fn simple_buffer() {
    let mut buffer = Buffer::from(vec!['g', 'o', 'o', 'd', 'w', 'i', 'l', 'l']);

    assert!(buffer.is_valid());
    assert_eq!(buffer.as_slice().len(), 8);
    assert_eq!(buffer.as_vec().len(), 8);
    assert_eq!(buffer.to_string(), String::from("0x67 'g' | 0x6f 'o' | 0x6f 'o' | 0x64 'd'\n0x77 'w' | 0x69 'i' | 0x6c 'l' | 0x6c 'l'\n"));

    buffer.extend(&buffer.clone());
    assert_eq!(buffer.as_slice().len(), 16);
    assert_eq!(buffer.as_vec().len(), 16);
}

macro_rules! buffer_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let (input, is_valid, can_4, can_8, can_str) = $value;
            let buffer = Buffer::from(input);

            assert_eq!(buffer.is_valid(), is_valid, "valid");
            if !is_valid {
                assert_eq!(buffer.clone().next_bytes(4).unwrap_err(), Error::Packet(PacketError::NotFourByte));
                assert_eq!(buffer.clone().next_bytes(8).unwrap_err(), Error::Packet(PacketError::NotFourByte));
                assert_eq!(buffer.clone().next_string().unwrap_err(), Error::Packet(PacketError::NotFourByte));
            } else if can_4 && !can_8 {
                assert_eq!(buffer.clone().next_bytes(8).unwrap_err(), Error::Packet(PacketError::Underrun));
            } else if !can_str {
                assert_eq!(buffer.clone().next_string().unwrap_err(), Error::Packet(PacketError::UnterminatedString));
            }
            assert_eq!(buffer.clone().next_bytes(4).is_ok(), can_4, "4-byte");
            assert_eq!(buffer.clone().next_bytes(8).is_ok(), can_8, "8-byte");
            assert_eq!(buffer.clone().next_string().is_ok(), can_str, "string");
        }
    )*
    }
}

buffer_tests! {
    byte_1: (rnd_buffer(1), false, false, false, false),
    byte_2: (rnd_buffer(2), false, false, false, false),
    byte_3: (rnd_buffer(3), false, false, false, false),
    byte_4: (rnd_buffer(4), true, true, false, false),
    byte_5: (rnd_buffer(5), false, false, false, false),
    byte_6: (rnd_buffer(6), false, false, false, false),
    byte_7: (rnd_buffer(7), false, false, false, false),
    byte_8_str: (rnd_string_buffer(8), true, true, true, true),
    byte_8_no_str: (rnd_buffer(8), true, true, true, false),
}

#[test]
fn arbitrary_bytes() {
    let buffer = Buffer::from(rnd_string_buffer(8));

    assert!(buffer.is_valid());
    assert!(buffer.clone().next_bytes(0).is_ok(), "0-byte");
    assert!(buffer.clone().next_bytes(2).is_err(), "2-byte");
    assert!(buffer.clone().next_bytes(4).is_ok(), "4-byte");
    assert!(buffer.clone().next_bytes(6).is_err(), "6-byte");
}

#[test]
fn error_type_check() {
    let empty_byte:Buffer = Default::default();
    let three_byte = Buffer::from(rnd_buffer(3));
    let four_byte = Buffer::from(rnd_buffer(4));
    let unterminated_string = Buffer::from(rnd_buffer(4));

    assert_eq!(three_byte.clone().next_bytes(4), Err(Error::Packet(PacketError::NotFourByte)));
    assert_eq!(three_byte.clone().next_string(), Err(Error::Packet(PacketError::NotFourByte)));

    assert_eq!(four_byte.clone().next_bytes(8), Err(Error::Packet(PacketError::Underrun)));
    assert_eq!(empty_byte.clone().next_string(), Err(Error::Packet(PacketError::Underrun)));
    assert_eq!(empty_byte.clone().next_bytes(4), Err(Error::Packet(PacketError::Underrun)));

    assert_eq!(unterminated_string.clone().next_string(), Err(Error::Packet(PacketError::UnterminatedString)));

}

#[test]
fn check_debug_output() {
    let buffer = Buffer::from(vec![0x0, 0x43, 0xCE, 0xDE]);

    assert_eq!(buffer.to_string(), "0x00 '•' | 0x43 'C' | 0xce '�' | 0xde '�'\n");
}

#[test]
fn get_next_checks() {
    let empty_buffer = Buffer::default();
    let invalid_buffer = Buffer::from(vec![0x0, 0x0, 0x0, 0x0, 0x0]);

    assert_eq!(empty_buffer.clone().next_block(), Err(Error::Packet(PacketError::Underrun)));
    assert_eq!(empty_buffer.clone().next_block_with_size(), Err(Error::Packet(PacketError::Underrun)));

    assert_eq!(invalid_buffer.clone().next_block(), Err(Error::Packet(PacketError::NotFourByte)));
    assert_eq!(invalid_buffer.clone().next_block_with_size(), Err(Error::Packet(PacketError::NotFourByte)));
}

#[test]
fn error_type_impl_checks() {
    assert_eq!(Error::Packet(PacketError::NotFourByte).to_string(), "buffer error: not 4-byte aligned");
    assert_eq!(Error::Packet(PacketError::UnterminatedString).to_string(), "buffer error: string not terminated with 0x0 null");
    assert_eq!(Error::Packet(PacketError::Underrun).to_string(), "buffer error: buffer not large enough for operation");
    assert_eq!(Error::Packet(PacketError::InvalidBuffer).to_string(), "buffer error: buffer contains invalid data");
    assert_eq!(Error::Packet(PacketError::InvalidMessage).to_string(), "buffer error: message conversion invalid");
    assert_eq!(Error::Packet(PacketError::InvalidTypesForMessage).to_string(), "buffer error: type conversion invalid");

    assert_eq!(Error::OSC(OSCError::ConvertFromString).to_string(), "osc error: string conversion failed");
    assert_eq!(Error::OSC(OSCError::AddressContent).to_string(), "osc error: address is not ascii");
    assert_eq!(Error::OSC(OSCError::UnknownType).to_string(), "osc error: unknown OSC type");
    assert_eq!(Error::OSC(OSCError::InvalidTypeFlag).to_string(), "osc error: unknown OSC type flag");
    assert_eq!(Error::OSC(OSCError::InvalidTypeConversion).to_string(), "osc error: type conversion invalid");
    assert_eq!(Error::OSC(OSCError::InvalidTimeUnderflow).to_string(), "osc error: time too early to represent");
    assert_eq!(Error::OSC(OSCError::InvalidTimeOverflow).to_string(), "osc error: time too late to represent");

    assert_eq!(Error::X32(X32Error::InvalidFader).to_string(), "x32 error: invalid fader");
    assert_eq!(Error::X32(X32Error::UnimplementedPacket).to_string(), "x32 error: unhandled message");
    assert_eq!(Error::X32(X32Error::MalformedPacket).to_string(), "x32 error: packet format invalid - not enough arguments");

    
}

#[test]
fn error_source() {
    use std::error::Error;

    assert_eq!(crate::Error::OSC(OSCError::AddressContent).source().unwrap().to_string(), "address is not ascii");
    assert_eq!(crate::Error::X32(X32Error::InvalidFader).source().unwrap().to_string(), "invalid fader");
    assert_eq!(crate::Error::Packet(PacketError::InvalidBuffer).source().unwrap().to_string(), "buffer contains invalid data");

}