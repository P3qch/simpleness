use std::{fs, io::Read};
use byteorder::ReadBytesExt;
use crate::memory::mapper0::Mapper0;

pub trait Mapper {
    fn cpu_map_read(&self, addr: u16) -> u8;
    fn cpu_map_write(&mut self, addr: u16, data: u8);
}

pub fn parse_rom(path: &str) -> Box<dyn Mapper> {
    let file = fs::File::open(path).unwrap();

    let mut reader: std::io::BufReader<fs::File> = std::io::BufReader::new(file);
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
            let mapper = Mapper0::new(prg_rom);
            Box::new(mapper)
        }
        _ => {
            panic!("Unsupported mapper number: {}", mapper_number);
        }
    }
}