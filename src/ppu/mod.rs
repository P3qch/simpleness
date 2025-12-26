mod address_register;
mod oam_sprite;
mod ppu_bus;
mod ppu_ctrl;
mod ppu_mask;
mod ppu_registers;
mod ppu_status;

use std::{
    collections::VecDeque, io::{BufReader, Cursor, Read}, os::raw, vec
};

pub use ppu_bus::NametableArrangement;
use ppu_ctrl::PPUCtrl;
use ppu_mask::PPUMask;
use ppu_registers::PpuRegisters;

use crate::{
    memory::mapper::SharedMapper,
    ppu::{oam_sprite::OAMSprite, ppu_bus::PPUBus, ppu_registers::ScrollRegister},
};

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

#[derive(Default, Clone, Copy)]
struct ScrollState {
    pub coarse_x: u8,
    pub coarse_y: u8,
    pub fine_x: u8,
    pub fine_y: u8,
}

pub struct Ppu {
    registers: PpuRegisters,
    ppu_bus: PPUBus,

    read_buffer: u8,

    current_scanline: i16,
    current_cycle: u64,

    had_pre_render_scanline: bool,

    screen_pixelbuffer: Vec<u8>,
    informed_frame_ready: bool, // has informed that the frame is ready to render
    pub should_nmi: bool,       // tells the cpu to nmi

    oam_data: [u8; 0x100],
    oam_addr: u8,

    scanline_sprites: [OAMSprite; 8],
    scanline_sprites_count: usize,

    opaque_bg_pixel_table: [[bool; 256]; 240],

    next_pixels: VecDeque<u8>,
}

impl Ppu {
    pub fn new(mirroring_mode: NametableArrangement) -> Self {
        Self {
            registers: PpuRegisters::new(),
            ppu_bus: PPUBus::new(mirroring_mode),
            read_buffer: 0,
            current_scanline: 0,
            current_cycle: 0,
            had_pre_render_scanline: false,
            screen_pixelbuffer: vec![0; 240 * 256 * 4],
            informed_frame_ready: false,
            should_nmi: false,
            oam_data: [0; 0x100],
            oam_addr: 0,
            scanline_sprites: [OAMSprite::from_bytes(&[0u8; 4], false); 8],
            scanline_sprites_count: 0,
            opaque_bg_pixel_table: [[false; 256]; 240],
            next_pixels: VecDeque::from([0; 0x10]), // pixels are emitted before.
        }
    }

    pub fn set_nametable_arrangement(&mut self, mode: NametableArrangement) {
        self.ppu_bus.set_nametable_arrangement(mode);
    }

    pub fn get_pixel_buffer(&self) -> &[u8] {
        &self.screen_pixelbuffer
    }

    fn call_nmi(&mut self) {
        if self.registers.ppu_ctrl.vblank_nmi_enable() == 1 {
            self.should_nmi = true;
        }
    }

