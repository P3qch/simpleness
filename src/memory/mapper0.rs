use crate::memory::mapper::Mapper;

pub struct Mapper0 {
    prg_rom: Vec<u8>,
    should_mirror_prg_rom_page: bool,
}

impl Mapper0 {
    pub fn new(prg_rom: Vec<u8>) -> Self {
        if prg_rom.len() == 0x4000 {
            Self {
                prg_rom,
                should_mirror_prg_rom_page: true,
            }
        } else if prg_rom.len() == 0x8000 {
            Self {
                prg_rom,
                should_mirror_prg_rom_page: false,
            }
        } else {
            panic!("Unsupported PRG ROM size: {}", prg_rom.len());
        }
    }

}

impl Mapper for Mapper0 {
    fn cpu_map_read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0xFFFF => {
                let prg_rom_addr = (addr - 0x8000) as usize;
                if prg_rom_addr < self.prg_rom.len() && !self.should_mirror_prg_rom_page {
                    self.prg_rom[prg_rom_addr]
                } else if prg_rom_addr & 0x3fff < self.prg_rom.len() && self.should_mirror_prg_rom_page {
                    self.prg_rom[prg_rom_addr & 0x3fff]
                } else {
                    panic!("PRG ROM read out of bounds: {:04X}", addr);
                }
            }
            _ => 0
        }
    }

    fn cpu_map_write(&mut self, _addr: u16, _data: u8) {
        // NROM does not support writes to PRG ROM
    }
}