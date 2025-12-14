use modular_bitfield::prelude::*;

#[bitfield(bits = 8)]
#[derive(Clone, Copy, Debug)]
pub struct PPUStatus {
    ppu_identifier: B5,
    sprite_overflow: B1,
    sprite_zero_hit: B1,
    vblank: B1,
}

