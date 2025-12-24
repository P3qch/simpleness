use modular_bitfield::prelude::*;

#[bitfield(bits = 8)]
#[derive(Clone, Copy, Debug)]
pub struct PPUCtrl {
    pub base_nametable_address: B2, // base nametable address (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
    pub increment_mode: B1,         // increment mode (0: add 1; 1: add 32)
    pub sprite_pattern_table_address: B1, // sprite pattern table address for 8x8 sprites (0: $0000; 1: $1000; ignored in 8x16 mode)
    pub background_pattern_table_address: B1, // background pattern table address (0: $0000; 1: $1000)
    pub sprite_size: B1,                      // Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
    pub master_slave_select: B1, // master/slave select (0: read backdrop from EXT pins; 1: output color on EXT pins)
    pub vblank_nmi_enable: B1, // Vblank NMI enable (0: off, 1: on)generate NMI (0: no NMI; 1: generate NMI)
}

impl PPUCtrl {
    /**
     * Resolves the asctual nametable address using the base_nametable_address field
     */
    pub fn get_base_nametable_address(&self) -> u16 {
        match self.base_nametable_address() {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => unreachable!(),
        }
    }

    pub fn get_increment_value(&self) -> u16 {
        if self.increment_mode() == 0 { 1 } else { 32 }
    }

    pub fn get_background_pattern_table_address(&self) -> u16 {
        if self.background_pattern_table_address() == 0 {
            0x0000
        } else {
            0x1000
        }
    }

    pub fn get_sprite_pattern_table_address(&self) -> u16 {
        if self.sprite_pattern_table_address() == 0 {
            0x0000
        } else {
            0x1000
        }
    }

    pub fn get_sprite_height(&self) -> u8 {
        if self.sprite_size() == 1 { 16 } else { 8 }
    }
}
