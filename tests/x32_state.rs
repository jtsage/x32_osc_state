use x32_osc_state::enums::{Fader, FaderIndex, FaderColor};
use x32_osc_state::osc;
use x32_osc_state::{X32ProcessResult, X32Console};

mod buffer_common;
use buffer_common::random_data_node;


fn make_node_message(s : &str) -> osc::Message {
    let mut msg = osc::Message::new("node");
    
    msg.add_item(s.to_owned());
    msg
}
#[test]
fn make_and_test_cues() {
    let mut state = X32Console::default();

    state.process(make_node_message("/-show/showfile/cue/000 100 \"Cue Idx0 Num100\" 1 1 0 0 1 0 0"));
    state.process(make_node_message("/-show/showfile/cue/001 110 \"Cue Idx1 Num110\" 1 2 -1 0 1 0 0"));
    state.process(make_node_message("/-show/showfile/cue/002 200 \"Cue Idx2 BadSceneSnip\" 1 5 5 0 1 0 0"));

    state.process(make_node_message("/-show/showfile/scene/001 \"SceneAAA\" \"aaa\" %111111110 1"));
    state.process(make_node_message("/-show/showfile/scene/002 \"SceneBBB\" \"aaa\" %111111110 1"));

    let result = state.process(make_node_message("/-show/showfile/snippet/000 \"Snip-001\" 1 1 0 32768 1 "));
    assert_eq!(result, X32ProcessResult::NoOperation);

    assert_eq!(state.cue_list_size(), (3,2,1));

    assert_eq!(state.active_cue(), "Cue: 0.0.0 :: -- [--] [--]");
    state.process(make_node_message("/-show/prepos/current 0"));
    assert_eq!(state.active_cue(), "Cue: 1.0.0 :: Cue Idx0 Num100 [01:SceneAAA] [00:Snip-001]");
    state.process(make_node_message("/-show/prepos/current 1"));
    assert_eq!(state.active_cue(), "Cue: 1.1.0 :: Cue Idx1 Num110 [02:SceneBBB] [--]");
    state.process(make_node_message("/-show/prepos/current 2"));
    assert_eq!(state.active_cue(), "Cue: 2.0.0 :: Cue Idx2 BadSceneSnip [--] [--]");
    state.process(make_node_message("/-show/prepos/current 3"));
    
    assert_eq!(state.active_cue(), "Cue: 0.0.0 :: -- [--] [--]");

    let result = state.process(make_node_message("/-show/prepos/current 0"));
    assert!(matches!(result, X32ProcessResult::CurrentCue(_)));

    state.process(make_node_message("/-prefs/show_control SNIPPETS"));
    assert_eq!(state.active_cue(), "Snippet: 00:Snip-001");
    state.process(make_node_message("/-show/prepos/current 1"));
    assert_eq!(state.active_cue(), "Snippet: --");

    state.process(make_node_message("/-show/prepos/current 1"));
    state.process(make_node_message("/-prefs/show_control SCENES"));
    assert_eq!(state.active_cue(), "Scene: 01:SceneAAA");
    state.process(make_node_message("/-show/prepos/current -1"));
    assert_eq!(state.active_cue(), "Scene: --");
}

fn make_fader_messages(f : &str, i : usize, v :(f32, bool, String)) -> [osc::Message;2] {
    let mix = format!("/{f}/{i:02}/mix {}   {:.1} OFF +0 OFF   -oo", if v.1 { "ON" } else { "OFF" } , v.0);
    let name = format!("/{f}/{i:02}/config \"{}\" 1 RD 33", v.2);

    [make_node_message(mix.as_str()), make_node_message(name.as_str())]
}

