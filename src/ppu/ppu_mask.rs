use modular_bitfield::prelude::*;

#[bitfield(bits = 8)]
#[derive(Clone, Copy, Debug)]
pub struct PPUMask {
    grayscale: B1, // Greyscale (0: normal color, 1: greyscale)
    show_background_left: B1, // Show background in leftmost 8 pixels (0: off; 1: on)
    show_sprites_left: B1, // Show sprites in leftmost 8 pixels (0: off; 1: on)
    show_background: B1, // Show background (0: off; 1: on)
    show_sprites: B1, // Show sprites (0: off; 1: on)
    emphasize_red: B1, // Emphasize red (0: off; 1: on)
    emphasize_green: B1, // Emphasize green (0: off; 1: on)
    emphasize_blue: B1, // Emphasize blue (0: off; 1: on)
}