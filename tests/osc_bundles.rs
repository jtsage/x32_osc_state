use x32_osc_state::osc::{Buffer, Packet, Bundle, Message, Type, TypeError};

#[test]
fn empty_bundle() {
    let bundle = Bundle::default();
    let buffer:Buffer = bundle.try_into().expect("unable to pack");

    assert!(buffer.is_valid());
    assert_eq!(buffer.len(), 16);
}

#[test]
fn empty_future_bundle() {
    let bundle = Bundle::future(2500);
    let buffer:Buffer = bundle.try_into().expect("unable to pack");

    assert!(buffer.is_valid());
    assert_eq!(buffer.len(), 16);
}

#[test]
fn single_message() {
	let mut bundle = Bundle::default();
	let mut message = Message::new("/hello");

	message.add_item(23_i32);

	bundle.add(message);

	let data = Packet::Bundle(bundle.clone());
	let buffer:Buffer = data.clone().try_into().expect("unable to pack buffer");

	assert!(buffer.is_valid());
	assert_eq!(buffer.len(), 36);

	let re_read:Packet = buffer.clone().try_into().expect("unable to pack struct");

	assert!(data.to_string().starts_with("|#bundle•||t:["));
	assert!(data.to_string().ends_with("]|M[|s:/hello••[8]||,:,i••[4]||i:23|]"));

	assert!(re_read.to_string().starts_with("|#bundle•||t:["));
	assert!(re_read.to_string().ends_with("]|M[|s:/hello••[8]||,:,i••[4]||i:23|]"));

	assert_eq!(re_read, data);

    match bundle.messages.get(0).unwrap() {
        Packet::Message(msg) => {
            let arg_1 = msg.args.get(0).expect("no args");
            assert_eq!(<Type as TryInto<i32>>::try_into(arg_1.clone()), Ok(23_i32));
        },
        _ => { panic!("wrong payload")}
    }
}

#[test]
fn dual_message() {
	let mut bundle = Bundle::default();
	let mut message = Message::new("/hello");

	message.add_item(23_i32);

	bundle.add(message.clone());
	bundle.add(message);

	let data = Packet::Bundle(bundle);
	let buffer:Buffer = data.clone().try_into().expect("unable to pack buffer");

	assert!(buffer.is_valid());
	assert_eq!(buffer.len(), 56);

	let re_read:Packet = buffer.clone().try_into().expect("unable to pack struct");

	assert!(data.to_string().starts_with("|#bundle•||t:["));
	assert!(data.to_string().ends_with("]|M[|s:/hello••[8]||,:,i••[4]||i:23|]M[|s:/hello••[8]||,:,i••[4]||i:23|]"));

	assert!(re_read.to_string().starts_with("|#bundle•||t:["));
	assert!(re_read.to_string().ends_with("]|M[|s:/hello••[8]||,:,i••[4]||i:23|]M[|s:/hello••[8]||,:,i••[4]||i:23|]"));
	assert_eq!(re_read, data);
}


#[test]
fn nested_bundle_message() {
	let mut bundle = Bundle::default();
	let mut message = Message::new("/hello");

	message.add_item(23_i32);

	bundle.add(message.clone());

	let mut bundle2 = Bundle::default();

	bundle2.add(message);
	bundle.add(bundle2);

	let data = Packet::Bundle(bundle);
	let buffer:Buffer = data.clone().try_into().expect("unable to pack buffer");

	assert!(buffer.is_valid());
	assert_eq!(buffer.len(), 76);

	let re_read:Packet = buffer.clone().try_into().expect("unable to pack struct");

	assert_eq!(re_read, data);
}

#[test]
fn invalid_bundle_buffers() {
	//[0x23, 0x62, 0x75, 0x6e, 0x64, 0x6c, 0x65, 0x0]
	let malformed = Buffer::from(vec![0x0, 0x0, 0x0]);
	let wrong_start = Buffer::from(vec![0x62, 0x62, 0x62, 0x0]);
	let empty_packet = Buffer::from(vec![0x23, 0x62, 0x75, 0x6e, 0x64, 0x6c, 0x65, 0x0]);
	let truncated_msg = Buffer::from(vec![
		0x23, 0x62, 0x75, 0x6e, 0x64, 0x6c, 0x65, 0x0, // #bundle\0
		0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // t:[0,0]
		0x0, 0x0, 0x0, 0x4, // [size:4 bytes]
	]);
	let bad_msg = Buffer::from(vec![
		0x23, 0x62, 0x75, 0x6e, 0x64, 0x6c, 0x65, 0x0, // #bundle\0
		0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // t:[0,0]
		0x0, 0x0, 0x0, 0x4, // [size:4 bytes]
		0x1, 0x1, 0x1, 0x1, // [invalid string address (no null)]
	]);

	let malformed_bundle = Bundle::try_from(malformed.clone());
	let malformed_bundle_from:Result<Packet, _> = malformed.try_into();

	assert!(malformed_bundle_from.is_err());
	assert_eq!(malformed_bundle_from, Err(TypeError::MisalignedBuffer));

	assert!(malformed_bundle.is_err());
	assert_eq!(malformed_bundle, Err(TypeError::MisalignedBuffer));

	let wrong_start_bundle = Bundle::try_from(wrong_start);
	assert!(wrong_start_bundle.is_err());
	assert_eq!(wrong_start_bundle, Err(TypeError::InvalidPacket));

	let empty_packet_bundle = Bundle::try_from(empty_packet.clone());
	let empty_packet_from:Result<Packet, _> = empty_packet.try_into();

	assert!(empty_packet_bundle.is_err());
	assert_eq!(empty_packet_bundle, Err(TypeError::InvalidPacket));
	assert!(empty_packet_from.is_err());
	assert_eq!(empty_packet_from, Err(TypeError::InvalidPacket));

	let bad_msg_bundle = Bundle::try_from(bad_msg);
	assert!(bad_msg_bundle.is_err());
	assert_eq!(bad_msg_bundle, Err(TypeError::InvalidPacket));

	let truncated_msg_bundle = Bundle::try_from(truncated_msg);
	assert!(truncated_msg_bundle.is_err());
	assert_eq!(truncated_msg_bundle, Err(TypeError::InvalidPacket));
}