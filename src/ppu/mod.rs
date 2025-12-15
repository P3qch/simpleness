mod ppu_bus;
mod ppu_ctrl;
mod ppu_mask;
mod ppu_status;
mod address_register;

use ppu_bus::PPUBus;
use ppu_ctrl::PPUCtrl;
use ppu_mask::PPUMask;
use ppu_status::PPUStatus;
use address_register::AddressRegister;

use crate::memory::mapper::SharedMapper;

const PPUCTRL: u16 = 0x2000;
const PPUMASK: u16 = 0x2001;
const PPUSTATUS: u16 = 0x2002;
const PPUADDR: u16 = 0x2006;
const PPUDATA: u16 = 0x2007;

pub struct PPU {
    write_latch: u8, //Toggles on each write to either PPUSCROLL or PPUADDR, indicating whether this is the first or second write. Clears on reads of PPUSTATUS. Sometimes called the 'write latch' or 'write toggle'.
    ppu_ctrl: PPUCtrl,
    ppu_mask: PPUMask,
    ppu_status: PPUStatus,
    ppu_bus: PPUBus,
    ppu_addr: AddressRegister,
    w: bool,
    current_scanline: i16,
    current_cycle: i16,
    had_pre_render_scanline: bool,
    internal_vram_read_buffer: u8,
}

#[derive(Clone, Copy)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl PPU {
    pub fn new() -> Self {
        let ppu_mask = PPUMask::from_bytes([0]);
        let ppu_ctrl = PPUCtrl::from_bytes([0]);
        let ppu_status = PPUStatus::from_bytes([0]); 
        Self {
            write_latch: 0,
            ppu_status: ppu_status,
            ppu_ctrl: ppu_ctrl,
            ppu_mask: ppu_mask,
            ppu_bus: PPUBus::new(),
            ppu_addr: AddressRegister::new(),
            w: false,
            current_scanline: 0,
            current_cycle: 0,
            had_pre_render_scanline: false,
            internal_vram_read_buffer: 0,
        }
    }

    fn call_nmi(&self) {
        todo!();
    }

    pub fn set_mapper(&mut self, mapper: SharedMapper) {
        self.ppu_bus.set_mapper(mapper);
    }

    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            PPUSTATUS => {
                self.w = false; // a side effect of reading PPUSTATUS is that the write toggle is cleared
                self.ppu_status.into_bytes()[0]
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
                    _ => old_internal_buffer
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

                    if self.ppu_ctrl.vblank_nmi_enable() == 0 && new_val.vblank_nmi_enable() == 1 && self.ppu_status.vblank() == 1 {
                        self.call_nmi();
                    }
                }
            }
            PPUADDR => {
                self.ppu_addr.update(value, &mut self.w);
            }
            _ => {}
        }
    }   

    pub fn tick(&mut self) {

    }
}
