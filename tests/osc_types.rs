use x32_osc_state::enums::{Error, OSCError, PacketError};
use x32_osc_state::osc::{Buffer, Type};
use chrono::DateTime;
use std::time::SystemTime;

mod buffer_common;
use buffer_common::rnd_buffer;

macro_rules! simple_type_test {
    ($($name:ident: $value:expr, $ty:ty,)*) => {
    $(
        #[test]
        fn $name() {
            let (input, type_char, buffer_len, formatted_string) = $value;
            let osc_type:Type = input.clone().into();
            
            assert!(!osc_type.is_error(), "into type");
            assert_eq!(osc_type.as_type_char(), Ok(type_char), "char match");

            let orig_value:Result<$ty, _> = osc_type.clone().try_into();

            assert_eq!(orig_value, Ok(input), "orig match extracted");
            assert_eq!(osc_type.to_string(), formatted_string, "string fmt");

            let buffer:Buffer = osc_type.clone().try_into().expect("buffer pack failed");

            assert!(buffer.is_valid(), "valid buffer");
            assert_eq!(buffer.len(), buffer_len);

            let re_read:Result<Type, _> = (buffer.as_slice(), type_char).try_into();

            assert!(!re_read.is_err(), "buffer re-read {:?}", re_read);
            assert_eq!(re_read.unwrap(), osc_type, "original matches re-read");
        }
    )*
    }
}

simple_type_test! {
    integer : (23_i32, 'i', 4, "|i:23|"), i32,
    long_integer : (23_i64, 'h', 8, "|h:23|"), i64,
    float : (69.69_f32, 'f', 4, "|f:69.69|"), f32,
    double : (69.69_f64, 'd', 8, "|d:69.69|"), f64,
    string : (String::from("hello"), 's', 8, "|s:hello•••[8]|"), String,
    string_utf8 : (String::from("hello💩"), 's', 12, "|s:hello💩•••[12]|"), String,
    type_list : (vec!['a','b'], ',', 4, "|,:,ab•[4]|"), Vec<char>,
    boolean_true : (true, 'T', 0, "|T:|"), bool,
    boolean_false : (false, 'F', 0, "|F:|"), bool,
    char : ('x', 'c', 4, "|c:x|"), char,
    color : ([127_u8, 127_u8, 127_u8, 255_u8], 'r', 4, "|r:[127, 127, 127, 255]|"), [u8;4],
    // time_tag : (TimeTag::from((46_u32, 92_u32)), 't', 8, "|t:[46, 92]|"), TimeTag,
    bang : (Type::Bang(), 'I', 0, "|I:|"), Type,
    null : (Type::Null(), 'N', 0, "|N:|"), Type,
    type_list_empty : (vec![], ',', 0, "|,:|"), Vec<char>,
    string_empty : (String::new(), 's', 4, "|s:••••[4]|"), String,
}

#[test]
fn buffer_size() {
    let ar_1  = rnd_buffer(1);
    let ar_2  = rnd_buffer(2);
    let ar_3  = rnd_buffer(3);
    let ar_4  = rnd_buffer(4);
    let ar_6  = rnd_buffer(6);
    let ar_8  = rnd_buffer(8);

    let error_type_bad_number = Err(Error::Packet(PacketError::Underrun));
    let error_type_bad_buffer = Err(Error::Packet(PacketError::NotFourByte));

    assert_eq!(Type::try_from_vec(&ar_1, 'f'), error_type_bad_buffer);
    assert_eq!(Type::try_from_vec(&ar_2, 'f'), error_type_bad_buffer);
    assert_eq!(Type::try_from_vec(&ar_3, 'f'), error_type_bad_buffer);
    assert_eq!(Type::try_from_vec(&ar_6, 'f'), error_type_bad_buffer);

    assert!(matches!(Type::try_from_vec(&ar_4, 'f'), Ok(Type::Float(_))));
    assert_eq!(Type::try_from_vec(&ar_8, 'f'), error_type_bad_number);

    assert_eq!(Type::try_from_vec(&ar_4, 'd'), error_type_bad_number);
    assert!(matches!(Type::try_from_vec(&ar_8, 'd'), Ok(Type::Double(_))));

    assert_eq!(Type::try_from_vec(&ar_8, 'r'), error_type_bad_number);
    assert_eq!(Type::try_from_vec(&ar_8, 'c'), error_type_bad_number);

    assert_eq!(Type::try_from_buffer( Err(Error::Packet(PacketError::Underrun)), 'f'),  Err(Error::Packet(PacketError::Underrun)));
    assert!(matches!(Type::try_from_buffer(Ok(ar_4.clone()), 'f'), Ok(Type::Float(_))));
}


#[test]
fn invalid_type_conversion_to_osc_type() {
    let osc_type = Type::from(12_i32);

    let decoded:Result<String, _> = osc_type.try_into();

    assert!(decoded.is_err());
    assert_eq!(decoded, Err(Error::OSC(OSCError::InvalidTypeConversion)));
}

#[test]
fn decode_unknown_type() {
    let buffer = rnd_buffer(4);
    let osc_type = Type::try_from_vec(&buffer, 'x');

    assert_eq!(osc_type, Err(Error::OSC(OSCError::InvalidTypeFlag)));
}

#[test]
fn encode_unknown_type() {
    let osc_type = Type::Unknown();

    assert_eq!(osc_type.as_type_char().unwrap_err(), Error::OSC(OSCError::UnknownType));
}

#[test]
fn cast_default_type() {
    let osc_type:Type = Default::default();

    assert_eq!(osc_type, Type::Unknown());
}


