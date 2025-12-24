use crate::memory::mapper::Mapper;

pub struct Mapper0 {
    prg_rom: Vec<u8>,
    should_mirror_prg_rom_page: bool,
    chr_rom: Vec<u8>,
}

impl Mapper0 {
    pub fn new(prg_rom: Vec<u8>, mut chr_rom: Vec<u8>) -> Self {
        let should_mirror_prg_rom_page = prg_rom.len() == 0x4000;
        if prg_rom.len() != 0x4000 && prg_rom.len() != 0x8000 {
            panic!("Invalid prg rom size");
        } 

        if chr_rom.len() == 0 {
            chr_rom = vec![0; 0x2000];
        }

        if chr_rom.len() != 0x2000 && chr_rom.len() != 0 {
            panic!("Invalid chr rom size");
        }

        Self { prg_rom, should_mirror_prg_rom_page, chr_rom }
    }
}

impl Mapper for Mapper0 {
    fn cpu_map_read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0xFFFF => {
                let prg_rom_addr = (addr - 0x8000) as usize;
                if prg_rom_addr < self.prg_rom.len() && !self.should_mirror_prg_rom_page {
                    self.prg_rom[prg_rom_addr]
                } else if prg_rom_addr & 0x3fff < self.prg_rom.len()
                    && self.should_mirror_prg_rom_page
                {
                    self.prg_rom[prg_rom_addr & 0x3fff]
                } else {
                    panic!("PRG ROM read out of bounds: {:04X}", addr);
                }
            }
            _ => 0,
        }
    }

    fn cpu_map_write(&mut self, _addr: u16, _data: u8) {
        // NROM does not support writes to PRG ROM
    }

    fn ppu_map_read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let chr_rom_addr = addr as usize;
                if chr_rom_addr < self.chr_rom.len() {
                    self.chr_rom[chr_rom_addr]
                } else {
                    panic!("CHR ROM read out of bounds: {:04X}", addr);
                }
            }
            _ => 0,
        }
    }

    fn ppu_map_write(&mut self, addr: u16, data: u8) {
        if let 0x0000..=0x1FFF = addr {
            self.chr_rom[addr as usize] = data;
        }
    }
}
