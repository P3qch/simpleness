use crate::memory::mapper::SharedMapper;

#[derive(Clone, Copy)]
pub enum NametableArrangement {
    Vertical,
    Horizontal,
}

pub struct PPUBus {
    mapper: Option<SharedMapper>,
    nametable_ram: [u8; 0x1000],
    pallette_ram: [u8; 0x20],
    nametable_arrangement: NametableArrangement,
}

impl PPUBus {
    pub fn new(nametable_arrangement: NametableArrangement) -> Self {
        Self {
            mapper: None,
            nametable_ram: [0; 0x1000],
            pallette_ram: [0; 0x20],
            nametable_arrangement,
        }
    }

    pub fn set_mapper(&mut self, mapper: SharedMapper) {
        self.mapper = Some(mapper);
    }

    pub fn set_nametable_arrangement(&mut self, mode: NametableArrangement) {
        self.nametable_arrangement = mode;
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        if self.mapper.is_none() {
            panic!("Attempted to read from PPU bus before loading ROM");
        }
        let mapper = self.mapper.as_ref().unwrap().borrow();

        
        match addr {
            0..=0x1fff => mapper.ppu_map_read(addr),
            0x2000..=0x2fff => {
                let nametable_addr = addr as usize & 0x0fff;
                self.nametable_ram[self.apply_nametable_arrangement(nametable_addr)]
            }
            0x3f00..=0x3fff => {
                let mut pallette_addr = addr as usize & 0x1f;

                if pallette_addr == 0x10
                    || pallette_addr == 0x14
                    || pallette_addr == 0x18
                    || pallette_addr == 0x1C
                {
                    pallette_addr -= 0x10;
                }

                self.pallette_ram[pallette_addr]
            }
            _ => 0,
        }
    }

    fn apply_nametable_arrangement(&self, mut nametable_addr: usize) -> usize {
        match self.nametable_arrangement {
            NametableArrangement::Vertical => {
                if let 0x400..0x800 | 0xc00..0x1000 = nametable_addr {
                    nametable_addr -= 0x400;
                }
            }
            NametableArrangement::Horizontal => {
                if nametable_addr >= 0x800 {
                    nametable_addr -= 0x800;
                }
            }
        }
        nametable_addr
    }
    
    pub fn write_u8(&mut self, addr: u16, data: u8) {
        if self.mapper.is_none() {
            panic!("Attempted to write to PPU bus before loading ROM");
        }
        let mut mapper = self.mapper.as_ref().unwrap().borrow_mut();

        match addr {
            0..=0x1fff => {
                mapper.ppu_map_write(addr, data);
            }
            0x2000..=0x2fff => {
                let nametable_addr = addr as usize & 0x0fff;
                self.nametable_ram[self.apply_nametable_arrangement(nametable_addr)] = data;
            }
            0x3f00..=0x3fff => {
                let mut pallette_addr = addr as usize & 0x1f;

                if pallette_addr == 0x10
                    || pallette_addr == 0x14
                    || pallette_addr == 0x18
                    || pallette_addr == 0x1C
                {
                    pallette_addr -= 0x10;
                }

                self.pallette_ram[pallette_addr] = data;
            }
            _ => {}
        }
    }
}
