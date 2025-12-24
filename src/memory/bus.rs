use crate::memory::mapper::SharedMapper;
use crate::ppu::{OAMDMA, PPU};

const INTERNAL_RAM_SIZE: usize = 0x800;

pub struct Bus {
    internal_ram: [u8; INTERNAL_RAM_SIZE],
    mapper: Option<SharedMapper>,
    pub ppu: PPU,
}

// For now we only support nrom (no mapper)
impl Bus {
    pub fn new() -> Self {
        Self {
            internal_ram: [0xff; INTERNAL_RAM_SIZE],
            mapper: None,
            ppu: PPU::new(),
        }
    }

    pub fn set_mapper(&mut self, mapper: SharedMapper) {
        self.mapper = Some(mapper);
        self.ppu.set_mapper(self.mapper.as_ref().unwrap().clone());
    }

    pub fn read_u8(&mut self, addr: u16) -> u8 {
        if let None = self.mapper {
            panic!("Attempted to read from bus before loading ROM");
        }
        let mapper = self.mapper.as_ref().unwrap().borrow();

        match addr {
            0x0000..=0x1FFF => self.internal_ram[addr as usize & (INTERNAL_RAM_SIZE - 1)],
            0x2000..=0x3FFF => {
                let ppu_register_addr = 0x2000 + (addr % 8);
                self.ppu.read_register(ppu_register_addr)
            }
            _ => mapper.cpu_map_read(addr),
        }
    }

    pub fn read_u16(&mut self, addr: u16) -> u16 {
        u16::from_le_bytes([self.read_u8(addr), self.read_u8(addr.wrapping_add(1))])
    }

    pub fn read_u16_no_page_crossing(&mut self, addr: u16) -> u16 {
        let lo = self.read_u8(addr); // make sure we don't cross a page!!
        let hi_addr = (addr & 0xff00) | (addr.wrapping_add(1) & 0xff);
        let hi = self.read_u8(hi_addr);
        u16::from_le_bytes([lo, hi])
    }

    pub fn read_buffer(&mut self, addr: u16, length: u16) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(length as usize);
        for i in 0..length {
            buffer.push(self.read_u8(addr.wrapping_add(i)));
        }
        buffer
    }

    pub fn write_u8(&mut self, addr: u16, data: u8) -> usize {
        let mut extra_cpu_cycles = 0;
        if let None = self.mapper {
            panic!("Attempted to write to bus before loading ROM");
        }

        match addr {
            0x0000..=0x1FFF => {
                self.internal_ram[addr as usize & (INTERNAL_RAM_SIZE - 1)] = data;
            }
            0x2000..=0x3FFF => {
                let ppu_register_addr = 0x2000 + (addr % 8);
                self.ppu.write_register(ppu_register_addr, data);
            }

            OAMDMA => {
                // Perform OAM DMA transfer
                let base_addr = (data as u16) << 8;
                extra_cpu_cycles += 513; // 513 or 514 cycles depending on CPU cycle alignment
                for i in 0..0x100 {
                    let byte = self.read_u8(base_addr.wrapping_add(i));

                    for _ in 0..3 {
                        self.ppu.tick();
                    }

                    self.ppu.write_register(0x2004, byte); // OAMDATA register

                    for _ in 0..3 {
                        self.ppu.tick();
                    }
                }
            }

            _ => {
                let mut mapper = self.mapper.as_ref().unwrap().borrow_mut();
                mapper.cpu_map_write(addr, data);
            }
        }
        extra_cpu_cycles
    }
}
