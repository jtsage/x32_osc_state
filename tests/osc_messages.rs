use x32_osc_state::osc::{TypeError, Buffer, Type, Message, Packet};
use chrono::DateTime;
use std::time::SystemTime;

const C_NULL:char = '\0';

#[test]
fn address_only() {
    let osc_packet = Message::new("/hello");
    let expected_buffer = Buffer::from(vec![
        '/', 'h', 'e', 'l', 'l', 'o', C_NULL, C_NULL,
    ]).into();

    let actual_buffer:Buffer = osc_packet.clone().try_into().expect("buffer pack failed");

    assert_eq!(actual_buffer, expected_buffer);
    assert_eq!(osc_packet.to_string(), String::from("|s:/hello••[8]||,:|"));

    let re_pack:Result<Message, _> = expected_buffer.clone().try_into();

    assert!(re_pack.is_ok());
    assert_eq!(osc_packet, re_pack.unwrap());
}

#[test]
fn address_only_force_empty_list() {
    let mut osc_packet = Message::new("/hello");
    let expected_buffer = Buffer::from(vec![
        '/', 'h', 'e', 'l', 'l', 'o', C_NULL, C_NULL,
        ',', C_NULL, C_NULL, C_NULL
    ]).into();

    osc_packet.force_empty_args = true;

    let actual_buffer:Buffer = osc_packet.clone().try_into().expect("buffer pack failed");

    assert_eq!(actual_buffer, expected_buffer);
    assert_eq!(osc_packet.to_string(), String::from("|s:/hello••[8]||,:,•••[4]|"));

    let re_pack:Result<Message, _> = expected_buffer.clone().try_into();

    assert!(re_pack.is_ok());
    assert_eq!(osc_packet, re_pack.unwrap());
}

#[test]
fn message_to_packet() {
    let message = Message::new("/hello");
    let expected_packet = Packet::Message(message.clone());

    let packet:Packet = message.into();
    
    assert_eq!(packet, expected_packet);
}

#[test]
fn single_add_by_primitive_string() {
    let mut osc_packet = Message::new("/hello");
    let expected_buffer = Buffer::from(vec![
        '/', 'h', 'e', 'l', 'l', 'o', C_NULL, C_NULL,
        ',', 's', C_NULL, C_NULL,
        'w', 'o', 'r', 'l', 'd', C_NULL, C_NULL, C_NULL
    ]).into();

    osc_packet.add_item(String::from("world"));

    let actual_buffer:Buffer = osc_packet.clone().try_into().expect("buffer pack failed");

    assert_eq!(actual_buffer, expected_buffer);
    assert_eq!(osc_packet.to_string(), String::from("|s:/hello••[8]||,:,s••[4]||s:world•••[8]|"));

    let re_pack:Result<Message, _> = expected_buffer.clone().try_into();

    assert!(re_pack.is_ok());
    assert_eq!(osc_packet, re_pack.unwrap())
}

#[test]
fn all_null_types() {
    let mut osc_packet = Message::new("/hello");
    let expected_buffer = Buffer::from(vec![
        '/', 'h', 'e', 'l', 'l', 'o', C_NULL, C_NULL,
        ',', 'I', 'N', 'T', 'F', C_NULL, C_NULL, C_NULL,
    ]).into();

    osc_packet.add_item(Type::Bang());
    osc_packet.add_item(Type::Null());
    osc_packet.add_item(true);
    osc_packet.add_item(false);

    let actual_buffer:Buffer = osc_packet.clone().try_into().expect("buffer pack failed");

    assert_eq!(actual_buffer, expected_buffer);
    assert_eq!(osc_packet.to_string(), String::from("|s:/hello••[8]||,:,INTF•••[8]||I:||N:||T:||F:|"));

    let re_pack:Result<Message, _> = expected_buffer.clone().try_into();

    assert!(re_pack.is_ok());
    assert_eq!(osc_packet, re_pack.unwrap())
}

#[test]
fn all_number_types() {
    let mut osc_packet = Message::new("/hello");
    let mut expected_buffer:Buffer = Buffer::from(vec![
        '/', 'h', 'e', 'l', 'l', 'o', C_NULL, C_NULL,
        ',', 'i', 'h', 'c', 'f', 'd', C_NULL, C_NULL,
        C_NULL, C_NULL, C_NULL, char::from(23),
        C_NULL, C_NULL, C_NULL, C_NULL, C_NULL, C_NULL, C_NULL, char::from(23),
        C_NULL, C_NULL, C_NULL, 'x',
    ]).into();

    let num_float = (69.69_f32).to_be_bytes();
    let num_double = (69.69_f64).to_be_bytes();

    expected_buffer.extend(&Buffer::from(num_float.to_vec()));
    expected_buffer.extend(&Buffer::from(num_double.to_vec()));

    osc_packet.add_item(23_i32);
    osc_packet.add_item(23_i64);
    osc_packet.add_item('x');
    osc_packet.add_item(69.69_f32);
    osc_packet.add_item(69.69_f64);

    let actual_buffer:Buffer = osc_packet.clone().try_into().expect("buffer pack failed");

    assert_eq!(actual_buffer, expected_buffer);
    assert_eq!(osc_packet.to_string(), String::from("|s:/hello••[8]||,:,ihcfd••[8]||i:23||h:23||c:x||f:69.69||d:69.69|"));

    let re_pack:Result<Message, _> = expected_buffer.clone().try_into();

    assert!(re_pack.is_ok());
    assert_eq!(osc_packet, re_pack.unwrap())
}


