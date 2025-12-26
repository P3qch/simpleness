use modular_bitfield::prelude::*;

use crate::ppu::{
    address_register::AddressRegister, ppu_ctrl::PPUCtrl, ppu_mask::PPUMask,
    ppu_status::PPUStatus,
};

#[derive(Clone, Copy)]
#[bitfield(bits = 16)]
pub struct ScrollRegister {
    pub coarse_x: B5,
    pub coarse_y: B5,
    pub nametable_select: B2,
    pub fine_y: B3,
    unused: B1,
}

impl From<u16> for ScrollRegister {
    fn from(value: u16) -> Self {
        Self::from_bytes(value.to_le_bytes())
    }
}

impl From<ScrollRegister> for u16 {
    fn from(reg: ScrollRegister) -> Self {
        u16::from_le_bytes(reg.into_bytes())
    }
}

pub struct PpuRegisters {
    pub ppu_ctrl: PPUCtrl,
    pub ppu_mask: PPUMask,
    pub ppu_status: PPUStatus,
    pub ppu_addr: AddressRegister,
    pub w: bool,
    pub v: u16,
    pub t: u16, // temporary VRAM address
    pub x: u8  // fine X scroll (0–7)
}


impl PpuRegisters {
    pub fn new() -> Self {
        Self {
            ppu_ctrl: PPUCtrl::new(),
            ppu_mask: PPUMask::new(),
            ppu_status: PPUStatus::new(),
            ppu_addr: AddressRegister::new(),
            w: false,
            v: 0,
            t: 0,
            x: 0,
        }
    }

    pub fn get_nametable_address(&self) -> u16 {
        0x2000 + 0x400 * (ScrollRegister::from(self.v).nametable_select() as u16)
    }
}