#[test]
fn make_and_test_faders() {
    let mut state = X32Console::default();

    let dca = random_data_node();
    let bus = random_data_node();
    let main = random_data_node();
    let mtx = random_data_node();
    let channel = random_data_node();
    let aux = random_data_node();

    make_fader_messages("auxin", 2, aux.clone()).iter().for_each(|item|{ state.process(item.clone()); });
    make_fader_messages("bus", 8, bus.clone()).iter().for_each(|item|{ state.process(item.clone()); });
    make_fader_messages("mtx", 4, mtx.clone()).iter().for_each(|item|{ state.process(item.clone()); });
    make_fader_messages("ch", 23, channel.clone()).iter().for_each(|item|{ state.process(item.clone()); });
    make_fader_messages("main", 1, main.clone()).iter().for_each(|item|{ state.process(item.clone()); });
    make_fader_messages("dca", 3, dca.clone()).iter().for_each(|item|{ state.process(item.clone()); });

    let aux_fader = state.fader(&FaderIndex::Aux(2)).expect("invalid fader");

    assert_eq!(aux_fader.name(), aux.2);
    assert_eq!(aux_fader.level().0, Fader::level_from_string(&format!("{}", aux.0)));
    assert_eq!(aux_fader.is_on().0, aux.1);
    assert_eq!(aux_fader.color(), FaderColor::Red);

    let bus_fader = state.fader(&FaderIndex::Bus(8)).expect("invalid fader");

    assert_eq!(bus_fader.name(), bus.2);
    assert_eq!(bus_fader.level().0, Fader::level_from_string(&format!("{}", bus.0)));
    assert_eq!(bus_fader.is_on().0, bus.1);

    let mtx_fader = state.fader(&FaderIndex::Matrix(4)).expect("invalid fader");

    assert_eq!(mtx_fader.name(), mtx.2);
    assert_eq!(mtx_fader.level().0, Fader::level_from_string(&format!("{}", mtx.0)));
    assert_eq!(mtx_fader.is_on().0, mtx.1);

    let chan_fader = state.fader(&FaderIndex::Channel(23)).expect("invalid fader");

    assert_eq!(chan_fader.name(), channel.2);
    assert_eq!(chan_fader.level().0, Fader::level_from_string(&format!("{}", channel.0)));
    assert_eq!(chan_fader.is_on().0, channel.1);

    let main_fader = state.fader(&FaderIndex::Main(1)).expect("invalid fader");

    assert_eq!(main_fader.name(), main.2);
    assert_eq!(main_fader.level().0, Fader::level_from_string(&format!("{}", main.0)));
    assert_eq!(main_fader.is_on().0, main.1);

    let dca_fader = state.fader(&FaderIndex::Dca(3)).expect("invalid fader");

    assert_eq!(dca_fader.name(), dca.2);
    assert_eq!(dca_fader.level().0, Fader::level_from_string(&format!("{}", dca.0)));
    assert_eq!(dca_fader.is_on().0, dca.1);

    state.reset();

    let dca_fader = state.fader(&FaderIndex::Dca(3)).expect("invalid fader");

    assert_eq!(dca_fader.name(), "DCA3");
    assert_eq!(dca_fader.level().0, 0_f32);
    assert_eq!(dca_fader.is_on().0, false);

    let msg1 = make_fader_messages("bus", 2, bus);
    let result = state.process(msg1[0].clone());
    assert!(matches!(result, X32ProcessResult::Fader(_)));
}

#[test]
fn meter_test() {
    let mut state = X32Console::default();
    let float_original:[f32; 5] = [4.5, 1.0, 0.0, 0.5, 0.75];

    let mut buffer_msg = osc::Message::new("/meters/0");
    let float_packed = float_original
        .map(|f| f.to_le_bytes())
        .iter()
        .flat_map(|u| *u)
        .collect::<Vec<u8>>();

    buffer_msg.add_item(osc::Type::Blob(float_packed));

    let result = state.process(buffer_msg);
    let expected = X32ProcessResult::Meters((0, float_original.clone().to_vec()));
    assert_eq!(result, expected);

    let mut buffer_msg = osc::Message::new("/meters/0");
    buffer_msg.add_item(String::from("bad type"));
    let result = state.process(buffer_msg);
    assert_eq!(result, X32ProcessResult::NoOperation);
}