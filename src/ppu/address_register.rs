pub struct AddressRegister {
    hibyte: u8,
    lobyte: u8,
}

impl AddressRegister {
    pub fn new() -> Self {
        Self {
            hibyte: 0,
            lobyte: 0,
        }
    }

    pub fn update(&mut self, value: u8, write_latch: &mut bool) {
        if !*write_latch {
            self.hibyte = value & 0x3f;
            *write_latch = true;
        } else {
            self.lobyte = value;
            *write_latch = false;
        }
    }

    pub fn get_address(&self) -> u16 {
        ((self.hibyte as u16) << 8) | (self.lobyte as u16)
    }

    pub fn increment(&mut self, increment: u16) {
        let addr = self.get_address().wrapping_add(increment);
        self.lobyte = addr as u8;
        self.hibyte = ((addr >> 8) & 0x3f) as u8;
    }
}
