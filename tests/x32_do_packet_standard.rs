use x32_osc_state::x32;
use x32_osc_state::osc;

mod buffer_common;
use buffer_common::random_data;

fn fader_level_test(fader: &str, index: usize, level: f32) {
	level_test(&format!("/{fader}/{index:02}/mix/fader"), fader, index, level);
}
fn fader_mute_test(fader: &str, index: usize, mute: bool) {
	mute_test(&format!("/{fader}/{index:02}/mix/on"), fader, index, mute);
}

fn level_test(address: &str, fader: &str, index: usize, level: f32) {
	let mut msg = osc::Message::new(&address);

	msg.add_item(level);

	let expected = x32::FaderUpdate{
		source: fader.parse::<x32::FaderType>().unwrap_or_default(),
		index : index-1,
		level: Some(level),
		..Default::default()
	};
	let update = x32::ConsoleMessage::try_from(msg);
	assert_eq!(update, Ok(x32::ConsoleMessage::Fader(expected)));
}

fn mute_test(address: &str, fader: &str, index: usize, is_on: bool) {
	let mut msg = osc::Message::new(&address);

	msg.add_item(is_on as i32);

	let expected = x32::FaderUpdate{
		source: fader.parse::<x32::FaderType>().unwrap_or_default(),
		index : index - 1,
		is_on: Some(is_on),
		..Default::default()
	};

	let update = x32::ConsoleMessage::try_from(msg);
	assert_eq!(update, Ok(x32::ConsoleMessage::Fader(expected)));
}

fn name_test(index_str: &str, fader: &str, index: usize, name : &str) {
	let address = &format!("/{fader}/{index_str}/config/name");
	let mut msg = osc::Message::new(&address);

	msg.add_item(name.to_owned());

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
		let rand_data = random_data();
		
		if i == 1 {
			level_test("/main/st/mix/fader", "main", i, rand_data.0);
		}

		if i == 2 {
			level_test("/main/m/mix/fader", "main", i, rand_data.0);
		}

		if i <= 6 {
			fader_level_test("mtx", i, rand_data.0);
		}

		if i < 8 {
			fader_level_test("auxin", i, rand_data.0);
			level_test(&format!("/dca/{i}/fader"), "dca", i, rand_data.0);
		}

		if i <= 16 {
			fader_level_test("bus", i, rand_data.0);
		}

		if i <= 32 {
			fader_level_test("ch", i, rand_data.0);
		}
	}
}



#[test]
fn fader_mute() {
	for i in 1..32 {
		let rand_data = random_data();
		
		if i == 1 {
			mute_test("/main/st/mix/on", "main", i, rand_data.1);
		}

		if i == 2 {
			mute_test("/main/m/mix/on", "main", i, rand_data.1);
		}

		if i <= 6 {
			fader_mute_test("mtx", i, rand_data.1);
		}

		if i < 8 {
			fader_mute_test("auxin", i, rand_data.1);
			mute_test(&format!("/dca/{i}/on"), "dca", i, rand_data.1);
		}

		if i <= 16 {
			fader_mute_test("bus", i, rand_data.1);
		}

		if i <= 32 {
			fader_mute_test("ch", i, rand_data.1);
		}
	}
}


#[test]
fn fader_name() {
	for i in 1..32 {
		let rand_data = random_data();
		
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
	let msg = osc::Message::new("/-show/prepos/current");

	let mut msg_1 = msg.clone();
	msg_1.add_item(1_i32);

	let update = x32::ConsoleMessage::try_from(msg_1);
	assert_eq!(update, Ok(x32::ConsoleMessage::CurrentCue(1)));

	let mut msg_2 = msg.clone();
	msg_2.add_item(32.5_f32);

	let update = x32::ConsoleMessage::try_from(msg_2);
	assert_eq!(update, Ok(x32::ConsoleMessage::CurrentCue(-1)));
}

#[test]
fn show_mode() {
	let msg = osc::Message::new("/-prefs/show_control");

	let mut msg_1 = msg.clone();
	msg_1.add_item(1_i32);

	let update = x32::ConsoleMessage::try_from(msg_1);
	assert_eq!(update, Ok(x32::ConsoleMessage::ShowMode(x32::ShowMode::Scenes)));

	let mut msg_2 = msg.clone();
	msg_2.add_item(2_i32);

	let update = x32::ConsoleMessage::try_from(msg_2);
	assert_eq!(update, Ok(x32::ConsoleMessage::ShowMode(x32::ShowMode::Snippets)));

	let mut msg_0 = msg.clone();
	msg_0.add_item(0_i32);

	let update = x32::ConsoleMessage::try_from(msg_0);
	assert_eq!(update, Ok(x32::ConsoleMessage::ShowMode(x32::ShowMode::Cues)));

	let mut msg_7 = msg.clone();
	msg_7.add_item(7_i32);

	let update = x32::ConsoleMessage::try_from(msg_7);
	assert_eq!(update, Ok(x32::ConsoleMessage::ShowMode(x32::ShowMode::Cues)));
}

#[test]
fn unhandled_message() {
	let msg = osc::Message::new("/dca/2/config/icon");

	let result = x32::ConsoleMessage::try_from(msg);

	assert!(result.is_err());
	assert_eq!(result, Err(x32::Error::UnimplementedPacket));
}

#[test]
fn invalid_faders() {
	let level = osc::Message::new("/auxin/09/mix/fader");
	let mute = osc::Message::new("/ch/36/mix/on");
	let name = osc::Message::new("/dca/9/config/name");
	
	let u_level = x32::ConsoleMessage::try_from(level);
	let u_mute = x32::ConsoleMessage::try_from(mute);
	let u_name = x32::ConsoleMessage::try_from(name);

	assert!(u_level.is_err());
	assert_eq!(u_level, Err(x32::Error::InvalidFader));
	assert!(u_mute.is_err());
	assert_eq!(u_mute, Err(x32::Error::InvalidFader));
	assert!(u_name.is_err());
	assert_eq!(u_name, Err(x32::Error::InvalidFader));
}