mod cpu;
mod memory;
use cpu::olc6502::Olc6502;

fn main() {
    let bus = memory::bus::Bus::new();
    let mut _cpu = Olc6502::new(bus);
    _cpu.pc = 0xc000;
    loop {
       // Emulation loop
        _cpu.execute_instruction();
    }
}