#[test]
fn type_char_invalid() {
    let osc_type_flag ='c';
    let osc_buffer = Buffer::from(vec![0x0, 0x0, 0xde, 0x01]);

    let osc_type = Type::try_from((osc_buffer.as_slice(), osc_type_flag));

    assert_eq!(osc_type, Err(Error::OSC(OSCError::ConvertFromString)));
}

#[test]
fn type_string_invalid() {
    let osc_type_flag ='s';
    let raw_buffer:Vec<u8> = vec![0x0, 0x0, 0xde, 0x01, 0x64, 0x64, 0x64, 0x0];
    let osc_buffer = Buffer::from(raw_buffer.clone());

    let osc_type = Type::try_from((osc_buffer.as_slice(), osc_type_flag));
    let osc_type_opt = Type::try_from_buffer(Ok(raw_buffer), osc_type_flag);

    assert!(osc_type_opt.is_err());
    assert_eq!(osc_type, Err(Error::OSC(OSCError::ConvertFromString)));
}

// MARK: time tags
#[test]
fn type_time_too_early() {
    let time_string = String::from("1932-01-01T06:00:00.000Z");
    let time_object = DateTime::parse_from_rfc3339(&time_string);

    assert!(time_object.is_ok());

    let time_system = SystemTime::from(time_object.unwrap());

    let decoded:Result<Type, _> = Type::try_from(time_system);

    assert!(decoded.is_err());
    assert_eq!(decoded, Err(Error::OSC(OSCError::InvalidTimeUnderflow)));
}

#[test]
fn type_time_too_late() {
    let time_string = String::from("2045-01-01T06:00:00.000Z");
    let time_object = DateTime::parse_from_rfc3339(&time_string);

    assert!(time_object.is_ok());

    let time_system = SystemTime::from(time_object.unwrap());

    let decoded:Result<Type, _> = Type::try_from(time_system);

    assert!(decoded.is_err());
    assert_eq!(decoded, Err(Error::OSC(OSCError::InvalidTimeOverflow)));
}

#[test]
fn type_time() {
    let time_string = String::from("2000-04-25T01:30:30.125Z");
    let time_object = DateTime::parse_from_rfc3339(&time_string);
    let osc_buffer = Buffer::from(vec![0xbc, 0xaf, 0x73, 0xb6, 0x20, 0x0, 0x0, 0x0]);

    assert!(time_object.is_ok());

    let time_system = SystemTime::from(time_object.unwrap());

    let osc_type = Type::try_from(time_system).unwrap();

    assert!(!osc_type.is_error());
    assert_eq!(<Type as Into<Buffer>>::into(osc_type.clone()), osc_buffer);
    assert_eq!(osc_type.as_type_char(), Ok('t'));

    let osc_value:SystemTime = osc_type.clone().try_into().expect("conversion error");

    assert_eq!(osc_value, time_system);
    assert_eq!(osc_type.to_string(), format!("|t:[3165615030, 536870912]|"));

    assert_eq!(Type::try_from_vec(&osc_buffer.as_vec(), 't'), Ok(osc_type));
}

#[test]
fn time_output_error() {
    let osc_type = Type::from(23_i32);

    let decoded:Result<SystemTime,_> = osc_type.try_into();

    assert!(decoded.is_err());
    assert_eq!(decoded, Err(Error::OSC(OSCError::InvalidTypeConversion)));
}

#[test]
fn blob_type_good_six() {
    let blob_buffer:Vec<u8> = vec![0x0, 0x0, 0xde, 0x01, 0x64, 0x64];
    let expect_buffer:Vec<u8> = vec![0x0, 0x0, 0x0, 0x6, 0x0, 0x0, 0xde, 0x01, 0x64, 0x64, 0x0, 0x0];

    let osc_type = Type::Blob(blob_buffer);

    assert_eq!(osc_type.as_type_char(), Ok('b'));

    let packed_buffer:Buffer = osc_type.clone().into();

    assert_eq!(packed_buffer.as_vec(), expect_buffer);

    let re_pack = Type::try_from_vec(&expect_buffer, 'b');

    assert_eq!(osc_type.to_string(), "|b:[~b:6~]|");
    assert_eq!(osc_type, re_pack.unwrap());
}

#[test]
fn blob_type_good_eight() {
    let blob_buffer:Vec<u8> = vec![0x0, 0x0, 0xde, 0x01, 0x64, 0x64, 0x2, 0x2];
    let expect_buffer:Vec<u8> = vec![0x0, 0x0, 0x0, 0x8, 0x0, 0x0, 0xde, 0x01, 0x64, 0x64, 0x2, 0x2];

    let osc_type = Type::Blob(blob_buffer);

    let packed_buffer:Buffer = osc_type.clone().into();

    assert_eq!(packed_buffer.as_vec(), expect_buffer);

    let re_pack = Type::try_from_vec(&expect_buffer, 'b');

    assert_eq!(osc_type, re_pack.unwrap());
}

#[test]
fn blob_type_short_twelve() {
    let expect_buffer:Vec<u8> = vec![0x0, 0x0, 0x0, 0x12, 0x0, 0x0, 0xde, 0x01, 0x64, 0x64, 0x2, 0x2];

    let re_pack = Type::try_from_vec(&expect_buffer, 'b');

    assert!(re_pack.is_err());
    assert_eq!(re_pack, Err(Error::Packet(PacketError::Underrun)))
}


#[test]
fn blob_type_empty() {
    let expect_buffer:Vec<u8> = vec![];

    let re_pack = Type::try_from_vec(&expect_buffer, 'b');

    assert!(re_pack.is_err());
    assert_eq!(re_pack, Err(Error::Packet(PacketError::Underrun)));
}

