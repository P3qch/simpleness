use modular_bitfield::prelude::*;

#[derive(Clone, Copy)]
#[bitfield(bits = 8)]
pub struct OAMSpriteAttributes {
    pub pallette: B2,
    pub unused: B3,
    pub priority: B1,
    pub flip_horizontal: B1,
    pub flip_vertical: B1,
}

#[derive(Clone, Copy)]
pub struct OAMSprite {
    y: u8,
    tile_index: u8,
    attributes: OAMSpriteAttributes,
    x: u8,
}

#[allow(dead_code)]
impl OAMSprite {
    pub fn new(y: u8, tile_index: u8, attributes_byte: u8, x: u8) -> Self {
        Self {
            y,
            tile_index,
            attributes: OAMSpriteAttributes::from_bytes([attributes_byte]),
            x,
        }
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        assert!(data.len() == 4, "OAMSprite data must be exactly 4 bytes");
        Self::new(data[0], data[1], data[2], data[3])
    }

    pub fn get_y(&self) -> u8 {
        self.y
    }

    pub fn get_tile_index(&self) -> u8 {
        self.tile_index
    }

    pub fn get_attributes(&self) -> OAMSpriteAttributes {
        self.attributes
    }

    pub fn get_x(&self) -> u8 {
        self.x
    }
}
