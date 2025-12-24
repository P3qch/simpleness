use modular_bitfield::prelude::*;

#[bitfield(bits = 8)]
#[derive(Clone, Copy, Debug)]
pub struct JoypadState {
    pub a: B1,
    pub b: B1,
    pub select: B1,
    pub start: B1,
    pub up: B1,
    pub down: B1,
    pub left: B1,
    pub right: B1,
}

pub struct Joypad {
    pub state: JoypadState,
    shift_register_strobe: bool,
    current_read_number: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            state: JoypadState::new(),
            shift_register_strobe: false,
            current_read_number: 0,
        }
    }

    pub fn set_shift_register_strobe(&mut self, val: bool) {
        self.shift_register_strobe = val;
        self.current_read_number = 0;
    }

    /*
    Each read reports one bit at a time of the state
    */
    pub fn read_status(&mut self) -> u8 {
        if self.shift_register_strobe {
                self.current_read_number = 0;
        }

        if self.current_read_number < 8 {
            let a = (self.state.into_bytes()[0] >> self.current_read_number) & 1;
            self.current_read_number += 1;
            a
        } else {
            1
        }
    }
}
