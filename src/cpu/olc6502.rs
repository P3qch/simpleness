use crate::cpu::instructions::{AddressingMode, Instruction, OPCODE_MAP};
use crate::memory::bus::Bus;
use bitflags::bitflags;

const NMI_ADDRESS: u16 = 0xfffa;
const RESET_ADDRESS: u16 = 0xfffc;
const IRQ_ADDRESS: u16 = 0xfffe;

bitflags! {
    /// Represents a set of flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct StatusFlags: u8 {
        const C = 1 << 0; // carry
        const Z = 1 << 1; // zero
        const I = 1 << 2; // interrupt disable
        const D = 1 << 3; // decimal mode (unused here)
        const B = 1 << 4; // break
        const U = 1 << 5; // unused
        const V = 1 << 6; // overflow
        const N = 1 << 7; // negative
    }
}

pub struct Olc6502 {
    pub(crate) bus: Bus,

    // Registers
    pub(crate) a: u8,
    pub(crate) x: u8,
    pub(crate) y: u8,
    p: StatusFlags,
    s: u8,
    pub pc: u16,

    // instruction argument
    operand: u16,
    cycles: u64,
}

impl Olc6502 {
    pub fn new(bus: Bus) -> Self {
        Self {
            bus,
            a: 0,
            x: 0,
            y: 0,
            p: StatusFlags::I | StatusFlags::U,
            s: 0xfd,
            pc: 0,
            operand: 0,

            cycles: 7,
        }
    }

    pub fn tick(&mut self) {
        let cpu_cycles_ran = self.execute_instruction();
        for _ in 0..cpu_cycles_ran * 3 {
            self.bus.ppu.tick();
            
            if self.bus.ppu.should_nmi {
                self.bus.ppu.should_nmi = false;
                self.nmi();
                for _ in 0..6 {
                    self.bus.ppu.tick();
                }
            }
        }

    }

    fn bus_write_u8(&mut self, addr: u16, data: u8) {
        let extra_cycles = self.bus.write_u8(addr, data);
        self.cycles += extra_cycles as u64;
    }

    pub fn reset(&mut self) {
        self.pc = self.bus.read_u16(RESET_ADDRESS);
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.s -= 3;
        self.p |= StatusFlags::I;
    }

    pub fn nmi(&mut self) {
        self.push_u16(self.pc);
        self.push_u8(self.p.bits() | StatusFlags::I.bits()); // we gotta add this B flag here
        self.p.insert(StatusFlags::I);
        self.pc = self.bus.read_u16(NMI_ADDRESS);
        self.cycles += 2;
    }

