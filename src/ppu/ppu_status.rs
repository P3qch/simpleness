use modular_bitfield::prelude::*;

#[bitfield(bits = 8)]
#[derive(Clone, Copy, Debug)]
pub struct PPUStatus {
    pub ppu_identifier: B5,
    pub sprite_overflow: B1,
    pub sprite_zero_hit: B1,
    pub vblank: B1,
}