    pub fn frame_ready(&mut self) -> bool {
        if self.registers.ppu_status.vblank() == 1 && !self.informed_frame_ready {
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
                self.registers.w = false; // a side effect of reading PPUSTATUS is that the write toggle is cleared
                let result = self.registers.ppu_status.into_bytes()[0];

                self.registers.ppu_status.set_vblank(0); // Reading PPUSTATUS clears the vblank flag

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
                let addr = self.registers.v;
                let data = self.ppu_bus.read_u8(addr);

                let increment = self.registers.ppu_ctrl.get_increment_value();
                self.registers.v += increment;

                match addr {
                    0x3f00..=0x3fff => data, // Palette reads do not use the buffer
                    _ => {
                        let ret = self.read_buffer;
                        self.read_buffer = data;
                        ret
                    }
                }
            }

            _ => 0,
        }
    }

    pub fn write_register(&mut self, addr: u16, value: u8) {
        let mut new_t = ScrollRegister::from(self.registers.t);
        match addr {
            PPUCTRL => {
                if self.had_pre_render_scanline {
                    new_t.set_nametable_select(value & 0b11);
                    self.registers.t = new_t.into();

                    let old_val = self.registers.ppu_ctrl;
                    self.registers.ppu_ctrl = PPUCtrl::from_bytes([value]);
                    if old_val.vblank_nmi_enable() == 0
                        && self.registers.ppu_ctrl.vblank_nmi_enable() == 1
                        && self.registers.ppu_status.vblank() == 1
                    {
                        self.call_nmi();
                    }
                }
            }

            PPUMASK => {
                if self.had_pre_render_scanline {
                    self.registers.ppu_mask = PPUMask::from_bytes([value]);
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
                if !self.is_rendering() {
                    if !self.registers.w {
                        new_t.set_coarse_x((value >> 3) & 0b11111);
                        self.registers.x = value & 0b111;
                    } else {
                        new_t.set_coarse_y((value >> 3) & 0b11111);
                        new_t.set_fine_y(value & 0b111);
                    }

                    self.registers.w = !self.registers.w;
                    self.registers.t = new_t.into();
                }
            }

            PPUADDR => {
                if !self.registers.w {
                    self.registers.t = (self.registers.t & 0xff) | ((value as u16 & 0x3f) << 8);
                } else {
                    self.registers.t = (self.registers.t & 0xff00) | value as u16 & 0xff;
                    self.registers.v = self.registers.t;
                }
                self.registers.w = !self.registers.w;
            }

            PPUDATA => {
                let addr = self.registers.v;
                self.ppu_bus.write_u8(addr, value);

                let increment = self.registers.ppu_ctrl.get_increment_value();
                self.registers.v += increment;
            }

            _ => {}
        }
    }

    fn is_rendering(&self) -> bool {
        self.current_scanline >= 0
            && self.current_scanline < 240
            && (self.registers.ppu_mask.show_background() == 1
                || self.registers.ppu_mask.show_sprites() == 1)
            && self.registers.ppu_status.vblank() == 0
    }

    pub fn tick(&mut self) {
        let current_pixel_x = (self.current_cycle % 341) as u16;
        let current_pixel_y = self.current_scanline as u16;

        if current_pixel_y < 240 && current_pixel_x < 256 {
            if self.registers.ppu_mask.show_background() == 1 {
                self._old_render_background(current_pixel_x, current_pixel_y);
            }
            if self.registers.ppu_mask.show_sprites() == 1 {
                self.render_sprites(current_pixel_x, current_pixel_y);
            }
        } else if self.current_scanline == 241 && current_pixel_x == 1 {
            // Entering VBlank
            self.registers.ppu_status.set_vblank(1);

            if self.registers.ppu_ctrl.vblank_nmi_enable() == 1 {
                self.call_nmi();
            }
        } else if self.current_scanline == 261 {
            // Pre-render scanline
            if current_pixel_x == 1 {
                self.registers.ppu_status.set_vblank(0);
                self.registers.ppu_status.set_sprite_zero_hit(0);
                self.had_pre_render_scanline = true;
            }
            if let 280..304 = current_pixel_x {
                self.registers.v &= !0b1111011_11100000;
                self.registers.v |= self.registers.t & 0b1111011_11100000
            }
        } else if 320 == current_pixel_x
            && (self.current_scanline < 240 || self.current_scanline == 261)
        {
            // supposedly sprite evaluation happens in x = 257..=320 but i'm going to do it in one go because lazy
            // the sprite tile loading interval
            self.do_sprite_evaluation();

            self.oam_addr = 0;
        }

        if self.registers.ppu_mask.show_background() == 1
            && (self.current_scanline < 240 || self.current_scanline >= 261)
        {
            match current_pixel_x {
                256 => {
                    let mut v = ScrollRegister::from(self.registers.v);

                    if v.fine_y() < 7 {
                        v.set_fine_y(v.fine_y() + 1);
                    } else {
                        v.set_fine_y(0);

                        match v.coarse_y() {
                            29 => {
                                v.set_coarse_y(0);
                                v.set_nametable_select(v.nametable_select() ^ 0b10);
                            }
                            31 => {
                                v.set_coarse_y(0);
                            }
                            _ => {
                                v.set_coarse_y(v.coarse_y() + 1);
                            }
                        }
                    }
                    self.registers.v = v.into();
                }
                257 => {
                    self.registers.v &= !0b100_00011111;
                    self.registers.v |= self.registers.t & 0b100_00011111;
                }
                328 | 336 | 8..=256 => {
                    if current_pixel_x.is_multiple_of(8) {
                        let mut v = ScrollRegister::from(self.registers.v);

                        if v.coarse_x() == 31 {
                            v.set_coarse_x(0);
                            v.set_nametable_select(v.nametable_select() ^ 0b01);
                        } else {
                            v.set_coarse_x(v.coarse_x() + 1);
                        }

                        self.registers.v = v.into();
                    }
                }
                _ => (),
            }
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

    fn render_sprites(&mut self, current_pixel_x: u16, current_pixel_y: u16) {
        let mut sprites =
            vec![OAMSprite::from_bytes(&[0, 0, 0, 0], false); self.scanline_sprites_count];
        self.scanline_sprites[..self.scanline_sprites_count].clone_into(&mut sprites);
        for sprite in sprites
            .iter()
            .filter(|s| {
                let x = s.get_x() as u16;
                x <= current_pixel_x && current_pixel_x < x + 8
            })
            .rev()
        // reverse to prioritise lower indexes in oam
        {
            self.render_sprite(current_pixel_x, current_pixel_y, *sprite);
        }
    }

    fn do_sprite_evaluation(&mut self) {
        self.scanline_sprites_count = 0;
        let mut oam_reader = BufReader::new(Cursor::new(&self.oam_data));

        let mut bytes = [0u8; 4];

        for i in 0..64 {
            oam_reader.read_exact(&mut bytes).unwrap();
            let sprite = OAMSprite::from_bytes(&bytes, i == 0);

            let next_scanline = ((self.current_scanline + 1) % 262) as u16; // it's aight to cast because of scanline range
            let sprite_height = self.registers.ppu_ctrl.get_sprite_height() as u16;
            if sprite.get_rendered_y() <= next_scanline
                && next_scanline < sprite.get_rendered_y() + sprite_height
                && self.scanline_sprites_count < 8
                && sprite.get_y() > 1
            {
                self.scanline_sprites[self.scanline_sprites_count] = sprite;
                self.scanline_sprites_count += 1;
            }
        }
    }

    fn _old_render_background(&mut self, current_pixel_x: u16, current_pixel_y: u16) {
        let current_tile_x = current_pixel_x / 8;
        let current_tile_y = current_pixel_y / 8;

        let nametable_address = self.registers.ppu_ctrl.get_base_nametable_address();
        let attribute_table_address = nametable_address + ATTRIBUTE_TABLE_OFFSET;
        let pattern_table_address = self
            .registers
            .ppu_ctrl
            .get_background_pattern_table_address();

        let nametable_entry = {
            let nametable_index = current_tile_x + current_tile_y * 32;
            self.ppu_bus.read_u8(nametable_address + nametable_index) //
        };

        let pallette_address = {
            let attribute_table_index = (current_tile_x / 4) + (current_tile_y / 4) * 8;
            let attribute_byte = self
                .ppu_bus
                .read_u8(attribute_table_address + attribute_table_index);
            let quadrant = ((current_tile_y % 4) / 2) * 2 + ((current_tile_x % 4) / 2);
            let pallette_table_index = (attribute_byte >> (quadrant * 2)) & 0b11;

            PALLETTE_TABLE_START + (pallette_table_index as u16 * 4)
        };

        let pixel_color = self.get_pattern_pixel(
            pattern_table_address,
            nametable_entry as u16,
            current_pixel_y % 8,
            current_pixel_x % 8,
        );

        self.opaque_bg_pixel_table[current_pixel_y as usize][current_pixel_x as usize] =
            pixel_color != 0;

        let pallette_value = self.get_pallette_value(pallette_address, pixel_color as u16);

        self.draw_pixel(current_pixel_x, current_pixel_y, pallette_value as usize);
    }

    fn render_background(&mut self, current_pixel_x: u16, current_pixel_y: u16) {
        let v = self.registers.v;
        let parsed_v = ScrollRegister::from(v);

        let current_tile_x = parsed_v.coarse_x() as u16;
        let current_tile_y = parsed_v.coarse_y() as u16;

        // println!("({current_tile_x}, {current_tile_y}) ({}, {})", self.registers.x, parsed_v.fine_y());

        let pattern_table_address = self
            .registers
            .ppu_ctrl
            .get_background_pattern_table_address();

        let nametable_entry = {
            self.ppu_bus.read_u8(0x2000 | (v & 0x0FFF)) //
        };

        let pallette_address = {
            let attribute_byte = self
                .ppu_bus
                .read_u8(0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07));
            let quadrant = ((current_tile_y % 4) / 2) * 2 + ((current_tile_x % 4) / 2);
            let pallette_table_index = (attribute_byte >> (quadrant * 2)) & 0b11;

            PALLETTE_TABLE_START + (pallette_table_index as u16 * 4)
        };

        let future_pixel_color = self.get_pattern_pixel(
            pattern_table_address,
            nametable_entry as u16,
            parsed_v.fine_y() as u16,
            current_pixel_x % 8, // + self.registers.x as u16 % 8,
        );

        self.next_pixels.push_back(future_pixel_color);
        let pixel_color = self.next_pixels.pop_front().unwrap();

        let pallette_value = self.get_pallette_value(pallette_address, pixel_color as u16);
        if current_pixel_x < 256 {
            self.opaque_bg_pixel_table[current_pixel_y as usize][current_pixel_x as usize] =
                pixel_color != 0;
            self.draw_pixel(current_pixel_x, current_pixel_y, pallette_value as usize);
        }
    }

    fn render_sprite(&mut self, current_pixel_x: u16, current_pixel_y: u16, sprite: OAMSprite) {
        // reverse this to give the first sprite most priority
        let mut current_sprite_line = self.current_scanline as u8 - sprite.get_rendered_y() as u8;
        if sprite.get_attributes().flip_vertical() == 1 {
            current_sprite_line =
                self.registers.ppu_ctrl.get_sprite_height() - 1 - current_sprite_line;
        }

        let mut current_sprite_x = current_pixel_x as u8 - sprite.get_x();
        if sprite.get_attributes().flip_horizontal() == 1 {
            current_sprite_x = 7 - current_sprite_x
        }

        let pallette_table =
            PALLETTE_TABLE_START + 0x10 + (sprite.get_attributes().pallette() as u16 * 4);

        let pattern_table = if self.registers.ppu_ctrl.get_sprite_height() == 8 {
            self.registers.ppu_ctrl.get_sprite_pattern_table_address()
        } else {
            // == 16
            if sprite.get_tile_index() & 1 == 1 {
                0x1000
            } else {
                0
            }
        };

        let tile_index = if self.registers.ppu_ctrl.get_sprite_height() == 16
            && ((current_sprite_line >= 8 && sprite.get_attributes().flip_vertical() == 0)
                || (current_sprite_line < 8 && sprite.get_attributes().flip_vertical() == 1))
        {
            sprite.get_tile_index() + 1
        } else {
            sprite.get_tile_index()
        } as u16;

        let current_tile_y = current_sprite_line as u16 % 8;
        let current_tile_x = current_sprite_x as u16;

        let pixel_color =
            self.get_pattern_pixel(pattern_table, tile_index, current_tile_y, current_tile_x);

        if sprite.is_sprite_0()
            && self.opaque_bg_pixel_table[current_pixel_y as usize][current_pixel_x as usize]
        {
            self.registers.ppu_status.set_sprite_zero_hit(1);
        }

        if pixel_color != 0
            && (sprite.get_attributes().priority() == 0
                || (sprite.get_attributes().priority() == 1
                    && !self.opaque_bg_pixel_table[current_pixel_y as usize]
                        [current_pixel_x as usize]))
        {
            let pallette_value = self.get_pallette_value(pallette_table, pixel_color as u16);
            self.draw_pixel(current_pixel_x, current_pixel_y, pallette_value as usize);
        }
    }

    fn get_pallette_value(&mut self, pallette_table_address: u16, pallette_index: u16) -> u8 {
        if pallette_index == 0 {
            self.ppu_bus.read_u8(PALLETTE_TABLE_START)
        } else {
            self.ppu_bus
                .read_u8(pallette_table_address + pallette_index)
        }
    }

    fn get_pattern_pixel(
        &mut self,
        pattern_table: u16,
        tile_index: u16,
        fine_y: u16,
        fine_x: u16,
    ) -> u8 {
        let pattern_lsb_address = pattern_table + (tile_index * 16 + 0) + fine_y;
        let pattern_msb_address = pattern_table + (tile_index * 16 + 8) + fine_y;

        let pattern_byte_lsb = self.ppu_bus.read_u8(pattern_lsb_address);
        let pattern_byte_msb = self.ppu_bus.read_u8(pattern_msb_address);

        let current_pixel_color_lsb = select_bit_n(pattern_byte_lsb, fine_x as u8);
        let current_pixel_color_msb = select_bit_n(pattern_byte_msb, fine_x as u8);
        let pixel_color = current_pixel_color_lsb + (current_pixel_color_msb << 1);
        pixel_color
    }

    fn draw_pixel(&mut self, current_pixel_x: u16, current_pixel_y: u16, mut color: usize) {
        if self.registers.ppu_mask.grayscale() == 1 {
            color &= 0x30;
        }

        let pixel_color = COLORS[color];

        let current_pixel_index = current_pixel_y as usize * 256 * 4 + current_pixel_x as usize * 4;
        self.screen_pixelbuffer[current_pixel_index + 0] = pixel_color.0;
        self.screen_pixelbuffer[current_pixel_index + 1] = pixel_color.1;
        self.screen_pixelbuffer[current_pixel_index + 2] = pixel_color.2;
        self.screen_pixelbuffer[current_pixel_index + 3] = 0xff;
    }
}

fn select_bit_n(x: u8, n: u8) -> u8 {
    (x >> (7 - n)) & 1
}