    pub fn execute_instruction(&mut self) -> u64 {
        let current_byte = self.bus.read_u8(self.pc);

        let opcode_option = OPCODE_MAP.get(&current_byte);

        let opcode = match opcode_option {
            Some(op) => op,
            None => panic!(
                "Unknown opcode: {:02X} at PC: {:04X}",
                current_byte, self.pc
            ),
        };

        self.pc += 1;
        let old_cycles = self.cycles;
        self.handle_addressing(opcode.mode, opcode.cross_cycle);

        // let opcode_bytes = self.bus.read_buffer(self.pc, opcode.mode.size() as u16);
        // let old_pc = self.pc;
        // println!(
        //     "{:04X}  {:02X} {} {}  {} {:28}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
        //     old_pc,
        //     opcode_bytes[0],
        //     if opcode.mode.size() > 1 {format!("{:02X}", opcode_bytes[1])} else {String::from("  ")},
        //     if opcode.mode.size() > 2 {format!("{:02X}", opcode_bytes[2])} else {String::from("  ")} ,
        //     format!("{}", opcode.instr),
        //     opcode.mode.format_operand(self.operand, self),
        //     self.a,
        //     self.x,
        //     self.y,
        //     self.p.bits(),
        //     self.s,
        //     old_cycles
        // );

        match opcode.instr {
            Instruction::ADC => self.inst_adc(),
            Instruction::AND => self.inst_and(),
            Instruction::ASL => {
                if opcode.mode == AddressingMode::Acc {
                    self.inst_asl_accumulator()
                } else {
                    self.inst_asl_memory()
                }
            }
            Instruction::BCC => self.inst_bcc(),
            Instruction::BCS => self.inst_bcs(),
            Instruction::BEQ => self.inst_beq(),
            Instruction::BIT => self.inst_bit(),
            Instruction::BMI => self.inst_bmi(),
            Instruction::BNE => self.inst_bne(),
            Instruction::BPL => self.inst_bpl(),
            Instruction::BRK => self.inst_brk(),
            Instruction::BVC => self.inst_bvc(),
            Instruction::BVS => self.inst_bvs(),
            Instruction::CLC => self.inst_clc(),
            Instruction::CLD => self.inst_cld(),
            Instruction::CLI => self.inst_cli(),
            Instruction::CLV => self.inst_clv(),
            Instruction::CMP => self.inst_cmp(),
            Instruction::CPX => self.inst_cpx(),
            Instruction::CPY => self.inst_cpy(),
            Instruction::DEC => self.inst_dec(),
            Instruction::DEX => self.inst_dex(),
            Instruction::DEY => self.inst_dey(),
            Instruction::EOR => self.inst_eor(),
            Instruction::INC => self.inst_inc(),
            Instruction::INX => self.inst_inx(),
            Instruction::INY => self.inst_iny(),
            Instruction::JMP => self.inst_jmp(),
            Instruction::JSR => self.inst_jsr(),
            Instruction::LDA => self.inst_lda(),
            Instruction::LDX => self.inst_ldx(),
            Instruction::LDY => self.inst_ldy(),
            Instruction::LSR => {
                if opcode.mode == AddressingMode::Acc {
                    self.inst_lsr_accumulator()
                } else {
                    self.inst_lsr_memory()
                }
            }
            Instruction::NOP => (),
            Instruction::ORA => self.inst_ora(),
            Instruction::PHA => self.inst_pha(),
            Instruction::PHP => self.inst_php(),
            Instruction::PLA => self.inst_pla(),
            Instruction::PLP => self.inst_plp(),
            Instruction::ROL => {
                if opcode.mode == AddressingMode::Acc {
                    self.inst_rol_accumulator()
                } else {
                    self.inst_rol_memory()
                }
            }
            Instruction::ROR => {
                if opcode.mode == AddressingMode::Acc {
                    self.inst_ror_accumulator()
                } else {
                    self.inst_ror_memory()
                }
            }
            Instruction::RTI => self.inst_rti(),
            Instruction::RTS => self.inst_rts(),
            Instruction::SBC => self.inst_sbc(),
            Instruction::SEC => self.inst_sec(),
            Instruction::SED => self.inst_sed(),
            Instruction::SEI => self.inst_sei(),
            Instruction::STA => self.inst_sta(),
            Instruction::STX => self.inst_stx(),
            Instruction::STY => self.inst_sty(),
            Instruction::TAX => self.inst_tax(),
            Instruction::TAY => self.inst_tay(),
            Instruction::TSX => self.inst_tsx(),
            Instruction::TXA => self.inst_txa(),
            Instruction::TXS => self.inst_txs(),
            Instruction::TYA => self.inst_tya(),
        }

        self.cycles += opcode.cycles as u64;

        self.cycles - old_cycles
    }

    fn handle_addressing(&mut self, am: AddressingMode, cross_cycle: bool) {
        match am {
            AddressingMode::Abs => {
                self.operand = self.bus.read_u16(self.pc);
                self.pc += 2;
            }
            AddressingMode::AbsX => {
                let base = self.bus.read_u16(self.pc);
                self.operand = base.wrapping_add(self.x as u16);
                self.pc += 2;

                if cross_cycle && ((base & 0x00ff) + (self.x as u16) > 0x00ff) {
                    self.cycles += 1;
                }
            }
            AddressingMode::AbsY => {
                let base = self.bus.read_u16(self.pc);
                self.operand = base.wrapping_add(self.y as u16);
                self.pc += 2;

                if cross_cycle && ((base & 0x00ff) + (self.y as u16) > 0x00ff) {
                    self.cycles += 1;
                }
            }
            AddressingMode::Imm => {
                self.operand = self.pc;
                self.pc += 1;
            }
            AddressingMode::Impl => {}
            AddressingMode::Ind => {
                let address = self.bus.read_u16(self.pc);
                self.operand = self.bus.read_u16_no_page_crossing(address);

                self.pc += 2;
            }
            AddressingMode::XInd => {
                let address = self.bus.read_u8(self.pc).wrapping_add(self.x);
                self.operand = self.bus.read_u16_no_page_crossing(address as u16);
                self.pc += 1;
            }
            AddressingMode::IndY => {
                let address = self.bus.read_u8(self.pc);
                let base = self.bus.read_u16_no_page_crossing(address as u16);
                self.operand = base.wrapping_add(self.y as u16);
                self.pc += 1;

                if cross_cycle && ((base & 0x00ff) + (self.y as u16) > 0x00ff) {
                    self.cycles += 1;
                }
            }
            AddressingMode::Rel => {
                self.operand = self.bus.read_u8(self.pc) as i8 as i16 as u16;
                self.pc += 1;
            }
            AddressingMode::Zpg => {
                self.operand = self.bus.read_u8(self.pc) as u16;
                self.pc += 1;
            }
            AddressingMode::ZpgX => {
                self.operand = self.bus.read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc += 1;
            }
            AddressingMode::ZpgY => {
                self.operand = self.bus.read_u8(self.pc).wrapping_add(self.y) as u16;
                self.pc += 1;
            }
            _ => (),
        };
    }

