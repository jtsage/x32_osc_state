use x32_osc_state::x32;
use x32_osc_state::osc;

mod buffer_common;
use buffer_common::random_data_node;

fn fader_level_mute_test(fader: &str, index: usize, level: f32, is_on: bool) {
	level_mute_test(&format!("/{fader}/{index:02}/mix"), fader, index, level, is_on);
}

fn level_mute_test(address: &str, fader: &str, index: usize, level: f32, is_on: bool) {
	let mut msg = osc::Message::new("node");
	let arg_1 = format!("{address} {}   {level:.1} OFF +0 OFF   -oo", if is_on { "ON" } else { "OFF" });

	msg.add_item(arg_1);

	let expected = x32::FaderUpdate{
		source: fader.parse::<x32::FaderType>().unwrap_or_default(),
		index : index-1,
		level: Some(x32::util::level_from_string(&format!("{level}"))),
		is_on : Some(is_on),
		..Default::default()
	};
	let update = x32::ConsoleMessage::try_from(msg);
	assert_eq!(update, Ok(x32::ConsoleMessage::Fader(expected)));
}


fn name_test(index_str: &str, fader: &str, index: usize, name : &str) {
	let address = format!("/{fader}/{index_str}/config \"{name}\" 1 RD 33");
	let mut msg = osc::Message::new("node");

	msg.add_item(address.to_owned());

	let expected = x32::FaderUpdate{
		source: fader.parse::<x32::FaderType>().unwrap_or_default(),
		index : index - 1,
		label: Some(name.to_owned()),
		..Default::default()
	};

	let update = x32::ConsoleMessage::try_from(msg);
	assert_eq!(update, Ok(x32::ConsoleMessage::Fader(expected)));
}

#[test]
fn fader_level() {
	for i in 1..32 {
		let rand_data = random_data_node();
		
		if i == 1 {
			level_mute_test("/main/st/mix", "main", i, rand_data.0, rand_data.1);
		}

		if i == 2 {
			level_mute_test("/main/m/mix", "main", i, rand_data.0, rand_data.1);
		}

		if i <= 6 {
			fader_level_mute_test("mtx", i, rand_data.0, rand_data.1);
		}

		if i < 8 {
			fader_level_mute_test("auxin", i, rand_data.0, rand_data.1);
			level_mute_test(&format!("/dca/{i}/mix"), "dca", i, rand_data.0, rand_data.1);
		}

		if i <= 16 {
			fader_level_mute_test("bus", i, rand_data.0, rand_data.1);
		}

		if i <= 32 {
			fader_level_mute_test("ch", i, rand_data.0, rand_data.1);
		}
	}
}


#[test]
fn fader_name() {
	for i in 1..32 {
		let rand_data = random_data_node();
		
		if i == 1 {
			name_test("st", "main", i, rand_data.2.as_str());
		}

		if i == 2 {
			name_test("m", "main", i, rand_data.2.as_str());
		}

		if i <= 6 {
			name_test(&format!("{i:02}"), "mtx", i, rand_data.2.as_str());
		}

		if i < 8 {
			name_test(&format!("{i:02}"), "auxin", i, rand_data.2.as_str());
			name_test(&format!("{i}"), "dca", i, rand_data.2.as_str());
		}

		if i <= 16 {
			name_test(&format!("{i:02}"), "bus", i, rand_data.2.as_str());
		}

		if i <= 32 {
			name_test(&format!("{i:02}"), "ch", i, rand_data.2.as_str());
		}
	}
}


#[test]
fn cue_position() {
	let msg = osc::Message::new("node");

	let mut msg_1 = msg.clone();
	msg_1.add_item(String::from("/-show/prepos/current 1"));

	let update = x32::ConsoleMessage::try_from(msg_1);
	assert_eq!(update, Ok(x32::ConsoleMessage::CurrentCue(1)));

	let mut msg_2 = msg.clone();
	msg_2.add_item(String::from("/-show/prepos/current -1"));

	let update = x32::ConsoleMessage::try_from(msg_2);
	assert_eq!(update, Ok(x32::ConsoleMessage::CurrentCue(-1)));
}

