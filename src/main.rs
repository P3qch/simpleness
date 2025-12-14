mod cpu;
mod memory;
use cpu::olc6502::Olc6502;

use crate::memory::mapper;

fn main() {
    let mapper_box = mapper::parse_rom("roms/nestest.nes");

    let bus = memory::bus::Bus::new(mapper_box);

    let mut cpu = Olc6502::new(bus);

    cpu.reset();

    loop {
       // Emulation loop
        cpu.execute_instruction();
    }
}
