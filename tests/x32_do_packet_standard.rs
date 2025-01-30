use x32_osc_state::x32;
use x32_osc_state::osc;
use x32_osc_state::enums::{ShowMode, FaderIndex};
use x32_osc_state::enums::{Error, X32Error};

mod buffer_common;
use buffer_common::random_data;

fn level_test(fader: FaderIndex, level: f32) {
    let mix_str = match fader {
        FaderIndex::Dca(_) => "fader",
        _ => "mix/fader",
    };

    let mut msg = osc::Message::new(&format!("/{}/{mix_str}", fader.get_x32_address()));

    msg.add_item(level);

    let expected = x32::updates::FaderUpdate{
        source: fader,
        level: Some(level),
        ..Default::default()
    };
    let update = x32::ConsoleMessage::try_from(msg);
    assert_eq!(update, Ok(x32::ConsoleMessage::Fader(expected)));
}

fn mute_test(fader: FaderIndex, is_on: bool) {
    let mix_str = match fader {
        FaderIndex::Dca(_) => "on",
        _ => "mix/on",
    };
    let mut msg = osc::Message::new(&format!("/{}/{mix_str}", fader.get_x32_address()));

    msg.add_item(is_on as i32);

    let expected = x32::updates::FaderUpdate{
        source: fader,
        is_on: Some(is_on),
        ..Default::default()
    };

    let update = x32::ConsoleMessage::try_from(msg);
    assert_eq!(update, Ok(x32::ConsoleMessage::Fader(expected)));
}

fn name_test(fader: FaderIndex, name : &str) {
    let address = &format!("/{}/config/name", fader.get_x32_address());
    let mut msg = osc::Message::new(&address);

    msg.add_item(name.to_owned());

    let expected = x32::updates::FaderUpdate{
        source: fader,
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
        
        if i <=2 {
            level_test(FaderIndex::Main(i), rand_data.0);
        }

        if i <= 6 {
            level_test(FaderIndex::Matrix(i), rand_data.0);
        }

        if i < 8 {
            level_test(FaderIndex::Dca(i), rand_data.0);
            level_test(FaderIndex::Aux(i), rand_data.0);
        }

        if i <= 16 {
            level_test(FaderIndex::Bus(i), rand_data.0);
        }

        if i <= 32 {
            level_test(FaderIndex::Channel(i), rand_data.0);
        }
    }
}


#[test]
fn fader_mute() {
    for i in 1..32 {
        let rand_data = random_data();
        
        if i <= 2 {
            mute_test(FaderIndex::Main(i), rand_data.1);
        }

        if i <= 6 {
            mute_test(FaderIndex::Matrix(i), rand_data.1);
        }

        if i < 8 {
            mute_test(FaderIndex::Dca(i), rand_data.1);
            mute_test(FaderIndex::Aux(i), rand_data.1);
        }

        if i <= 16 {
            mute_test(FaderIndex::Bus(i), rand_data.1);
        }

        if i <= 32 {
            mute_test(FaderIndex::Channel(i), rand_data.1);
        }
    }
}


#[test]
fn fader_name() {
    for i in 1..32 {
        let rand_data = random_data();
        
        if i <= 2 {
            name_test(FaderIndex::Main(i), rand_data.2.as_str());
        }

        if i <= 6 {
            name_test(FaderIndex::Matrix(i), rand_data.2.as_str());
        }

        if i < 8 {
            name_test(FaderIndex::Dca(i), rand_data.2.as_str());
            name_test(FaderIndex::Aux(i), rand_data.2.as_str());
        }

        if i <= 16 {
            name_test(FaderIndex::Bus(i), rand_data.2.as_str());
        }

        if i <= 32 {
            name_test(FaderIndex::Channel(i), rand_data.2.as_str());
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
    assert_eq!(update, Ok(x32::ConsoleMessage::ShowMode(ShowMode::Scenes)));

    let mut msg_2 = msg.clone();
    msg_2.add_item(2_i32);

    let update = x32::ConsoleMessage::try_from(msg_2);
    assert_eq!(update, Ok(x32::ConsoleMessage::ShowMode(ShowMode::Snippets)));

    let mut msg_0 = msg.clone();
    msg_0.add_item(0_i32);

    let update = x32::ConsoleMessage::try_from(msg_0);
    assert_eq!(update, Ok(x32::ConsoleMessage::ShowMode(ShowMode::Cues)));

    let mut msg_7 = msg.clone();
    msg_7.add_item(7_i32);

    let update = x32::ConsoleMessage::try_from(msg_7);
    assert_eq!(update, Ok(x32::ConsoleMessage::ShowMode(ShowMode::Cues)));
}

#[test]
fn unhandled_message() {
    let msg = osc::Message::new("/dca/2/config/icon");

    let result = x32::ConsoleMessage::try_from(msg);

    assert!(result.is_err());
    assert_eq!(result, Err(Error::X32(X32Error::UnimplementedPacket)));
}

#[test]
fn color_message() {
    let mut msg = osc::Message::new("/dca/2/config/color");

    msg.add_item(1_i32);

    let result = x32::ConsoleMessage::try_from(msg);

    assert!(matches!(result, Ok(x32::ConsoleMessage::Fader(_))));
}

#[test]
fn invalid_faders() {
    let level = osc::Message::new("/auxin/09/mix/fader");
    let mute = osc::Message::new("/ch/36/mix/on");
    let name = osc::Message::new("/dca/9/config/name");
    let color = osc::Message::new("/mtx/8/config/color");
    
    let u_level = x32::ConsoleMessage::try_from(level);
    let u_mute = x32::ConsoleMessage::try_from(mute);
    let u_name = x32::ConsoleMessage::try_from(name);
    let u_color = x32::ConsoleMessage::try_from(color);

    assert_eq!(u_level, Err(Error::X32(X32Error::InvalidFader)));
    assert_eq!(u_mute, Err(Error::X32(X32Error::InvalidFader)));
    assert_eq!(u_name, Err(Error::X32(X32Error::InvalidFader)));
    assert_eq!(u_color, Err(Error::X32(X32Error::InvalidFader)));
}