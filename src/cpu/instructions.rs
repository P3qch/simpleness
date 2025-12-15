use std::fmt::Display;
use crate::cpu::olc6502::Olc6502;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AddressingMode {
    Acc, 
    Abs, 
    AbsX, 
    AbsY,
    Imm,
    Impl,
    Ind,
    XInd,
    IndY,
    Rel,
    Zpg,
    ZpgX,
    ZpgY
}

impl AddressingMode {
    pub fn size(&self) -> u8 {
        match self {
            AddressingMode::Acc => 1,
            AddressingMode::Abs => 3,
            AddressingMode::AbsX => 3,
            AddressingMode::AbsY => 3,
            AddressingMode::Imm => 2,
            AddressingMode::Impl => 1,
            AddressingMode::Ind => 3,
            AddressingMode::XInd => 2,
            AddressingMode::IndY => 2,
            AddressingMode::Rel => 2,
            AddressingMode::Zpg => 2,
            AddressingMode::ZpgX => 2,
            AddressingMode::ZpgY => 2,
        }
    }

    pub fn format_operand(&self, operand: u16, cpu: &mut Olc6502) -> String {
        match self {
            AddressingMode::Acc => String::from("A"),
            AddressingMode::Abs => format!("${:04X}", operand),
            AddressingMode::AbsX => format!("${:04X},X", operand),
            AddressingMode::AbsY => format!("${:04X},Y", operand),
            AddressingMode::Imm => format!("#${:02X}", cpu.bus.read_u8(operand)),
            AddressingMode::Impl => String::from(""),
            AddressingMode::Ind => format!("(${:04X})", operand),
            AddressingMode::XInd => format!("(${:02X},X)", operand as u8),
            AddressingMode::IndY => format!("(${:02X}),Y", operand as u8),
            AddressingMode::Rel => format!("${:04X}", operand),
            AddressingMode::Zpg => format!("${:02X}", operand as u8),
            AddressingMode::ZpgX => format!("${:02X},X", operand as u8),
            AddressingMode::ZpgY => format!("${:02X},Y", operand as u8),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let instr_str = match self {
            Instruction::ADC => "ADC",
            Instruction::AND => "AND",
            Instruction::ASL => "ASL",
            Instruction::BCC => "BCC",
            Instruction::BCS => "BCS",
            Instruction::BEQ => "BEQ",
            Instruction::BIT => "BIT",
            Instruction::BMI => "BMI",
            Instruction::BNE => "BNE",
            Instruction::BPL => "BPL",
            Instruction::BRK => "BRK",
            Instruction::BVC => "BVC",
            Instruction::BVS => "BVS",
            Instruction::CLC => "CLC",
            Instruction::CLD => "CLD",
            Instruction::CLI => "CLI",
            Instruction::CLV => "CLV",
            Instruction::CMP => "CMP",
            Instruction::CPX => "CPX",
            Instruction::CPY => "CPY",
            Instruction::DEC => "DEC",
            Instruction::DEX => "DEX",
            Instruction::DEY => "DEY",
            Instruction::EOR => "EOR",
            Instruction::INC => "INC",
            Instruction::INX => "INX",
            Instruction::INY => "INY",
            Instruction::JMP => "JMP",
            Instruction::JSR => "JSR",
            Instruction::LDA => "LDA",
            Instruction::LDX => "LDX",
            Instruction::LDY => "LDY",
            Instruction::LSR => "LSR",
            Instruction::NOP => "NOP",
            Instruction::ORA => "ORA",
            Instruction::PHA => "PHA",
            Instruction::PHP => "PHP",
            Instruction::PLA => "PLA",
            Instruction::PLP => "PLP",
            Instruction::ROL => "ROL",
            Instruction::ROR => "ROR",
            Instruction::RTI => "RTI",
            Instruction::RTS => "RTS",
            Instruction::SBC => "SBC",
            Instruction::SEC => "SEC",
            Instruction::SED => "SED",
            Instruction::SEI => "SEI",
            Instruction::STA => "STA",
            Instruction::STX => "STX",
            Instruction::STY => "STY",
            Instruction::TAX => "TAX",
            Instruction::TAY => "TAY",
            Instruction::TSX => "TSX",
            Instruction::TXA => "TXA",
            Instruction::TXS => "TXS",
            Instruction::TYA => "TYA",
        };
        write!(f, "{}", instr_str)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Opcode {
    pub code: u8,
    pub instr: Instruction,
    pub mode: AddressingMode,
    pub cycles: u8,
    pub cross_cycle: bool
}

pub const OPCODES: &[Opcode] = &[
    // ADC
    Opcode { code: 0x69,  instr: Instruction::ADC, mode: AddressingMode::Imm, cycles: 2, cross_cycle: false},
    Opcode { code: 0x65,  instr: Instruction::ADC, mode: AddressingMode::Zpg, cycles: 3, cross_cycle: false},
    Opcode { code: 0x75,  instr: Instruction::ADC, mode: AddressingMode::ZpgX, cycles: 4, cross_cycle: false},
    Opcode { code: 0x6d,  instr: Instruction::ADC, mode: AddressingMode::Abs, cycles: 4, cross_cycle: false},
    Opcode { code: 0x7d,  instr: Instruction::ADC, mode: AddressingMode::AbsX, cycles: 4, cross_cycle: true},
    Opcode { code: 0x79,  instr: Instruction::ADC, mode: AddressingMode::AbsY, cycles: 4, cross_cycle: true},
    Opcode { code: 0x61,  instr: Instruction::ADC, mode: AddressingMode::XInd, cycles: 6, cross_cycle: false},
    Opcode { code: 0x71,  instr: Instruction::ADC, mode: AddressingMode::IndY, cycles: 5, cross_cycle: true},

    // AND
    Opcode { code: 0x29, instr: Instruction::AND, mode: AddressingMode::Imm,  cycles: 2, cross_cycle: false },
    Opcode { code: 0x25, instr: Instruction::AND, mode: AddressingMode::Zpg,  cycles: 3, cross_cycle: false },
    Opcode { code: 0x35, instr: Instruction::AND, mode: AddressingMode::ZpgX, cycles: 4, cross_cycle: false },
    Opcode { code: 0x2d, instr: Instruction::AND, mode: AddressingMode::Abs,  cycles: 4, cross_cycle: false },
    Opcode { code: 0x3d, instr: Instruction::AND, mode: AddressingMode::AbsX, cycles: 4, cross_cycle: true },
    Opcode { code: 0x39, instr: Instruction::AND, mode: AddressingMode::AbsY, cycles: 4, cross_cycle: true },
    Opcode { code: 0x21, instr: Instruction::AND, mode: AddressingMode::XInd, cycles: 6, cross_cycle: false },
    Opcode { code: 0x31, instr: Instruction::AND, mode: AddressingMode::IndY, cycles: 5, cross_cycle: true },

    // ASL
    Opcode { code: 0x0a, instr: Instruction::ASL, mode: AddressingMode::Acc,  cycles: 2, cross_cycle: false },
    Opcode { code: 0x06, instr: Instruction::ASL, mode: AddressingMode::Zpg,  cycles: 5, cross_cycle: false },
    Opcode { code: 0x16, instr: Instruction::ASL, mode: AddressingMode::ZpgX, cycles: 6, cross_cycle: false },
    Opcode { code: 0x0e, instr: Instruction::ASL, mode: AddressingMode::Abs,  cycles: 6, cross_cycle: false },
    Opcode { code: 0x1e, instr: Instruction::ASL, mode: AddressingMode::AbsX, cycles: 7, cross_cycle: false },

    // BCC
    Opcode { code: 0x90, instr: Instruction::BCC, mode: AddressingMode::Rel, cycles: 2, cross_cycle: true },

    // BCS
    Opcode { code: 0xB0, instr: Instruction::BCS, mode: AddressingMode::Rel, cycles: 2, cross_cycle: true },

    // BEQ
    Opcode { code: 0xf0, instr: Instruction::BEQ, mode: AddressingMode::Rel, cycles: 2, cross_cycle: true },
    
    // BIT
    Opcode { code: 0x24, instr: Instruction::BIT, mode: AddressingMode::Zpg, cycles: 3, cross_cycle: false },
    Opcode { code: 0x2c, instr: Instruction::BIT, mode: AddressingMode::Abs, cycles: 4, cross_cycle: false },  
    
    // BMI
    Opcode { code: 0x30, instr: Instruction::BMI, mode: AddressingMode::Rel, cycles: 2, cross_cycle: true },
     
    // BNE
    Opcode { code: 0xD0, instr: Instruction::BNE, mode: AddressingMode::Rel, cycles: 2, cross_cycle: true },
    
    // BPL
    Opcode { code: 0x10, instr: Instruction::BPL, mode: AddressingMode::Rel, cycles: 2, cross_cycle: true },

    // BRK - it is actually Implied but using Imm to simplify fetch of next PC
    Opcode { code: 0x00, instr: Instruction::BRK, mode: AddressingMode::Imm, cycles: 7, cross_cycle: false },

    // BVC
    Opcode { code: 0x50, instr: Instruction::BVC, mode: AddressingMode::Rel, cycles: 2, cross_cycle: true },

    // BVS
    Opcode { code: 0x70, instr: Instruction::BVS, mode: AddressingMode::Rel, cycles: 2, cross_cycle: true },

    // CLC
    Opcode { code: 0x18, instr: Instruction::CLC, mode: AddressingMode::Impl, cycles: 2, cross_cycle: false },

    // CLD
    Opcode { code: 0xD8, instr: Instruction::CLD, mode: AddressingMode::Impl, cycles: 2, cross_cycle: false },

    // CLI
    Opcode { code: 0x58, instr: Instruction::CLI, mode: AddressingMode::Impl, cycles: 2, cross_cycle: false },

    // CLV
    Opcode { code: 0xB8, instr: Instruction::CLV, mode: AddressingMode::Impl, cycles: 2, cross_cycle: false },

    // CMP
    Opcode { code: 0xC9, instr: Instruction::CMP, mode: AddressingMode::Imm,  cycles: 2, cross_cycle: false },
    Opcode { code: 0xC5, instr: Instruction::CMP, mode: AddressingMode::Zpg,  cycles: 3, cross_cycle: false },
    Opcode { code: 0xD5, instr: Instruction::CMP, mode: AddressingMode::ZpgX, cycles: 4, cross_cycle: false },
    Opcode { code: 0xCD, instr: Instruction::CMP, mode: AddressingMode::Abs,  cycles: 4, cross_cycle: false },
    Opcode { code: 0xDD, instr: Instruction::CMP, mode: AddressingMode::AbsX, cycles: 4, cross_cycle: true },
    Opcode { code: 0xD9, instr: Instruction::CMP, mode: AddressingMode::AbsY, cycles: 4, cross_cycle: true },
    Opcode { code: 0xC1, instr: Instruction::CMP, mode: AddressingMode::XInd, cycles: 6, cross_cycle: false },
    Opcode { code: 0xD1, instr: Instruction::CMP, mode: AddressingMode::IndY, cycles: 5, cross_cycle: true }, 

    // CPX
    Opcode { code: 0xE0, instr: Instruction::CPX, mode: AddressingMode::Imm,  cycles: 2, cross_cycle: false },
    Opcode { code: 0xE4, instr: Instruction::CPX, mode: AddressingMode::Zpg,  cycles: 3, cross_cycle: false },
    Opcode { code: 0xEC, instr: Instruction::CPX, mode: AddressingMode::Abs,  cycles: 4, cross_cycle: false },

    // CPY
    Opcode { code: 0xC0, instr: Instruction::CPY, mode: AddressingMode::Imm,  cycles: 2, cross_cycle: false },  
    Opcode { code: 0xC4, instr: Instruction::CPY, mode: AddressingMode::Zpg,  cycles: 3, cross_cycle: false },
    Opcode { code: 0xCC, instr: Instruction::CPY, mode: AddressingMode::Abs,  cycles: 4, cross_cycle: false },

    // DEC
    Opcode { code: 0xC6, instr: Instruction::DEC, mode: AddressingMode::Zpg,  cycles: 5, cross_cycle: false },
    Opcode { code: 0xD6, instr: Instruction::DEC, mode: AddressingMode::ZpgX, cycles: 6, cross_cycle: false },
    Opcode { code: 0xCE, instr: Instruction::DEC, mode: AddressingMode::Abs,  cycles: 6, cross_cycle: false },
    Opcode { code: 0xDE, instr: Instruction::DEC, mode: AddressingMode::AbsX, cycles: 7, cross_cycle: false },

    // DEX
    Opcode { code: 0xCA, instr: Instruction::DEX, mode: AddressingMode::Impl, cycles: 2, cross_cycle: false },

    // DEY
    Opcode { code: 0x88, instr: Instruction::DEY, mode: AddressingMode::Impl, cycles: 2, cross_cycle: false },

    // EOR
    Opcode { code: 0x49, instr: Instruction::EOR, mode: AddressingMode::Imm,  cycles: 2, cross_cycle: false },
    Opcode { code: 0x45, instr: Instruction::EOR, mode: AddressingMode::Zpg,  cycles: 3, cross_cycle: false },
    Opcode { code: 0x55, instr: Instruction::EOR, mode: AddressingMode::ZpgX, cycles: 4, cross_cycle: false },
    Opcode { code: 0x4d, instr: Instruction::EOR, mode: AddressingMode::Abs,  cycles: 4, cross_cycle: false },
    Opcode { code: 0x5d, instr: Instruction::EOR, mode: AddressingMode::AbsX, cycles: 4, cross_cycle: true },
    Opcode { code: 0x59, instr: Instruction::EOR, mode: AddressingMode::AbsY, cycles: 4, cross_cycle: true },
    Opcode { code: 0x41, instr: Instruction::EOR, mode: AddressingMode::XInd, cycles: 6, cross_cycle: false },
    Opcode { code: 0x51, instr: Instruction::EOR, mode: AddressingMode::IndY, cycles: 5, cross_cycle: true },

    // INC
    Opcode { code: 0xE6, instr: Instruction::INC, mode: AddressingMode::Zpg,  cycles: 5, cross_cycle: false },
    Opcode { code: 0xF6, instr: Instruction::INC, mode: AddressingMode::ZpgX, cycles: 6, cross_cycle: false },
    Opcode { code: 0xEE, instr: Instruction::INC, mode: AddressingMode::Abs,  cycles: 6, cross_cycle: false },
    Opcode { code: 0xFE, instr: Instruction::INC, mode: AddressingMode::AbsX, cycles: 7, cross_cycle: false },

    // INX
    Opcode { code: 0xE8, instr: Instruction::INX, mode: AddressingMode::Impl, cycles: 2, cross_cycle: false },  

    // INY
    Opcode { code: 0xC8, instr: Instruction::INY, mode: AddressingMode::Impl, cycles: 2, cross_cycle: false },

    // JMP
    Opcode { code: 0x4C, instr: Instruction::JMP, mode: AddressingMode::Abs,  cycles: 3, cross_cycle: false },
    Opcode { code: 0x6C, instr: Instruction::JMP, mode: AddressingMode::Ind,  cycles: 5, cross_cycle: false },

    // JSR
    Opcode { code: 0x20, instr: Instruction::JSR, mode: AddressingMode::Abs,  cycles: 6, cross_cycle: false },

    // LDA
    Opcode { code: 0xA9, instr: Instruction::LDA, mode: AddressingMode::Imm,  cycles: 2, cross_cycle: false },
    Opcode { code: 0xA5, instr: Instruction::LDA, mode: AddressingMode::Zpg,  cycles: 3, cross_cycle: false },
    Opcode { code: 0xB5, instr: Instruction::LDA, mode: AddressingMode::ZpgX, cycles: 4, cross_cycle: false },
    Opcode { code: 0xAD, instr: Instruction::LDA, mode: AddressingMode::Abs,  cycles: 4, cross_cycle: false },
    Opcode { code: 0xBD, instr: Instruction::LDA, mode: AddressingMode::AbsX, cycles: 4, cross_cycle: true },
    Opcode { code: 0xB9, instr: Instruction::LDA, mode: AddressingMode::AbsY, cycles: 4, cross_cycle: true },
    Opcode { code: 0xA1, instr: Instruction::LDA, mode: AddressingMode::XInd, cycles: 6, cross_cycle: false },
    Opcode { code: 0xB1, instr: Instruction::LDA, mode: AddressingMode::IndY, cycles: 5, cross_cycle: true },

    // LDX
    Opcode { code: 0xA2, instr: Instruction::LDX, mode: AddressingMode::Imm,  cycles: 2, cross_cycle: false },
    Opcode { code: 0xA6, instr: Instruction::LDX, mode: AddressingMode::Zpg,  cycles: 3, cross_cycle: false },
    Opcode { code: 0xB6, instr: Instruction::LDX, mode: AddressingMode::ZpgY, cycles: 4, cross_cycle: false },
    Opcode { code: 0xAE, instr: Instruction::LDX, mode: AddressingMode::Abs,  cycles: 4, cross_cycle: false },
    Opcode { code: 0xBE, instr: Instruction::LDX, mode: AddressingMode::AbsY, cycles: 4, cross_cycle: true },

    // LDY
    Opcode { code: 0xA0, instr: Instruction::LDY, mode: AddressingMode::Imm,  cycles: 2, cross_cycle: false },
    Opcode { code: 0xA4, instr: Instruction::LDY, mode: AddressingMode::Zpg,  cycles: 3, cross_cycle: false },
    Opcode { code: 0xB4, instr: Instruction::LDY, mode: AddressingMode::ZpgX, cycles: 4, cross_cycle: false },
    Opcode { code: 0xAC, instr: Instruction::LDY, mode: AddressingMode::Abs,  cycles: 4, cross_cycle: false },
    Opcode { code: 0xBC, instr: Instruction::LDY, mode: AddressingMode::AbsX, cycles: 4, cross_cycle: true },
    
    // LSR
    Opcode { code: 0x4A, instr: Instruction::LSR, mode: AddressingMode::Acc,  cycles: 2, cross_cycle: false },
    Opcode { code: 0x46, instr: Instruction::LSR, mode: AddressingMode::Zpg,  cycles: 5, cross_cycle: false },
    Opcode { code: 0x56, instr: Instruction::LSR, mode: AddressingMode::ZpgX, cycles: 6, cross_cycle: false },
    Opcode { code: 0x4E, instr: Instruction::LSR, mode: AddressingMode::Abs,  cycles: 6, cross_cycle: false },
    Opcode { code: 0x5E, instr: Instruction::LSR, mode: AddressingMode::AbsX, cycles: 7, cross_cycle: false },

    // NOP
    Opcode { code: 0xEA, instr: Instruction::NOP, mode: AddressingMode::Impl, cycles: 2, cross_cycle: false },

    // ORA
    Opcode { code: 0x09, instr: Instruction::ORA, mode: AddressingMode::Imm,  cycles: 2, cross_cycle: false },
    Opcode { code: 0x05, instr: Instruction::ORA, mode: AddressingMode::Zpg,  cycles: 3, cross_cycle: false },
    Opcode { code: 0x15, instr: Instruction::ORA, mode: AddressingMode::ZpgX, cycles: 4, cross_cycle: false },
    Opcode { code: 0x0D, instr: Instruction::ORA, mode: AddressingMode::Abs,  cycles: 4, cross_cycle: false },
    Opcode { code: 0x1D, instr: Instruction::ORA, mode: AddressingMode::AbsX, cycles: 4, cross_cycle: true },
    Opcode { code: 0x19, instr: Instruction::ORA, mode: AddressingMode::AbsY, cycles: 4, cross_cycle: true },
    Opcode { code: 0x01, instr: Instruction::ORA, mode: AddressingMode::XInd, cycles: 6, cross_cycle: false },
    Opcode { code: 0x11, instr: Instruction::ORA, mode: AddressingMode::IndY, cycles: 5, cross_cycle: true },

    // PHA
    Opcode { code: 0x48, instr: Instruction::PHA, mode: AddressingMode::Impl, cycles: 3, cross_cycle: false },

    // PHP
    Opcode { code: 0x08, instr: Instruction::PHP, mode: AddressingMode::Impl, cycles: 3, cross_cycle: false },
    
    // PLA
    Opcode { code: 0x68, instr: Instruction::PLA, mode: AddressingMode::Impl, cycles: 4, cross_cycle: false },

    // PLP
    Opcode { code: 0x28, instr: Instruction::PLP, mode: AddressingMode::Impl, cycles: 4, cross_cycle: false },

    // ROL
    Opcode { code: 0x2A, instr: Instruction::ROL, mode: AddressingMode::Acc,  cycles: 2, cross_cycle: false },
    Opcode { code: 0x26, instr: Instruction::ROL, mode: AddressingMode::Zpg,  cycles: 5, cross_cycle: false },
    Opcode { code: 0x36, instr: Instruction::ROL, mode: AddressingMode::ZpgX, cycles: 6, cross_cycle: false },
    Opcode { code: 0x2E, instr: Instruction::ROL, mode: AddressingMode::Abs,  cycles: 6, cross_cycle: false },
    Opcode { code: 0x3E, instr: Instruction::ROL, mode: AddressingMode::AbsX, cycles: 7, cross_cycle: false },
    
    // ROR
    Opcode { code: 0x6A, instr: Instruction::ROR, mode: AddressingMode::Acc,  cycles: 2, cross_cycle: false },
    Opcode { code: 0x66, instr: Instruction::ROR, mode: AddressingMode::Zpg,  cycles: 5, cross_cycle: false },
    Opcode { code: 0x76, instr: Instruction::ROR, mode: AddressingMode::ZpgX, cycles: 6, cross_cycle: false },
    Opcode { code: 0x6E, instr: Instruction::ROR, mode: AddressingMode::Abs,  cycles: 6, cross_cycle: false },
    Opcode { code: 0x7E, instr: Instruction::ROR, mode: AddressingMode::AbsX, cycles: 7, cross_cycle: false },  

    // RTI
    Opcode { code: 0x40, instr: Instruction::RTI, mode: AddressingMode::Impl, cycles: 6, cross_cycle: false },

    // RTS
    Opcode { code: 0x60, instr: Instruction::RTS, mode: AddressingMode::Impl, cycles: 6, cross_cycle: false },

    // SBC
    Opcode { code: 0xE9, instr: Instruction::SBC, mode: AddressingMode::Imm,  cycles: 2, cross_cycle: false },
    Opcode { code: 0xE5, instr: Instruction::SBC, mode: AddressingMode::Zpg,  cycles: 3, cross_cycle: false },
    Opcode { code: 0xF5, instr: Instruction::SBC, mode: AddressingMode::ZpgX, cycles: 4, cross_cycle: false },
    Opcode { code: 0xED, instr: Instruction::SBC, mode: AddressingMode::Abs,  cycles: 4, cross_cycle: false },
    Opcode { code: 0xFD, instr: Instruction::SBC, mode: AddressingMode::AbsX, cycles: 4, cross_cycle: true },
    Opcode { code: 0xF9, instr: Instruction::SBC, mode: AddressingMode::AbsY, cycles: 4, cross_cycle: true },
    Opcode { code: 0xE1, instr: Instruction::SBC, mode: AddressingMode::XInd, cycles: 6, cross_cycle: false },
    Opcode { code: 0xF1, instr: Instruction::SBC, mode: AddressingMode::IndY, cycles: 5, cross_cycle: true },


    // SEC
    Opcode { code: 0x38, instr: Instruction::SEC, mode: AddressingMode::Impl, cycles: 2, cross_cycle: false },
    
    // SED
    Opcode { code: 0xF8, instr: Instruction::SED, mode: AddressingMode::Impl, cycles: 2, cross_cycle: false },

    // SEI
    Opcode { code: 0x78, instr: Instruction::SEI, mode: AddressingMode::Impl, cycles: 2, cross_cycle: false },

    // STA
    Opcode { code: 0x85, instr: Instruction::STA, mode: AddressingMode::Zpg,  cycles: 3, cross_cycle: false },
    Opcode { code: 0x95, instr: Instruction::STA, mode: AddressingMode::ZpgX, cycles: 4, cross_cycle: false },
    Opcode { code: 0x8D, instr: Instruction::STA, mode: AddressingMode::Abs,  cycles: 4, cross_cycle: false },
    Opcode { code: 0x9D, instr: Instruction::STA, mode: AddressingMode::AbsX, cycles: 5, cross_cycle: false },
    Opcode { code: 0x99, instr: Instruction::STA, mode: AddressingMode::AbsY, cycles: 5, cross_cycle: false },
    Opcode { code: 0x81, instr: Instruction::STA, mode: AddressingMode::XInd, cycles: 6, cross_cycle: false },
    Opcode { code: 0x91, instr: Instruction::STA, mode: AddressingMode::IndY, cycles: 6, cross_cycle: false },

    // STX
    Opcode { code: 0x86, instr: Instruction::STX, mode: AddressingMode::Zpg,  cycles: 3, cross_cycle: false },
    Opcode { code: 0x96, instr: Instruction::STX, mode: AddressingMode::ZpgY, cycles: 4, cross_cycle: false },
    Opcode { code: 0x8E, instr: Instruction::STX, mode: AddressingMode::Abs,  cycles: 4, cross_cycle: false },

    // STY
    Opcode { code: 0x84, instr: Instruction::STY, mode: AddressingMode::Zpg,  cycles: 3, cross_cycle: false },
    Opcode { code: 0x94, instr: Instruction::STY, mode: AddressingMode::ZpgX, cycles: 4, cross_cycle: false },
    Opcode { code: 0x8C, instr: Instruction::STY, mode: AddressingMode::Abs,  cycles: 4, cross_cycle: false },
    
    // were here
    // TAX
    Opcode { code: 0xAA, instr: Instruction::TAX, mode: AddressingMode::Impl, cycles: 2, cross_cycle: false },

    // TAY
    Opcode { code: 0xA8, instr: Instruction::TAY, mode: AddressingMode::Impl, cycles: 2, cross_cycle: false },

    // TSX
    Opcode { code: 0xBA, instr: Instruction::TSX, mode: AddressingMode::Impl, cycles: 2, cross_cycle: false },

    // TXA
    Opcode { code: 0x8A, instr: Instruction::TXA, mode: AddressingMode::Impl, cycles: 2, cross_cycle: false },

    // TXS
    Opcode { code: 0x9A, instr: Instruction::TXS, mode: AddressingMode::Impl, cycles: 2, cross_cycle: false },

    // TYA
    Opcode { code: 0x98, instr: Instruction::TYA, mode: AddressingMode::Impl, cycles: 2, cross_cycle: false },

    
    ];