#[test]
fn complex_numbers() {
    let time_string = String::from("2000-04-25T01:30:30.125Z");
    let time_object = DateTime::parse_from_rfc3339(&time_string);
    let time_system = SystemTime::from(time_object.expect("unable to build sys time"));
    let time_arg = Type::try_from(time_system).expect("unable to build time arg");

    let mut osc_packet = Message::new("/hello");
    let mut expected_buffer:Buffer = Buffer::from(vec![
        '/', 'h', 'e', 'l', 'l', 'o', C_NULL, C_NULL,
        ',', 't', 'r', C_NULL,
    ]).into();

    let num_s = (3165615030_u32).to_be_bytes();
    let num_f = (536870912_u32).to_be_bytes();

    expected_buffer.extend(&Buffer::from(num_s.to_vec()));
    expected_buffer.extend(&Buffer::from(num_f.to_vec()));

    let color = [127_u8,127,127,255];
    expected_buffer.extend(&Buffer::from(color.clone().to_vec()));

    osc_packet.add_item(time_arg);
    osc_packet.add_item(color.clone());

    let actual_buffer:Buffer = osc_packet.clone().try_into().expect("buffer pack failed");

    assert_eq!(actual_buffer, expected_buffer);
    assert_eq!(osc_packet.to_string(), String::from("|s:/hello••[8]||,:,tr•[4]||t:[3165615030, 536870912]||r:[127, 127, 127, 255]|"));

    let re_pack:Result<Message, _> = expected_buffer.clone().try_into();

    assert!(re_pack.is_ok());
    assert_eq!(osc_packet, re_pack.unwrap())
}

#[test]
fn decode_unknown_type() {
    let buffer:Buffer = Buffer::from(vec![
        '/', 'h', 'e', 'l', 'l', 'o', C_NULL, C_NULL,
        ',', 'x', C_NULL, C_NULL,
    ]).into();

    let osc_packet:Result<Message, _> = buffer.try_into();

    assert!(osc_packet.is_err());
    assert_eq!(osc_packet, Err(TypeError::InvalidPacket));
}

#[test]
fn invalid_buffer() {
    let buffer = Buffer::from(vec![0x0, 0x0]);

    let decode:Result<Message, _> = buffer.try_into();

    assert!(decode.is_err());
    assert_eq!(decode, Err(TypeError::MisalignedBuffer))
}



#[test]
fn empty_buffer() {
    let buffer = Buffer::default();

    let decode:Result<Message, _> = buffer.try_into();

    assert!(decode.is_err());
    assert_eq!(decode, Err(TypeError::InvalidPacket));
}

#[test]
fn invalid_message_bad_arg() {
    let mut message = Message::new("hello");

    message.add_item(Type::Error(TypeError::InvalidTypeFlag));

    assert!(!message.is_valid());

    let buffer:Result<Buffer, _> = message.try_into();

    assert!(buffer.is_err());
    assert_eq!(buffer, Err(TypeError::InvalidPacket));
}

#[test]
fn invalid_message_bad_address() {
    let message = Message::new("hello💩");

    assert!(!message.is_valid());

    let buffer:Result<Buffer, _> = message.try_into();

    assert!(buffer.is_err());
    assert_eq!(buffer, Err(TypeError::InvalidPacket));
}


#[test]
fn single_send_blob() {
    let mut osc_packet = Message::new("/hello");
    let expected_buffer = Buffer::from(vec![
        '/', 'h', 'e', 'l', 'l', 'o', C_NULL, C_NULL,
        ',', 'b', 'i', C_NULL,
        C_NULL, C_NULL, C_NULL, char::from(6),
        'A', 'B', 'C', 'D', 'E', 'F', C_NULL, C_NULL,
        C_NULL, C_NULL, C_NULL, char::from(23)
    ]).into();

    let data_buffer = vec![0x41_u8, 0x42, 0x43, 0x44, 0x45, 0x46];

    osc_packet.add_item(Type::Blob(data_buffer));
    osc_packet.add_item(23_i32);

    let actual_buffer:Buffer = osc_packet.clone().try_into().expect("buffer pack failed");

    assert_eq!(actual_buffer, expected_buffer);
    assert_eq!(osc_packet.to_string(), String::from("|s:/hello••[8]||,:,bi•[4]||b:[~b:6~]||i:23|"));

    let re_pack:Result<Message, _> = expected_buffer.clone().try_into();

    assert!(re_pack.is_ok());
    assert_eq!(osc_packet, re_pack.unwrap())
}


#[test]
fn empty_send_blob() {
    let mut osc_packet = Message::new("/hello");
    let expected_buffer = Buffer::from(vec![
        '/', 'h', 'e', 'l', 'l', 'o', C_NULL, C_NULL,
        ',', 'b', C_NULL, C_NULL,
        C_NULL, C_NULL, C_NULL, char::from(0),
    ]).into();

    let data_buffer = vec![];

    osc_packet.add_item(Type::Blob(data_buffer));

    let actual_buffer:Buffer = osc_packet.clone().try_into().expect("buffer pack failed");

    assert_eq!(actual_buffer, expected_buffer);
    assert_eq!(osc_packet.to_string(), String::from("|s:/hello••[8]||,:,b••[4]||b:[~b:0~]|"));

    let re_pack:Result<Message, _> = expected_buffer.clone().try_into();

    assert!(re_pack.is_ok());
    assert_eq!(osc_packet, re_pack.unwrap())
}


#[test]
fn decode_blob_buffer_underrun() {
    let expected_buffer:Buffer = Buffer::from(vec![
        '/', 'h', 'e', 'l', 'l', 'o', C_NULL, C_NULL,
        ',', 'b', C_NULL, C_NULL,
        C_NULL, C_NULL, C_NULL, char::from(3),
    ]).into();

    let re_pack:Result<Message, _> = expected_buffer.clone().try_into();

    assert!(re_pack.is_err());
    assert_eq!(re_pack, Err(TypeError::InvalidPacket));
}