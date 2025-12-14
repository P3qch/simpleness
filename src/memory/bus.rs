const INTERNAL_RAM_SIZE: usize = 0x800;

pub struct Bus {
    internal_ram: [u8; INTERNAL_RAM_SIZE],
    mapper: Box<dyn crate::memory::mapper::Mapper>,
}

// For now we only support nrom (no mapper)
impl Bus {
    pub fn new(mapper: Box<dyn crate::memory::mapper::Mapper>) -> Self {
        Self {
            internal_ram: [0xff; INTERNAL_RAM_SIZE],
            mapper,
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                self.internal_ram[addr as usize & (INTERNAL_RAM_SIZE - 1)]
            }
            _ => {
                self.mapper.cpu_map_read(addr)
            }
        }
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.read_u8(addr), self.read_u8(addr.wrapping_add(1))])
    }

    pub fn read_u16_no_page_crossing(&self, addr: u16) -> u16 {
        let lo = self.read_u8(addr); // make sure we don't cross a page!!
        let hi_addr = (addr & 0xff00) | (addr.wrapping_add(1) & 0xff);
        let hi = self.read_u8(hi_addr);
        u16::from_le_bytes([lo, hi])
    }

    pub fn read_buffer(&self, addr: u16, length: u16) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(length as usize);
        for i in 0..length {
            buffer.push(self.read_u8(addr.wrapping_add(i)));
        }
        buffer
    }

    pub fn write_u8(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.internal_ram[addr as usize & (INTERNAL_RAM_SIZE - 1)] = data;
            }
            _ => {
                self.mapper.cpu_map_write(addr, data);
            }
        }
    }
}
