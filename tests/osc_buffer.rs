use x32_osc_state::osc::Buffer;
use x32_osc_state::enums::{Error, PacketError};

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
                assert_eq!(buffer.clone().get_bytes(4).unwrap_err(), Error::Packet(PacketError::NotFourByte));
                assert_eq!(buffer.clone().get_bytes(8).unwrap_err(), Error::Packet(PacketError::NotFourByte));
                assert_eq!(buffer.clone().get_string().unwrap_err(), Error::Packet(PacketError::NotFourByte));
            } else if can_4 && !can_8 {
                assert_eq!(buffer.clone().get_bytes(8).unwrap_err(), Error::Packet(PacketError::Underrun));
            } else if !can_str {
                assert_eq!(buffer.clone().get_string().unwrap_err(), Error::Packet(PacketError::UnterminatedString));
            }
            assert_eq!(buffer.clone().get_bytes(4).is_ok(), can_4, "4-byte");
            assert_eq!(buffer.clone().get_bytes(8).is_ok(), can_8, "8-byte");
            assert_eq!(buffer.clone().get_string().is_ok(), can_str, "string");
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
    assert!(buffer.clone().get_bytes(0).is_ok(), "0-byte");
    assert!(buffer.clone().get_bytes(2).is_err(), "2-byte");
    assert!(buffer.clone().get_bytes(4).is_ok(), "4-byte");
    assert!(buffer.clone().get_bytes(6).is_err(), "6-byte");
}

#[test]
fn error_type_check() {
    let empty_byte:Buffer = Default::default();
    let three_byte = Buffer::from(rnd_buffer(3));
    let four_byte = Buffer::from(rnd_buffer(4));
    let unterminated_string = Buffer::from(rnd_buffer(4));

    assert_eq!(three_byte.clone().get_bytes(4), Err(Error::Packet(PacketError::NotFourByte)));
    assert_eq!(three_byte.clone().get_string(), Err(Error::Packet(PacketError::NotFourByte)));

    assert_eq!(four_byte.clone().get_bytes(8), Err(Error::Packet(PacketError::Underrun)));
    assert_eq!(empty_byte.clone().get_string(), Err(Error::Packet(PacketError::Underrun)));
    assert_eq!(empty_byte.clone().get_bytes(4), Err(Error::Packet(PacketError::Underrun)));

    assert_eq!(unterminated_string.clone().get_string(), Err(Error::Packet(PacketError::UnterminatedString)));

    assert_eq!(Error::Packet(PacketError::Underrun).to_string(), "Packet(Underrun)");
    assert_eq!(Error::Packet(PacketError::UnterminatedString).to_string(), "Packet(UnterminatedString)");
    assert_eq!(Error::Packet(PacketError::NotFourByte).to_string(), "Packet(NotFourByte)");
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

    assert_eq!(empty_buffer.clone().get_next_block(), Err(Error::Packet(PacketError::Underrun)));
    assert_eq!(empty_buffer.clone().get_next_byte_block(), Err(Error::Packet(PacketError::Underrun)));

    assert_eq!(invalid_buffer.clone().get_next_block(), Err(Error::Packet(PacketError::NotFourByte)));
    assert_eq!(invalid_buffer.clone().get_next_byte_block(), Err(Error::Packet(PacketError::NotFourByte)));
}

