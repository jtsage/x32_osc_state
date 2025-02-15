use x32_osc_state::x32;
use x32_osc_state::osc::Buffer;
use x32_osc_state::enums::{FaderBank, FaderBankKey};

#[test]
fn enum_full_update() {
    let update = x32::ConsoleRequest::full_update();

    assert_eq!(update.len(), 147);

    // for (i, item) in update.iter().enumerate() {
    // 	println!("{i:03}\n---\n{item}\n\n");
    // }
}

#[test]
fn keep_alive() {
    let update:Vec<Buffer> = x32::ConsoleRequest::KeepAlive().into();

    assert_eq!(update.len(), 1);
    assert_eq!(update.get(0), Some(&Buffer::from(vec![0x2f, 0x78, 0x72, 0x65, 0x6d, 0x6f, 0x74, 0x65, 0x0, 0x0, 0x0, 0x0])));
}

#[test]
fn vor_output() {
    let f_bank = FaderBank::new();

    assert_eq!(f_bank.vor_bundle(&FaderBankKey::Main).len(), 2);
    assert_eq!(f_bank.vor_bundle(&FaderBankKey::Aux).len(), 8);
    assert_eq!(f_bank.vor_bundle(&FaderBankKey::Bus).len(),16);
    assert_eq!(f_bank.vor_bundle(&FaderBankKey::Matrix).len(), 6);
    assert_eq!(f_bank.vor_bundle(&FaderBankKey::Channel).len(), 32);
    assert_eq!(f_bank.vor_bundle(&FaderBankKey::Dca).len(), 8);
}
