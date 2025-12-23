use crate::memory::mapper::SharedMapper; 

pub struct PPUBus {
    mapper: Option<SharedMapper>,
    nametable_ram: [u8; 0x1000], 
    pallette_ram: [u8; 0x20],    
}

impl PPUBus {
    pub fn new() -> Self {
        Self {
            mapper: None,
            nametable_ram: [0; 0x1000],
            pallette_ram: [0; 0x20],
        }
    }

    pub fn set_mapper(&mut self, mapper: SharedMapper) {
        self.mapper = Some(mapper);
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        if let None = self.mapper {
            panic!("Attempted to read from PPU bus before loading ROM");
        }
        let mapper = self.mapper.as_ref().unwrap().borrow();

        match addr {
            0..=0x1fff => {
                mapper.ppu_map_read(addr)
            }
            0x2000..=0x2fff => {
                let nametable_addr = addr as usize & 0x0fff;
                self.nametable_ram[nametable_addr]
            }
            0x3f00..=0x3fff => {
                let pallette_addr = addr as usize & 0x1f;
                self.pallette_ram[pallette_addr]
            }
            _ => 0
        }
    }

    pub fn write_u8(&mut self, addr: u16, data: u8) {
        if let None = self.mapper {
            panic!("Attempted to write to PPU bus before loading ROM");
        }
        let mut mapper = self.mapper.as_ref().unwrap().borrow_mut();

        match addr {
            0..=0x1fff => {
                mapper.ppu_map_write(addr, data);
            }
            0x2000..=0x2fff => {
                let nametable_addr = addr as usize & 0x0fff;
                self.nametable_ram[nametable_addr] = data;
            }
            0x3f00..=0x3fff => {
                let pallette_addr = addr as usize & 0x1f;
                self.pallette_ram[pallette_addr] = data;
            }
            _ => {}
        }
    }
}