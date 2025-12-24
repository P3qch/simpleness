mod address_register;
mod oam_sprite;
mod ppu_bus;
mod ppu_ctrl;
mod ppu_mask;
mod ppu_status;

use std::{
    io::{BufReader, Cursor, Read},
    vec,
};

use address_register::AddressRegister;
use ppu_bus::PPUBus;
use ppu_ctrl::PPUCtrl;
use ppu_mask::PPUMask;
use ppu_status::PPUStatus;

use crate::{memory::mapper::SharedMapper, ppu::oam_sprite::OAMSprite};

const PPUCTRL: u16 = 0x2000;
const PPUMASK: u16 = 0x2001;
const PPUSTATUS: u16 = 0x2002;
const OAMADDR: u16 = 0x2003;
const OAMDATA: u16 = 0x2004;
const PPUSCROLL: u16 = 0x2005;
const PPUADDR: u16 = 0x2006;
const PPUDATA: u16 = 0x2007;
pub const OAMDMA: u16 = 0x4014;

const PALLETTE_TABLE_START: u16 = 0x3F00;
const ATTRIBUTE_TABLE_OFFSET: u16 = 0x03C0;

const COLORS: [(u8, u8, u8); 64] = [
    (84, 84, 84),
    (0, 30, 116),
    (8, 16, 144),
    (48, 0, 136),
    (68, 0, 100),
    (92, 0, 48),
    (84, 4, 0),
    (60, 24, 0),
    (32, 42, 0),
    (8, 58, 0),
    (0, 64, 0),
    (0, 60, 0),
    (0, 50, 60),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (152, 150, 152),
    (8, 76, 196),
    (48, 50, 236),
    (92, 30, 228),
    (136, 20, 176),
    (160, 20, 100),
    (152, 34, 32),
    (120, 60, 0),
    (84, 90, 0),
    (40, 114, 0),
    (8, 124, 0),
    (0, 118, 40),
    (0, 102, 120),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (236, 238, 236),
    (76, 154, 236),
    (120, 124, 236),
    (176, 98, 236),
    (228, 84, 236),
    (236, 88, 180),
    (236, 106, 100),
    (212, 136, 32),
    (160, 170, 0),
    (116, 196, 0),
    (76, 208, 32),
    (56, 204, 108),
    (56, 180, 204),
    (60, 60, 60),
    (0, 0, 0),
    (0, 0, 0),
    (236, 238, 236),
    (168, 204, 236),
    (188, 188, 236),
    (212, 178, 236),
    (236, 174, 236),
    (236, 174, 212),
    (236, 180, 176),
    (228, 196, 144),
    (204, 210, 120),
    (180, 222, 120),
    (168, 226, 144),
    (152, 226, 180),
    (160, 214, 228),
    (160, 162, 160),
    (0, 0, 0),
    (0, 0, 0),
];

pub struct Ppu {
    ppu_ctrl: PPUCtrl,
    ppu_mask: PPUMask,
    ppu_status: PPUStatus,
    ppu_bus: PPUBus,
    ppu_addr: AddressRegister,
    w: bool,

    current_scanline: i16,
    current_cycle: u64,

    had_pre_render_scanline: bool,
    internal_vram_read_buffer: u8,

    screen_pixelbuffer: Vec<u8>,
    informed_frame_ready: bool,
    pub should_nmi: bool,

    oam_data: [u8; 0x100],
    oam_addr: u8,

    scanline_sprites: [OAMSprite; 8],
    scanline_sprites_count: usize,
}

impl Ppu {
    pub fn new() -> Self {
        let ppu_mask = PPUMask::from_bytes([0]);
        let ppu_ctrl = PPUCtrl::from_bytes([0]);
        let ppu_status = PPUStatus::from_bytes([0]);
        Self {
            ppu_status,
            ppu_ctrl,
            ppu_mask,
            ppu_bus: PPUBus::new(),
            ppu_addr: AddressRegister::new(),
            w: false,
            current_scanline: 0,
            current_cycle: 0,
            had_pre_render_scanline: false,
            internal_vram_read_buffer: 0,
            screen_pixelbuffer: vec![0; 240 * 256 * 4],
            informed_frame_ready: false,
            should_nmi: false,
            oam_data: [0; 0x100],
            oam_addr: 0,
            scanline_sprites: [OAMSprite::from_bytes(&[0u8; 4]); 8],
            scanline_sprites_count: 0,
        }
    }

