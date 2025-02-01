use x32_osc_state::enums::{FaderIndex, Fader};

#[test]
fn fader_index() {
	assert_eq!(serde_json::to_string(&FaderIndex::Aux(1)).unwrap(), "{\"index\":1,\"type\":\"aux\",\"name\":\"Aux01\"}");
	assert_eq!(serde_json::to_string(&FaderIndex::Main(1)).unwrap(), "{\"index\":1,\"type\":\"main\",\"name\":\"Main\"}");
	assert_eq!(serde_json::to_string(&FaderIndex::Matrix(1)).unwrap(), "{\"index\":1,\"type\":\"matrix\",\"name\":\"Mtx01\"}");
	assert_eq!(serde_json::to_string(&FaderIndex::Channel(1)).unwrap(), "{\"index\":1,\"type\":\"channel\",\"name\":\"Ch01\"}");
	assert_eq!(serde_json::to_string(&FaderIndex::Bus(1)).unwrap(), "{\"index\":1,\"type\":\"bus\",\"name\":\"MixBus01\"}");
	assert_eq!(serde_json::to_string(&FaderIndex::Dca(1)).unwrap(), "{\"index\":1,\"type\":\"dca\",\"name\":\"DCA1\"}");
	assert_eq!(serde_json::to_string(&FaderIndex::Unknown).unwrap(), "{\"index\":0,\"type\":\"unknown\",\"name\":\"\"}");
}

#[test]
fn fader() {
	let fader = Fader::new(FaderIndex::Channel(22));

	assert_eq!(serde_json::to_string(&fader).unwrap(), "{\"source\":{\"index\":22,\"type\":\"channel\",\"name\":\"Ch22\"},\"color\":\"White\",\"level\":\"-oo dB\",\"is_on\":false,\"label\":\"\"}");
}