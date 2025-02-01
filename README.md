# X32_OSC_STATE

A state machine for X32 OSC communication. Includes an OSC encoder/decoder

## Getting data from the state machine

```rust
use x32_osc_state as x32;

let mut state:x32::X32Console = x32::X32Console::default();

assert_eq!(state.active_cue(), "Cue: 0.0.0 :: -- [--] [--]");

let channel_01_fader = state.fader(&x32::enums::FaderIndex::Channel(1)).expect("Unknown Channel");

assert_eq!(channel_01_fader.name(), "Ch01");
assert_eq!(channel_01_fader.level(), (0_f32, String::from("-oo dB")));
assert_eq!(channel_01_fader.is_on(), (false, String::from("OFF")));
```

## Communicating to the X32

Please see the examples folder for a very simple version of a self-updating X32 state machine.

```rust
use x32_osc_state as x32;

// Ask the X32 for the cue list along with the status of all tracked faders:
//   - main, mono, matrix, aux, bus, dca, and channels
let x32_initial_data:Vec<x32::osc::Buffer> = x32::x32::ConsoleRequest::full_update();

// contains the raw byte buffer for the xremote command
let xremote_command = x32::enums::X32_XREMOTE.clone();
```

## Process updates from the X32

```rust
use x32_osc_state as x32;

let mut state:x32::X32Console = x32::X32Console::default();

let mut raw_buffer = [0; 1024];

let buffer = x32::osc::Buffer::from(raw_buffer.clone().to_vec());

// process function will take [`osc::Buffer`] or [`osc::Message`] if you prefer
// to do some pre-processing. Messages that are malformed or not understood
// are silently ignored
let result:x32::X32ProcessResult = state.process(buffer);

// This is for acting on new data immediately.  The result can be
// safely ignored.  Most processed data is cached by the state
// machine with the notable exception of meter information - if
// you wish to use it, you must handle it here.
match result {
    x32::X32ProcessResult::NoOperation => (),
    x32::X32ProcessResult::Meters((meter_id_int, meter_vec_u8)) => (),
    x32::X32ProcessResult::Fader(fader) => (),
    x32::X32ProcessResult::CurrentCue(string) => (),
}
```