    pub fn get_pixel_buffer(&self) -> &[u8] {
        &self.screen_pixelbuffer
    }

    fn call_nmi(&mut self) {
        if self.ppu_ctrl.vblank_nmi_enable() == 1 {
            self.should_nmi = true;
        }
    }

    pub fn frame_ready(&mut self) -> bool {
        // println!("{}", self.current_cycle);
        if self.ppu_status.vblank() == 1 && !self.informed_frame_ready {
            // if self.current_cycle == 0 {
            self.informed_frame_ready = true;
            true
        } else {
            false
        }
    }

    pub fn set_mapper(&mut self, mapper: SharedMapper) {
        self.ppu_bus.set_mapper(mapper);
    }

    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            PPUSTATUS => {
                self.w = false; // a side effect of reading PPUSTATUS is that the write toggle is cleared
                let result = self.ppu_status.into_bytes()[0];

                self.ppu_status.set_vblank(0); // Reading PPUSTATUS clears the vblank flag

                result
            }

            OAMDATA => {
                // Reads a byte from OAM at the current OAM address
                self.oam_data[self.oam_addr as usize]
            }

            PPUDATA => {
                /*
                The PPUDATA read buffer

                Reading from PPUDATA does not directly return the value at the current VRAM address, but instead returns the contents of an internal read buffer. This read buffer is updated on every PPUDATA read, but only after the previous contents have been returned to the CPU, effectively delaying PPUDATA reads by one. This is because PPU bus reads are too slow and cannot complete in time to service the CPU read. Because of this read buffer, after the VRAM address has been set through PPUADDR, one should first read PPUDATA to prime the read buffer (ignoring the result) before then reading the desired data from it.

                Note that the read buffer is updated only on PPUDATA reads. It is not affected by writes or other PPU processes such as rendering, and it maintains its value indefinitely until the next read.
                */
                let addr = self.ppu_addr.get_address();
                let data = self.ppu_bus.read_u8(addr);
                let old_internal_buffer = self.internal_vram_read_buffer;
                self.internal_vram_read_buffer = data;

                let increment = self.ppu_ctrl.get_increment_value();
                self.ppu_addr.increment(increment);

                match addr {
                    0x3f00..=0x3fff => data, // Palette reads do not use the buffer
                    _ => old_internal_buffer,
                }
            }

