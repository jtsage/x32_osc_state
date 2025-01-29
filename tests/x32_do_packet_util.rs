use x32_osc_state::x32::ConsoleMessage;
use x32_osc_state::osc::Buffer;
use x32_osc_state::enums::{Fader, FaderColor, FaderIndex};
use x32_osc_state::enums::{Error, X32Error};

#[test]
fn address_split() {
    let items_1 = ConsoleMessage::split_address("/howdy");
    let items_2 = ConsoleMessage::split_address("/howdy/ho");
    let items_3 = ConsoleMessage::split_address("/howdy/ho/neighbor");
    let items_4 = ConsoleMessage::split_address("/howdy/ho/neighbor/simpson");

    assert_eq!(items_1.0, "howdy");
    assert_eq!(items_1.1, "");
    assert_eq!(items_1.2, "");
    assert_eq!(items_1.3, "");

    assert_eq!(items_2.0, "howdy");
    assert_eq!(items_2.1, "ho");
    assert_eq!(items_2.2, "");
    assert_eq!(items_2.3, "");

    assert_eq!(items_3.0, "howdy");
    assert_eq!(items_3.1, "ho");
    assert_eq!(items_3.2, "neighbor");
    assert_eq!(items_3.3, "");

    assert_eq!(items_4.0, "howdy");
    assert_eq!(items_4.1, "ho");
    assert_eq!(items_4.2, "neighbor");
    assert_eq!(items_4.3, "simpson");
}


#[test]
fn check_level_conversion() {
    let known_value = [
        (0.0000, "-oo dB"),
        (0.0010, "-89.5 dB"),
        (0.0196, "-80.6 dB"),
        (0.0411, "-70.3 dB"),
        (0.0518, "-65.1 dB"),
        (0.0616, "-60.4 dB"),
        (0.0899, "-55.6 dB"),
        (0.1232, "-50.3 dB"),
        (0.1505, "-45.9 dB"),
        (0.1867, "-40.1 dB"),
        (0.2141, "-35.7 dB"),
        (0.2454, "-30.7 dB"),
        (0.3060, "-25.5 dB"),
        (0.3734, "-20.1 dB"),
        (0.4301, "-15.6 dB"),
        (0.4946, "-10.4 dB"),
        (0.6197, "-5.2 dB"),
        (0.7478, "-0.1 dB"),
        (0.7498, "+0.0 dB"),
        (0.7527, "+0.1 dB"),
        (0.7752, "+1.0 dB"),
        (0.7996, "+2.0 dB"),
        (0.8250, "+3.0 dB"),
        (0.8495, "+4.0 dB"),
        (0.8749, "+5.0 dB"),
        (0.9003, "+6.0 dB"),
        (0.9746, "+9.0 dB"),
        (1.0000, "+10.0 dB"),
    ];

    for v in known_value {
        assert_eq!(Fader::level_from_string(v.1), v.0, "{} -> {}", v.1, v.0);
        assert_eq!(Fader::level_to_string(v.0), v.1, "{} -> {}", v.0, v.1);
    }
}