    //// Instruction helpers ////

    fn branch(&mut self, condition: bool) {
        if condition {
            self.cycles += 1;
            let old_pc = self.pc;
            self.pc = self.pc.wrapping_add_signed(self.operand as i16);

            if (old_pc & 0xff00) != (self.pc & 0xff00) {
                self.cycles += 1;
            }
        }
    }

    fn push_u8(&mut self, data: u8) {
        self.bus_write_u8(0x0100 | (self.s as u16), data);
        self.s = self.s.wrapping_sub(1);
    }

    fn push_u16(&mut self, data: u16) {
        self.push_u8((data >> 8) as u8);
        self.push_u8((data & 0xff) as u8);
    }

    fn pop_u8(&mut self) -> u8 {
        self.s = self.s.wrapping_add(1);
        self.bus.read_u8(0x0100 | (self.s as u16))
    }

    fn pop_u16(&mut self) -> u16 {
        let lo = self.pop_u8() as u16;
        let hi = self.pop_u8() as u16;
        (hi << 8) | lo
    }

    //// Instructions ////

    fn set_zn_flags(&mut self, value: u8) {
        self.p.set(StatusFlags::Z, value == 0);
        self.p.set(StatusFlags::N, value & (1 << 7) != 0);
    }

    fn inst_adc(&mut self) {
        let memory = self.bus.read_u8(self.operand) as u16;
        let carry = self.p.contains(StatusFlags::C) as u16;

        // a sum set to set flags
        let sum_u16 = (self.a as u16) + memory + carry;
        let sum_u8 = (sum_u16 & 0xff) as u8;

        self.p.set(StatusFlags::C, sum_u16 > 0xff);
        self.p.set(
            StatusFlags::V,
            ((sum_u8 ^ self.a) & (sum_u8 ^ memory as u8) & 0x80) != 0,
        );
        self.set_zn_flags(sum_u8);

        self.a = sum_u8;
    }

    fn inst_and(&mut self) {
        let memory = self.bus.read_u8(self.operand);
        self.a &= memory;
        self.set_zn_flags(self.a);
    }

    fn inst_asl_accumulator(&mut self) {
        let old = self.a;
        self.a <<= 1;
        let result = self.a;

        self.set_zn_flags(result);
        self.p.set(StatusFlags::C, old & (1 << 7) != 0);
    }

    fn inst_asl_memory(&mut self) {
        let mut result = self.bus.read_u8(self.operand);
        let old = result;
        result <<= 1;
        self.bus_write_u8(self.operand, result);

        self.set_zn_flags(result);
        self.p.set(StatusFlags::C, old & (1 << 7) != 0);
    }

    fn inst_bcc(&mut self) {
        self.branch(!self.p.contains(StatusFlags::C));
    }

    fn inst_bcs(&mut self) {
        self.branch(self.p.contains(StatusFlags::C));
    }

    fn inst_beq(&mut self) {
        self.branch(self.p.contains(StatusFlags::Z));
    }

    fn inst_bit(&mut self) {
        let memory = self.bus.read_u8(self.operand);
        let res = self.a & memory;

        self.p.set(StatusFlags::Z, res == 0);
        self.p.set(StatusFlags::V, memory & (1 << 6) != 0);
        self.p.set(StatusFlags::N, memory & (1 << 7) != 0);
    }

    fn inst_bmi(&mut self) {
        self.branch(self.p.contains(StatusFlags::N));
    }

    fn inst_bne(&mut self) {
        self.branch(!self.p.contains(StatusFlags::Z));
    }

