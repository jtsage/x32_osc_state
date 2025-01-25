use x32_osc_state::x32;
use std::str::FromStr;

#[test]
fn string_to_fader_type() {
    assert_eq!(x32::FaderType::from_str("auxin"), Ok(x32::FaderType::Aux));
    assert_eq!(x32::FaderType::from_str("bus"), Ok(x32::FaderType::Bus));
    assert_eq!(x32::FaderType::from_str("ch"), Ok(x32::FaderType::Channel));
    assert_eq!(x32::FaderType::from_str("dca"), Ok(x32::FaderType::Dca));
    assert_eq!(x32::FaderType::from_str("main"), Ok(x32::FaderType::Main));
    assert_eq!(x32::FaderType::from_str("mtx"), Ok(x32::FaderType::Matrix));
    assert_eq!(x32::FaderType::from_str("other"), Err(x32::Error::InvalidFader));

    assert_eq!(x32::FaderType::Aux.to_string(), "auxin");
    assert_eq!(x32::FaderType::Bus.to_string(), "bus");
    assert_eq!(x32::FaderType::Channel.to_string(), "ch");
    assert_eq!(x32::FaderType::Dca.to_string(), "dca");
    assert_eq!(x32::FaderType::Main.to_string(), "main");
    assert_eq!(x32::FaderType::Matrix.to_string(), "mtx");
    assert_eq!(x32::FaderType::Unknown.to_string(), "");
}

#[test]
fn address_split() {
    let items_1 = x32::util::split_address("/howdy");
    let items_2 = x32::util::split_address("/howdy/ho");
    let items_3 = x32::util::split_address("/howdy/ho/neighbor");
    let items_4 = x32::util::split_address("/howdy/ho/neighbor/simpson");

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
fn check_bounds() {
    // Note - check bounds is ZERO based
    assert_eq!(x32::FaderType::Aux.check_bounds(8), false);
    assert_eq!(x32::FaderType::Channel.check_bounds(32), false);
    assert_eq!(x32::FaderType::Bus.check_bounds(16), false);
    assert_eq!(x32::FaderType::Dca.check_bounds(8), false);
    assert_eq!(x32::FaderType::Main.check_bounds(2), false);
    assert_eq!(x32::FaderType::Matrix.check_bounds(8), false);

    assert_eq!(x32::FaderType::Aux.check_bounds(7), true);
    assert_eq!(x32::FaderType::Channel.check_bounds(31), true);
    assert_eq!(x32::FaderType::Bus.check_bounds(15), true);
    assert_eq!(x32::FaderType::Dca.check_bounds(7), true);
    assert_eq!(x32::FaderType::Main.check_bounds(1), true);
    assert_eq!(x32::FaderType::Matrix.check_bounds(5), true);

    assert_eq!(x32::FaderType::Unknown.check_bounds(0), false);
    assert_eq!(x32::FaderType::Unknown.check_bounds(8), false);
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
        assert_eq!(x32::util::level_from_string(v.1), v.0, "{} -> {}", v.1, v.0);
        assert_eq!(x32::util::level_to_string(v.0), v.1, "{} -> {}", v.0, v.1);
    }
}