            _ => 0,
        }
    }

    pub fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            PPUCTRL => {
                if self.had_pre_render_scanline {
                    let new_val = PPUCtrl::from_bytes([value]);

                    if self.ppu_ctrl.vblank_nmi_enable() == 0
                        && new_val.vblank_nmi_enable() == 1
                        && self.ppu_status.vblank() == 1
                    {
                        self.call_nmi();
                    }

                    self.ppu_ctrl = new_val;
                }
            }

            PPUMASK => {
                if self.had_pre_render_scanline {
                    self.ppu_mask = PPUMask::from_bytes([value]);
                }
            }

            OAMADDR => {
                // Sets the OAM address for subsequent OAMDATA writes
                // Not implemented yet

                self.oam_addr = value;
            }

            OAMDATA => {
                // Writes a byte to OAM at the current OAM address, then increments the OAM address

                if !self.is_rendering() {
                    self.oam_data[self.oam_addr as usize] = value;
                    self.oam_addr = self.oam_addr.wrapping_add(1);
                }
            }

            PPUSCROLL => {
                // TODO: Not implemented yet
            }

            PPUADDR => {
                self.ppu_addr.update(value, &mut self.w);
            }

            PPUDATA => {
                let addr = self.ppu_addr.get_address();
                self.ppu_bus.write_u8(addr, value);

                let increment = self.ppu_ctrl.get_increment_value();
                self.ppu_addr.increment(increment);
            }

            _ => {}
        }
    }

    fn is_rendering(&self) -> bool {
        self.current_scanline >= 0 && self.current_scanline < 240
    }

    pub fn tick(&mut self) {
        let current_pixel_x = (self.current_cycle % 341) as u16;
        let current_pixel_y = self.current_scanline as u16;

        let current_tile_x = current_pixel_x / 8; // value between 0 and 32
        let current_tile_y = current_pixel_y / 8; // value between 0 and 30

        if current_pixel_y < 240 && current_pixel_x < 256 {
            if self.ppu_mask.show_background() == 1 {
                self.render_background(
                    current_pixel_x,
                    current_pixel_y,
                    current_tile_x,
                    current_tile_y,
                );
            }
            if self.ppu_mask.show_sprites() == 1 {
                for sprite in self.scanline_sprites[..self.scanline_sprites_count]
                    .iter()
                    .filter(|s| {
                        let x = s.get_x() as u16;
                        x <= current_pixel_x && current_pixel_x < x + 8
                    })
                    .rev()
                {
                    // reverse this to give the first sprite most priority
                    let mut current_sprite_line = self.current_scanline as u8 - sprite.get_y();
                    if sprite.get_attributes().flip_vertical() == 1 {
                        current_sprite_line = self.ppu_ctrl.get_sprite_height() - current_sprite_line;
                    }

                    let mut current_sprite_x = current_pixel_x as u8 - sprite.get_x();
                    if sprite.get_attributes().flip_horizontal() == 1 {
                        current_sprite_x = 7 - current_sprite_x
                    }

                    let pallette_table = PALLETTE_TABLE_START
                        + 0x10
                        + (sprite.get_attributes().pallette() as u16 * 4);
                    
                    let pattern_table = if self.ppu_ctrl.get_sprite_height() == 8 {
                        self.ppu_ctrl.get_sprite_pattern_table_address()
                    } else { // == 16
                        if sprite.get_tile_index() & 1 == 1 {
                            0x1000
                        } else {
                            0
                        }
                    };

                    let tile_index = if current_sprite_line >= 8 {
                        sprite.get_tile_index() + 1
                    } else {
                        sprite.get_tile_index()
                    } as u16;

                    let current_tile_y = (current_sprite_line % 8) as u16;
                    let current_tile_x = current_sprite_x as u16;

                    let pattern_lsb_address = pattern_table + (tile_index * 16 + 0) + (current_tile_y % 8);
                    let pattern_msb_address = pattern_table + (tile_index * 16 + 8) + (current_tile_y % 8);

                    let pattern_byte_lsb = self.ppu_bus.read_u8(pattern_lsb_address);
                    let pattern_byte_msb = self.ppu_bus.read_u8(pattern_msb_address);
                    // println!("{current_tile_x}");
                    let current_pixel_color_lsb =
                        select_bit_n(pattern_byte_lsb as usize, current_tile_x as usize);
                    let current_pixel_color_msb =
                        select_bit_n(pattern_byte_msb as usize, current_tile_x as usize);
                    let pixel_color = current_pixel_color_lsb + (current_pixel_color_msb << 1);

                    if !pixel_color.is_multiple_of(4) {
                        let actual_pixel_color = COLORS[self.ppu_bus.read_u8(pallette_table + pixel_color as u16) as usize];

                        let current_pixel_index = current_pixel_y as usize * 256 * 4 + current_pixel_x as usize * 4;
                        self.screen_pixelbuffer[current_pixel_index + 0] = actual_pixel_color.0;
                        self.screen_pixelbuffer[current_pixel_index + 1] = actual_pixel_color.1;
                        self.screen_pixelbuffer[current_pixel_index + 2] = actual_pixel_color.2;
                        self.screen_pixelbuffer[current_pixel_index + 3] = 0xff;
                    }

                }
            }
        } else if self.current_scanline == 241 && current_pixel_x == 1 {
            // Entering VBlank
            self.ppu_status.set_vblank(1);

            if self.ppu_ctrl.vblank_nmi_enable() == 1 {
                self.call_nmi();
            }
        } else if self.current_scanline == 261 && current_pixel_x == 1 {
            // Pre-render scanline
            self.ppu_status.set_vblank(0);
            self.had_pre_render_scanline = true;
        } else if 320 == current_pixel_x
            && (self.current_scanline < 240 || self.current_scanline == 261)
        {
            // supposedly sprite evaluation happens in x = 257..=320 but i'm going to do it in one go because lazy
            // the sprite tile loading interval
            self.do_sprite_evaluation();

            self.oam_addr = 0;
        }

        self.current_cycle += 1;
        if self.current_cycle >= 341 {
            self.current_cycle = 0;
            self.current_scanline += 1;

            if self.current_scanline > 261 {
                self.current_scanline = 0;
                self.informed_frame_ready = false;
            }
        }
    }

    fn do_sprite_evaluation(&mut self) {
        self.scanline_sprites_count = 0;
        let mut oam_reader = BufReader::new(Cursor::new(&self.oam_data));

        let mut bytes = [0u8; 4];

        for _ in 0..64 {
            oam_reader.read_exact(&mut bytes).unwrap();
            let sprite = OAMSprite::from_bytes(&bytes);

            let next_scanline = ((self.current_scanline + 1) % 262) as u8; // it's aight to cast because of scanline range
            let sprite_height = self.ppu_ctrl.get_sprite_height();
            if sprite.get_y() <= next_scanline && next_scanline <= sprite.get_y() + sprite_height && self.scanline_sprites_count < 8 {
                self.scanline_sprites[self.scanline_sprites_count] = sprite;
                self.scanline_sprites_count += 1;
            }
        }
    }

    fn render_background(
        &mut self,
        current_pixel_x: u16,
        current_pixel_y: u16,
        current_tile_x: u16,
        current_tile_y: u16,
    ) {
        // Visible scanlines
        let nametable_address = self.ppu_ctrl.get_base_nametable_address();
        let attribute_table_address = nametable_address + ATTRIBUTE_TABLE_OFFSET;
        let pattern_table_address = self.ppu_ctrl.get_background_pattern_table_address();

        // First we get the nametable entry for the current pixel
        let nametable_index = current_tile_x + current_tile_y * 32;
        let nametable_entry = self.ppu_bus.read_u8(nametable_address + nametable_index);

        // We want to get the matching pallette for the nametable entry
        let attribute_table_index = (current_tile_x / 4) + (current_tile_y / 4) * 8;
        let attribute_byte = self
            .ppu_bus
            .read_u8(attribute_table_address + attribute_table_index);
        let quadrant = ((current_tile_y % 4) / 2) * 2 + ((current_tile_x % 4) / 2);
        let pallette_table_index = (attribute_byte >> (quadrant * 2)) & 0b11;
        let pallette_address = PALLETTE_TABLE_START + (pallette_table_index as u16 * 4);

        // The nametable entry indexes the pattern table
        let pattern_table_index = nametable_entry as u16;
        let pattern_lsb_address =
            pattern_table_address + (pattern_table_index * 16 + 0) + (current_pixel_y % 8);
        let pattern_msb_address =
            pattern_table_address + (pattern_table_index * 16 + 8) + (current_pixel_y % 8);

        let pattern_byte_lsb = self.ppu_bus.read_u8(pattern_lsb_address);
        let pattern_byte_msb = self.ppu_bus.read_u8(pattern_msb_address);


        let current_pixel_color_lsb =
            select_bit_n(pattern_byte_lsb as usize, current_pixel_x as usize % 8);
        let current_pixel_color_msb =
            select_bit_n(pattern_byte_msb as usize, current_pixel_x as usize % 8);
        let pixel_color = current_pixel_color_lsb + (current_pixel_color_msb << 1);

        let mut pallette_value = if pixel_color.is_multiple_of(4) {
            self.ppu_bus.read_u8(pallette_address + 0)
        } else {
            self.ppu_bus.read_u8(pallette_address + pixel_color as u16)
        };

        if self.ppu_mask.grayscale() == 1 {
            pallette_value &= 0x30;
        }

        let actual_pixel_color = COLORS[pallette_value as usize];

        let current_pixel_index = current_pixel_y as usize * 256 * 4 + current_pixel_x as usize * 4;
        self.screen_pixelbuffer[current_pixel_index + 0] = actual_pixel_color.0;
        self.screen_pixelbuffer[current_pixel_index + 1] = actual_pixel_color.1;
        self.screen_pixelbuffer[current_pixel_index + 2] = actual_pixel_color.2;
        self.screen_pixelbuffer[current_pixel_index + 3] = 0xff;
    }
}

fn select_bit_n(x: usize, n: usize) -> usize {
    (x >> (7 - n)) & 1
}