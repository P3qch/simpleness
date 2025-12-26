use modular_bitfield::prelude::*;
use std::fmt::Debug;

#[derive(Clone, Copy)]
#[bitfield(bits = 8)]
pub struct OAMSpriteAttributes {
    pub pallette: B2,
    pub unused: B3,
    pub priority: B1,
    pub flip_horizontal: B1,
    pub flip_vertical: B1,
}

impl Debug for OAMSpriteAttributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OAMSpriteAttributes")
            .field("pallette", &self.pallette())
            .field("priority", &self.priority())
            .field("flip_horizontal", &self.flip_horizontal())
            .field("flip_vertical", &self.flip_vertical())
            
            .finish()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OAMSprite {
    y: u8,
    tile_index: u8,
    attributes: OAMSpriteAttributes,
    x: u8,

    is_sprite_0: bool,
}

#[allow(dead_code)]
impl OAMSprite {
    pub fn new(y: u8, tile_index: u8, attributes_byte: u8, x: u8, is_sprite_0: bool) -> Self {
        Self {
            y,
            tile_index,
            attributes: OAMSpriteAttributes::from_bytes([attributes_byte]),
            x,
            is_sprite_0,
        }
    }

    pub fn from_bytes(data: &[u8], is_sprite_0: bool) -> Self {
        assert!(data.len() == 4, "OAMSprite data must be exactly 4 bytes");
        Self::new(data[0], data[1], data[2], data[3], is_sprite_0)
    }

    pub fn is_sprite_0(&self) -> bool {
        self.is_sprite_0
    }

    pub fn get_y(&self) -> u8 {
        self.y
    }

    pub fn get_rendered_y(&self) -> u16 {
        self.y as u16 + 1
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
