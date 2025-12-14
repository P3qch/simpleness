mod cpu;
mod memory;
mod ppu;
use std::{cell::RefCell, rc::Rc};

use cpu::olc6502::Olc6502;

fn main() {
    let mut bus = memory::bus::Bus::new();

    let rom_content = std::fs::read("roms/nestest.nes").expect("Failed to read ROM file");
    let mapper = memory::mapper::parse_rom(rom_content);

    bus.set_mapper(Rc::new(RefCell::new(mapper)));

    let mut cpu = Olc6502::new(bus);

    cpu.reset();

    loop {
        // Emulation loop
        cpu.execute_instruction();
    }
}