    fn inst_bpl(&mut self) {
        self.branch(!self.p.contains(StatusFlags::N));
    }

    fn inst_brk(&mut self) {
        // no need to inc pc, as it was incremented because mode is set to immediate here
        self.push_u16(self.pc);
        let status = self.p | StatusFlags::B | StatusFlags::U;
        self.push_u8(status.bits());
        self.p.insert(StatusFlags::I);

        self.pc = self.bus.read_u16(IRQ_ADDRESS);
    }

    fn inst_bvc(&mut self) {
        self.branch(!self.p.contains(StatusFlags::V));
    }

    fn inst_bvs(&mut self) {
        self.branch(self.p.contains(StatusFlags::V));
    }

    fn inst_clc(&mut self) {
        self.p.remove(StatusFlags::C);
    }

    fn inst_cld(&mut self) {
        self.p.remove(StatusFlags::D);
    }

    fn inst_cli(&mut self) {
        self.p.remove(StatusFlags::I);
    }

    fn inst_clv(&mut self) {
        self.p.remove(StatusFlags::V);
    }

    fn inst_cmp(&mut self) {
        let memory = self.bus.read_u8(self.operand);
        let result = self.a.wrapping_sub(memory);

        self.p.set(StatusFlags::C, self.a >= memory);
        self.set_zn_flags(result);
    }

    fn inst_cpx(&mut self) {
        let memory = self.bus.read_u8(self.operand);
        let result = self.x.wrapping_sub(memory);

        self.p.set(StatusFlags::C, self.x >= memory);
        self.set_zn_flags(result);
    }

    fn inst_cpy(&mut self) {
        let memory = self.bus.read_u8(self.operand);
        let result = self.y.wrapping_sub(memory);

        self.p.set(StatusFlags::C, self.y >= memory);
        self.set_zn_flags(result);
    }

    fn inst_dec(&mut self) {
        let mut memory = self.bus.read_u8(self.operand);
        memory = memory.wrapping_sub(1);
        self.bus_write_u8(self.operand, memory);
        self.set_zn_flags(memory);
    }

    fn inst_dex(&mut self) {
        self.x = self.x.wrapping_sub(1);
        self.set_zn_flags(self.x);
    }

    fn inst_dey(&mut self) {
        self.y = self.y.wrapping_sub(1);
        self.set_zn_flags(self.y);
    }
    fn inst_eor(&mut self) {
        let memory = self.bus.read_u8(self.operand);
        self.a ^= memory;
        self.set_zn_flags(self.a);
    }

    fn inst_inc(&mut self) {
        let mut memory = self.bus.read_u8(self.operand);
        memory = memory.wrapping_add(1);
        self.bus_write_u8(self.operand, memory);
        self.set_zn_flags(memory);
    }

    fn inst_inx(&mut self) {
        self.x = self.x.wrapping_add(1);
        self.set_zn_flags(self.x);
    }

    fn inst_iny(&mut self) {
        self.y = self.y.wrapping_add(1);
        self.set_zn_flags(self.y);
    }

    fn inst_jmp(&mut self) {
        self.pc = self.operand;
    }

    fn inst_jsr(&mut self) {
        let ret_address = self.pc.wrapping_sub(1);
        self.push_u16(ret_address);
        self.pc = self.operand;
    }

    fn inst_lda(&mut self) {
        let memory = self.bus.read_u8(self.operand);
        self.a = memory;
        self.set_zn_flags(self.a);
    }

    fn inst_ldx(&mut self) {
        let memory = self.bus.read_u8(self.operand);
        self.x = memory;
        self.set_zn_flags(self.x);
    }

    fn inst_ldy(&mut self) {
        let memory = self.bus.read_u8(self.operand);
        self.y = memory;
        self.set_zn_flags(self.y);
    }

    fn inst_lsr_accumulator(&mut self) {
        let old = self.a;
        self.a >>= 1;
        let result = self.a;

        self.set_zn_flags(result);
        self.p.set(StatusFlags::C, old & 1 != 0);
    }

    fn inst_lsr_memory(&mut self) {
        let mut result = self.bus.read_u8(self.operand);
        let old = result;
        result >>= 1;
        self.bus_write_u8(self.operand, result);

        self.set_zn_flags(result);
        self.p.set(StatusFlags::C, old & 1 != 0);
    }

    fn inst_ora(&mut self) {
        let memory = self.bus.read_u8(self.operand);
        self.a |= memory;
        self.set_zn_flags(self.a);
    }

