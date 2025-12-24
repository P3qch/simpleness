use crate::joypad::Joypad;
use crate::memory::mapper::SharedMapper;
use crate::ppu::{OAMDMA, Ppu};
const INTERNAL_RAM_SIZE: usize = 0x800;

const JOY1: u16 = 0x4016;
const JOY2: u16 = 0x4017;

pub struct Bus {
    pub ppu: Ppu,
    pub joypad1: Joypad,
    pub joypad2: Joypad,
    internal_ram: [u8; INTERNAL_RAM_SIZE],
    mapper: Option<SharedMapper>,
}

// For now we only support nrom (no mapper)
impl Bus {
    pub fn new() -> Self {
        Self {
            ppu: Ppu::new(),
            joypad1: Joypad::new(),
            joypad2: Joypad::new(),
            internal_ram: [0xff; INTERNAL_RAM_SIZE],
            mapper: None,
        }
    }

    pub fn set_mapper(&mut self, mapper: SharedMapper) {
        self.mapper = Some(mapper);
        self.ppu.set_mapper(self.mapper.as_ref().unwrap().clone());
    }

    pub fn read_u8(&mut self, addr: u16) -> u8 {
        if self.mapper.is_none() {
            panic!("Attempted to read from bus before loading ROM");
        }
        let mapper = self.mapper.as_ref().unwrap().borrow();

        match addr {
            0x0000..=0x1FFF => self.internal_ram[addr as usize & (INTERNAL_RAM_SIZE - 1)],
            0x2000..=0x3FFF => {
                let ppu_register_addr = 0x2000 + (addr % 8);
                self.ppu.read_register(ppu_register_addr)
            }
            JOY1 => self.joypad1.read_status(),
            JOY2 => self.joypad2.read_status(),
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

    pub fn write_u8(&mut self, addr: u16, data: u8) -> usize {
        let mut extra_cpu_cycles = 0;
        if self.mapper.is_none() {
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

            JOY1 => {
                self.joypad1.set_shift_register_strobe(data & 1 != 0);
                self.joypad2.set_shift_register_strobe(data & 1 != 0);
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
