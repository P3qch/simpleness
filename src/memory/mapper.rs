use crate::{memory::mapper0::Mapper0, ppu::NametableArrangement};
use byteorder::ReadBytesExt;
use std::{cell::RefCell, io::Read, rc::Rc};
use modular_bitfield::prelude::*;

pub type SharedMapper = Rc<RefCell<Box<dyn crate::memory::mapper::Mapper>>>;

pub trait Mapper {
    fn cpu_map_read(&self, addr: u16) -> u8;
    fn cpu_map_write(&mut self, addr: u16, data: u8);
    fn ppu_map_read(&self, addr: u16) -> u8;
    fn ppu_map_write(&mut self, addr: u16, data: u8);
}

#[bitfield(bits=8)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct INesFlag6 {
    pub nametable_arrangement: B1, // 0: vertical arrangement 1: horizontal arrangement
    pub persistant_ram: B1,
    pub has_trainer: B1,
    pub alternative_nametable_layout: B1,
    pub low_mapper_nybble: B4,
}

impl INesFlag6 {
    pub fn get_nametable_mirroring_mode(&self) -> NametableArrangement {
        if self.nametable_arrangement() == 0 {
            NametableArrangement::Vertical
        } else {
            NametableArrangement::Horizontal
        }
    }
}

pub struct Rom {
    pub mapper: Box<dyn Mapper>,
    pub flag6: INesFlag6,
}

impl Rom {
    pub fn new(mapper: Box<dyn Mapper>, flag6: INesFlag6) -> Self {
        Self { mapper, flag6 }
    }

    pub fn parse(rom_content: Vec<u8>) -> Self {
        let mut reader = std::io::BufReader::new(std::io::Cursor::new(rom_content));
        let mut magic_buf = [0u8; 4];
        reader.read_exact(&mut magic_buf).unwrap();
        if &magic_buf != b"NES\x1A" {
            panic!("Invalid ROM file: missing NES header");
        }
        let prg_rom_size = reader.read_u8().unwrap() as usize * 0x4000;
        let chr_rom_size = reader.read_u8().unwrap() as usize * 0x2000;

        let flag6 = reader.read_u8().unwrap();
        let flag7 = reader.read_u8().unwrap();

        let mut garbage_buf = [0u8; 8];
        reader.read_exact(&mut garbage_buf).unwrap();

        let mut prg_rom = vec![0u8; prg_rom_size];
        reader.read_exact(&mut prg_rom).unwrap();

        let mut chr_rom = vec![0u8; chr_rom_size];
        reader.read_exact(&mut chr_rom).unwrap();

        let mapper_number = (flag7 & 0xF0) | (flag6 >> 4);

        match mapper_number {
            0 => {
                let mapper = Box::new(Mapper0::new(prg_rom, chr_rom));
                Self::new(mapper, INesFlag6::from_bytes([flag6]))
            }
            _ => {
                panic!("Unsupported mapper number: {}", mapper_number);
            }
        }
    }
}