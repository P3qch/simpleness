use modular_bitfield::prelude::*;

#[bitfield(bits = 8)]
#[derive(Clone, Copy, Debug)]
pub struct PPUCtrl {
    base_nametable_address: B2, // base nametable address (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
    increment_mode: B1, // increment mode (0: add 1; 1: add 32)
    sprite_pattern_table_address: B1, // sprite pattern table address for 8x8 sprites (0: $0000; 1: $1000; ignored in 8x16 mode)
    background_pattern_table_address: B1, // background pattern table address (0: $0000; 1: $1000)
    sprite_size: B1, // Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
    master_slave_select: B1, // master/slave select (0: read backdrop from EXT pins; 1: output color on EXT pins)
    vblank_nmi_enable: B1, // Vblank NMI enable (0: off, 1: on)generate NMI (0: no NMI; 1: generate NMI)
}