    fn inst_pha(&mut self) {
        self.push_u8(self.a);
    }

    fn inst_php(&mut self) {
        let status = self.p | StatusFlags::B | StatusFlags::U;
        self.push_u8(status.bits());
    }

    fn inst_pla(&mut self) {
        self.a = self.pop_u8();
        self.set_zn_flags(self.a);
    }

    fn inst_plp(&mut self) {
        let status = self.pop_u8();
        self.p = StatusFlags::from_bits_truncate(status);
        self.p.insert(StatusFlags::U);
        self.p.remove(StatusFlags::B);
    }

    fn inst_rol_accumulator(&mut self) {
        let old = self.a;
        let carry = if self.p.contains(StatusFlags::C) {
            1
        } else {
            0
        };
        self.a = (self.a << 1) | carry;
        let result = self.a;

        self.set_zn_flags(result);
        self.p.set(StatusFlags::C, old & (1 << 7) != 0);
    }

    fn inst_rol_memory(&mut self) {
        let mut result = self.bus.read_u8(self.operand);
        let old = result;
        let carry = if self.p.contains(StatusFlags::C) {
            1
        } else {
            0
        };
        result = (result << 1) | carry;
        self.bus_write_u8(self.operand, result);

        self.set_zn_flags(result);
        self.p.set(StatusFlags::C, old & (1 << 7) != 0);
    }

    fn inst_ror_accumulator(&mut self) {
        let old = self.a;
        let carry = if self.p.contains(StatusFlags::C) {
            1
        } else {
            0
        };
        self.a = (self.a >> 1) | (carry << 7);
        let result = self.a;

        self.set_zn_flags(result);
        self.p.set(StatusFlags::C, old & 1 != 0);
    }

    fn inst_ror_memory(&mut self) {
        let mut result = self.bus.read_u8(self.operand);
        let old = result;
        let carry = if self.p.contains(StatusFlags::C) {
            1
        } else {
            0
        };
        result = (result >> 1) | (carry << 7);
        self.bus_write_u8(self.operand, result);

        self.set_zn_flags(result);
        self.p.set(StatusFlags::C, old & 1 != 0);
    }

    fn inst_rti(&mut self) {
        let status = self.pop_u8();
        self.p = StatusFlags::from_bits_truncate(status);
        // there is no need to remove the I flag as it wasn't enabled when the register
        // was pushed to the stack.
        self.p.remove(StatusFlags::B);
        self.p.insert(StatusFlags::U);
        self.pc = self.pop_u16();
    }

    fn inst_rts(&mut self) {
        self.pc = self.pop_u16().wrapping_add(1);
    }

    fn inst_sbc(&mut self) {
        let memory = self.bus.read_u8(self.operand);
        let carry = 1 - (self.p.contains(StatusFlags::C) as u8);

        let result = self.a.wrapping_sub(memory).wrapping_sub(carry);

        self.set_zn_flags(result);
        self.p.set(
            StatusFlags::V,
            ((self.a ^ memory) & (self.a ^ result) & 0x80) != 0,
        );
        self.p.set(StatusFlags::C, (result as i8) >= 0);

        self.a = result;
    }

    fn inst_sec(&mut self) {
        self.p.insert(StatusFlags::C);
    }

    fn inst_sed(&mut self) {
        self.p.insert(StatusFlags::D);
    }

    fn inst_sei(&mut self) {
        self.p.insert(StatusFlags::I);
    }

    fn inst_sta(&mut self) {
        self.bus_write_u8(self.operand, self.a);
    }

    fn inst_stx(&mut self) {
        self.bus_write_u8(self.operand, self.x);
    }

    fn inst_sty(&mut self) {
        self.bus_write_u8(self.operand, self.y);
    }

    fn inst_tax(&mut self) {
        self.x = self.a;
        self.set_zn_flags(self.x);
    }

    fn inst_tay(&mut self) {
        self.y = self.a;
        self.set_zn_flags(self.y);
    }

    fn inst_tsx(&mut self) {
        self.x = self.s;
        self.set_zn_flags(self.x);
    }

    fn inst_txa(&mut self) {
        self.a = self.x;
        self.set_zn_flags(self.a);
    }

    fn inst_txs(&mut self) {
        self.s = self.x;
    }

    fn inst_tya(&mut self) {
        self.a = self.y;
        self.set_zn_flags(self.a);
    }
}