#[test]
fn show_mode() {
	let msg = osc::Message::new("node");

	let mut msg_1 = msg.clone();
	msg_1.add_item(String::from("/-prefs/show_control SCENES"));

	let buffer:osc::Buffer = msg_1.try_into().expect("unable to pack buffer");
	let update = x32::ConsoleMessage::try_from(buffer);
	assert_eq!(update, Ok(x32::ConsoleMessage::ShowMode(x32::ShowMode::Scenes)));

	let mut msg_2 = msg.clone();
	msg_2.add_item(String::from("/-prefs/show_control SNIPPETS"));

	let update = x32::ConsoleMessage::try_from(msg_2);
	assert_eq!(update, Ok(x32::ConsoleMessage::ShowMode(x32::ShowMode::Snippets)));

	let mut msg_0 = msg.clone();
	msg_0.add_item(String::from("/-prefs/show_control CUES"));

	let update = x32::ConsoleMessage::try_from(msg_0);
	assert_eq!(update, Ok(x32::ConsoleMessage::ShowMode(x32::ShowMode::Cues)));

	let mut msg_7 = msg.clone();
	msg_7.add_item(String::from("/-prefs/show_control GARBAGE"));

	let update = x32::ConsoleMessage::try_from(msg_7);
	assert_eq!(update, Ok(x32::ConsoleMessage::ShowMode(x32::ShowMode::Cues)));
}

#[test]
fn unhandled_message() {
	let mut msg = osc::Message::new("node");

	msg.add_item(String::from("/dca/2/config/icon"));

	let result = x32::ConsoleMessage::try_from(msg);

	assert!(result.is_err());
	assert_eq!(result, Err(x32::Error::UnimplementedPacket));
}

#[test]
fn invalid_message() {
	let msg = osc::Message::new("node");

	let result = x32::ConsoleMessage::try_from(msg);

	assert!(result.is_err());
	assert_eq!(result, Err(x32::Error::MalformedPacket));

	let buffer = osc::Buffer::from(vec![0x0, 0x0]);
	let result = x32::ConsoleMessage::try_from(buffer);
	assert!(result.is_err());
	assert_eq!(result, Err(x32::Error::MalformedPacket));
}

#[test]
fn read_cue() {
	let msg = osc::Message::new("node");

	let mut msg_1 = msg.clone();
	msg_1.add_item(String::from("/-show/showfile/cue/000 1200 \"Cue Idx0 Num1200\" 1 1 -1 0 1 0 0"));

	let update = x32::ConsoleMessage::try_from(msg_1);
	assert_eq!(update, Ok(x32::ConsoleMessage::Cue(x32::CueUpdate {
		index: 0,
		cue_number: String::from("12.0.0"),
		name: String::from("Cue Idx0 Num1200"),
		snippet: None,
		scene: Some(1)
	})));
}


#[test]
fn read_cue_2() {
	let msg = osc::Message::new("node");

	let mut msg_1 = msg.clone();
	msg_1.add_item(String::from("/-show/showfile/cue/001 100 \"Cue with snip\" 1 -1 23 0 1 0 0"));

	let update = x32::ConsoleMessage::try_from(msg_1);
	assert_eq!(update, Ok(x32::ConsoleMessage::Cue(x32::CueUpdate {
		index: 1,
		cue_number: String::from("1.0.0"),
		name: String::from("Cue with snip"),
		snippet: Some(23),
		scene: None
	})));
}

#[test]
fn read_scene() {
	let msg = osc::Message::new("node");

	let mut msg_1 = msg.clone();
	msg_1.add_item(String::from("/-show/showfile/scene/001 \"AAA\" \"aaa\" %111111110 1"));

	let update = x32::ConsoleMessage::try_from(msg_1);
	assert_eq!(update, Ok(x32::ConsoleMessage::Scene(x32::SceneUpdate {
		index: 1,
		name: String::from("AAA"),
	})));
}

#[test]
fn read_snippet() {
	let msg = osc::Message::new("node");

	let mut msg_1 = msg.clone();
	msg_1.add_item(String::from("/-show/showfile/snippet/030 \"Aaa\" 1 1 0 32768 1 "));

	let update = x32::ConsoleMessage::try_from(msg_1);
	assert_eq!(update, Ok(x32::ConsoleMessage::Snippet(x32::SnippetUpdate {
		index: 30,
		name: String::from("Aaa"),
	})));
}