#[test]
fn fader_color() {
    assert_eq!(FaderColor::parse_str("OFF"), FaderColor::Off);
    assert_eq!(FaderColor::parse_str("OFFi"), FaderColor::Off);
    assert_eq!(FaderColor::parse_str("RD"), FaderColor::Red);
    assert_eq!(FaderColor::parse_str("GN"), FaderColor::Green);
    assert_eq!(FaderColor::parse_str("YE"), FaderColor::Yellow);
    assert_eq!(FaderColor::parse_str("BL"), FaderColor::Blue);
    assert_eq!(FaderColor::parse_str("MG"), FaderColor::Magenta);
    assert_eq!(FaderColor::parse_str("CY"), FaderColor::Cyan);
    assert_eq!(FaderColor::parse_str("RDi"), FaderColor::RedInverted);
    assert_eq!(FaderColor::parse_str("GNi"), FaderColor::GreenInverted);
    assert_eq!(FaderColor::parse_str("YEi"), FaderColor::YellowInverted);
    assert_eq!(FaderColor::parse_str("BLi"), FaderColor::BlueInverted);
    assert_eq!(FaderColor::parse_str("MGi"), FaderColor::MagentaInverted);
    assert_eq!(FaderColor::parse_str("CYi"), FaderColor::CyanInverted);
    assert_eq!(FaderColor::parse_str("WHi"), FaderColor::WhiteInverted);
    assert_eq!(FaderColor::parse_str("WH"), FaderColor::White);

    assert_eq!(FaderColor::parse_int(1), FaderColor::Red);
    assert_eq!(FaderColor::parse_int(2), FaderColor::Green);
    assert_eq!(FaderColor::parse_int(3), FaderColor::Yellow);
    assert_eq!(FaderColor::parse_int(4), FaderColor::Blue);
    assert_eq!(FaderColor::parse_int(5), FaderColor::Magenta);
    assert_eq!(FaderColor::parse_int(6), FaderColor::Cyan);
    assert_eq!(FaderColor::parse_int(7), FaderColor::White);
    assert_eq!(FaderColor::parse_int(9), FaderColor::RedInverted);
    assert_eq!(FaderColor::parse_int(10), FaderColor::GreenInverted);
    assert_eq!(FaderColor::parse_int(11), FaderColor::YellowInverted);
    assert_eq!(FaderColor::parse_int(12), FaderColor::BlueInverted);
    assert_eq!(FaderColor::parse_int(13), FaderColor::MagentaInverted);
    assert_eq!(FaderColor::parse_int(14), FaderColor::CyanInverted);
    assert_eq!(FaderColor::parse_int(15), FaderColor::WhiteInverted);
    assert_eq!(FaderColor::parse_int(8), FaderColor::Off);
    assert_eq!(FaderColor::parse_int(0), FaderColor::Off);
}

#[test]
fn fader_index_stuff() {
    assert_eq!(FaderIndex::Main(1).get_vor_address(), "/main/01");
    assert_eq!(FaderIndex::Aux(1).get_vor_address(), "/auxin/01");

    assert_eq!(FaderIndex::Aux(1).default_label(), "Aux01");
    assert_eq!(FaderIndex::Bus(1).default_label(), "MixBus01");
    assert_eq!(FaderIndex::Dca(1).default_label(), "DCA1");
    assert_eq!(FaderIndex::Channel(1).default_label(), "Ch01");
    assert_eq!(FaderIndex::Matrix(1).default_label(), "Mtx01");
    assert_eq!(FaderIndex::Main(1).default_label(), "Main");
    assert_eq!(FaderIndex::Main(2).default_label(), "M/C");

    assert_eq!(FaderIndex::Unknown.default_label(), "");
    assert_eq!(FaderIndex::Unknown.get_x32_address(), "");
    assert_eq!(FaderIndex::Unknown.get_vor_address(), "/");
    assert_eq!(FaderIndex::Unknown.get_index(), 0);
    assert_eq!(FaderIndex::Unknown.get_x32_update(), vec![Buffer::default()]);

    let fake_fader = (String::from("boo"), String::from("01"));
    let fake_fader:Result<FaderIndex, _> = fake_fader.try_into();

    assert_eq!(fake_fader.unwrap_err(), Error::X32(X32Error::InvalidFader));

    let fake_fader = (String::from("boo"), 1_i32);
    let fake_fader:Result<FaderIndex, _> = fake_fader.try_into();

    assert_eq!(fake_fader.unwrap_err(), Error::X32(X32Error::InvalidFader));

    let fake_fader = (String::from("boo"), String::from("x"));
    let fake_fader:Result<FaderIndex, _> = fake_fader.try_into();

    assert_eq!(fake_fader.unwrap_err(), Error::X32(X32Error::InvalidFader));

    let fake_fader = (String::from("boo"), -1_i32);
    let fake_fader:Result<FaderIndex, _> = fake_fader.try_into();

    assert_eq!(fake_fader.unwrap_err(), Error::X32(X32Error::InvalidFader));
}