use crate::ppu::{
    address_register::AddressRegister, ppu_bus::PPUBus, ppu_ctrl::PPUCtrl, ppu_mask::PPUMask,
    ppu_status::PPUStatus,
};

pub struct PpuRegisters {
    pub ppu_ctrl: PPUCtrl,
    pub ppu_mask: PPUMask,
    pub ppu_status: PPUStatus,
    pub ppu_addr: AddressRegister,
    pub w: bool,
    pub v: u8,
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
        }
    }
}
