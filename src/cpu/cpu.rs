use cpu::opinfo::{OpInfo, OP_INFO};
use cpu::status_register::StatusRegister;
use cpu::utils;
use std::fmt;

use memory::{self, Memory};

type MemoryAddress = u16;
type PageCrossed = bool;

// TODO: set correct address
static STACK_BASE_ADDRESS: u16 = 0x0100;

pub struct OpResponse {
    bytes_consumed: usize,
    cycles_spent: usize,
}

#[derive(Copy, Clone, Debug)]
pub enum AddressingMode {
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndexedIndirect,
    IndirectIndexed,
    Indirect,
}

pub struct CPU {
    accumulator: u8,
    stack_pointer: u8,
    program_counter: u16,
    index_x: u8,
    index_y: u8,
    status_register: StatusRegister,

    /// count the total amount of cycles spent
    cycles: usize,
}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CPU: a:{:#04X}, s:{:#04X}, p:{:#04X}, x:{:#04X}, y:{:#04X}",
            self.accumulator, self.stack_pointer, self.program_counter, self.index_x, self.index_y
        )
    }
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            accumulator: 0,
            stack_pointer: 0,
            program_counter: 0,
            index_x: 0,
            index_y: 0,
            status_register: StatusRegister::new(),

            cycles: 0,
        }
    }

    pub fn powerup(&mut self, mem: &mut Memory) {
        self.status_register.set_all(0x34);
        self.accumulator = 0;
        self.index_x = 0;
        self.index_y = 0;
        self.stack_pointer = 0xFD;

        // Remaining tasks: set memory
        // LSFR = 0x00
        mem.write(0x4017, 0x00);
        mem.write_range(0x4000, 0x400F, 0x00);
    }

    pub fn reset(&mut self, mem: &mut Memory) {
        self.stack_pointer -= 3;
        self.status_register.interrupt_disable = true;

        // Remaining tasks: set memory
        mem.write(0x4015, 0x00);
    }

    fn carry_flag(&self) -> u8 {
        self.status_register.carry_flag as u8
    }

    /// Update the zero flag of the cpus status register. This is a common call
    /// in many instructions.
    fn update_zero_flag(&mut self, r: u8) {
        if r == 0x00 {
            self.status_register.zero_flag = true;
        }
    }

    fn update_negative_flag(&mut self, r: u8) {
        if r >> 7 & 0x01 == 0x01 {
            self.status_register.negative_flag = true;
        } else {
            self.status_register.negative_flag = false;
        }
    }

    /// Get the memory address stored at (actually behind) the current program
    /// counter.
    ///
    /// This method will increment the cpu cycles spent if it detects a page
    /// crossing.
    ///
    /// 1. All instructions have only one argument, so there is only one memory
    ///    address to read per instruction.
    /// 2. Depending on the addressing mode, either one or two bytes have to be
    ///    read from memory.
    ///
    /// # Bugs/TODOs
    ///
    /// 1. detect page crossings. Where?
    ///     * Which modes?
    ///
    fn get_address(&self, mem: &Memory, mode: AddressingMode) -> Option<u16> {
        // TODO: detect page crossings!
        use cpu::cpu::AddressingMode::*;

        match mode {
            Absolute => {
                let a = mem.read(self.program_counter + 1) as u16;
                let b = mem.read(self.program_counter + 2) as u16;

                Some((b << 8) + a)
            }
            AbsoluteX => {
                // TODO: detect page crossing
                let a = mem.read(self.program_counter + 1) as u16;
                let b = mem.read(self.program_counter + 2) as u16;
                let c = (b << 8) + a;

                Some(c + self.index_x as u16)
            }
            AbsoluteY => {
                // TODO: detect page crossing
                let a = mem.read(self.program_counter + 1) as u16;
                let b = mem.read(self.program_counter + 2) as u16;
                let c = (b << 8) + a;

                Some(c + self.index_y as u16)
            }
            Accumulator => None,
            Implicit => None,
            Immediate => Some(self.program_counter + 1),
            IndexedIndirect => {
                let a = mem.read(self.program_counter + 1) as u16;
                let b = a.wrapping_add(self.index_x as u16);
                let c = mem.read(b) as u16;
                let d = mem.read(b + 1) as u16;
                let e = (d << 8) + c;

                Some(e)
            }
            IndirectIndexed => {
                // TODO: detect page crossing
                let a = mem.read(self.program_counter + 1) as u16;
                let b = mem.read(a) as u16;
                let c = mem.read(a + 1) as u16;
                let d = (c << 8) + b;
                let e = d + self.index_y as u16;

                Some(e)
            }
            Relative => {
                let relative_value = mem.read(self.program_counter + 1);
                Some(self.program_counter + relative_value as u16)
            }
            ZeroPage => Some(mem.read(self.program_counter + 1) as u16),
            ZeroPageX => {
                let a = mem.read(self.program_counter + 1);
                let b = a.wrapping_add(self.index_x);

                Some(b as u16)
            }
            ZeroPageY => {
                let a = mem.read(self.program_counter + 1);
                let b = a.wrapping_add(self.index_y);

                Some(b as u16)
            }
            Indirect => {
                // TODO: test
                let addr_lo = mem.read(self.program_counter + 1) as u16;
                let addr_hi = match addr_lo {
                    0xFF => {
                        // bug in 6502
                        0x0000
                    }
                    _ => mem.read(self.program_counter + 2) as u16,
                };

                Some(addr_hi << 8 + addr_lo)
            }
        }
    }

    fn read16(&self, mem: &Memory, addr: u16) -> u16 {
        let hi = mem.read(addr) as u16;
        let lo = mem.read(addr + 1) as u16;

        (hi << 8) + lo
    }

    fn push(&mut self, mem: &mut Memory, val: u8) {
        let addr = STACK_BASE_ADDRESS + self.stack_pointer as u16;
        mem.write(addr, val);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn push16(&mut self, mem: &mut Memory, val: u16) {
        let hi = ((val >> 8) as u8) & 0xFF;
        let lo = (val as u8) & 0xFF;

        self.push(mem, hi);
        self.push(mem, lo);
    }

    fn pop(&mut self, mem: &mut Memory) -> u8 {
        let addr = STACK_BASE_ADDRESS + self.stack_pointer as u16 + 1;
        let val = mem.read(addr);

        self.stack_pointer = self.stack_pointer.wrapping_add(1);

        val
    }

    fn pop16(&mut self, mem: &mut Memory) -> u16 {
        let lo = self.pop(mem) as u16;
        let hi = self.pop(mem) as u16;

        (hi << 8) + lo
    }

    pub fn step(&mut self, mem: &mut Memory) {
        // TODO: check for interrupts
        //

        self.execute_next(mem);
    }

    /// Executes the next instruction stored at the program_counters address.
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn execute_next(&mut self, mem: &mut Memory) {
        use cpu::cpu::AddressingMode::*;

        match mem.read(self.program_counter) {
            // ADC
            0x69 => self.adc(mem, &OpInfo{mode: Immediate, bytes: 2, cycles: 2}),
            0x65 => self.adc(mem, &OpInfo{mode: ZeroPage,  bytes: 2, cycles: 3}),
            0x75 => self.adc(mem, &OpInfo{mode: ZeroPageX, bytes: 2, cycles: 4}),
            0x6D => self.adc(mem, &OpInfo{mode: Absolute,  bytes: 3, cycles: 4}),
            0x7D => self.adc(mem, &OpInfo{mode: AbsoluteX, bytes: 3, cycles: 4}),
            0x79 => self.adc(mem, &OpInfo{mode: AbsoluteY, bytes: 3, cycles: 4}),
            0x61 => self.adc(mem, &OpInfo{mode: IndexedIndirect, bytes: 2, cycles: 6}),
            0x71 => self.adc(mem, &OpInfo{mode: IndirectIndexed, bytes: 2, cycles: 5}),

            // AND
            0x29 => self.and(mem, AddressingMode::Immediate),
            0x25 => self.and(mem, AddressingMode::ZeroPage),
            0x35 => self.and(mem, AddressingMode::ZeroPageX),
            0x2D => self.and(mem, AddressingMode::ZeroPageY),
            0x3D => self.and(mem, AddressingMode::AbsoluteX),
            0x39 => self.and(mem, AddressingMode::AbsoluteY),
            0x21 => self.and(mem, AddressingMode::IndexedIndirect),
            0x31 => self.and(mem, AddressingMode::IndirectIndexed),

            // ASL
            0x0A => self.asl(mem, AddressingMode::Accumulator),
            0x06 => self.asl(mem, AddressingMode::ZeroPage),
            0x16 => self.asl(mem, AddressingMode::ZeroPageX),
            0x0E => self.asl(mem, AddressingMode::Absolute),
            0x1E => self.asl(mem, AddressingMode::AbsoluteX),

            // BCC
            0x90 => self.bcc(mem, &OpInfo{mode: Relative, bytes: 2, cycles: 2}),

            // BCS
            0xB0 => self.bcs(mem, &OpInfo{mode: Relative, bytes: 2, cycles: 2}),

            // BEQ (branch if equal)
            0xF0 => self.beq(mem, &OpInfo{mode: Relative, bytes: 2, cycles: 2}),

            // BIT (bit test)
            0x24 => self.bit(mem, &OpInfo{mode: ZeroPage, bytes: 2, cycles: 3}),
            0x2C => self.bit(mem, &OpInfo{mode: Absolute, bytes: 3, cycles: 4}),

            // BMI (branch if minus)
            0x30 => self.bmi(mem, &OpInfo{mode: Relative, bytes: 2, cycles: 2}),

            // BNE (branch if not equal)
            0xD0 => self.bne(mem, &OpInfo{mode: Relative, bytes: 2, cycles: 2}),

            // BPL (branch if positive)
            0x10 => self.bpl(mem, &OpInfo{mode: Relative, bytes: 2, cycles: 2}),

            // BRK (force interrupt)
            0x00 => self.brk(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 7}),

            // BVC (branch if overflow clear)
            0x50 => self.bvc(mem, &OpInfo{mode: Relative, bytes: 2, cycles: 2}),

            // BVS (branch if overflow set)
            0x70 => self.bvs(mem, &OpInfo{mode: Relative, bytes: 2, cycles: 2}),

            // CLC (clear carry flag)
            0x18 => self.clc(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 2}),

            // CLD (clear decimal mode)
            0xD8 => self.cld(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 2}),

            // CLI (clear interrupt disable)
            0x58 => self.cli(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 2}),

            // CLV (clear overflow flag)
            0xB8 => self.clv(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 2}),

            // CMP (compare)
            0xC9 => self.cmp(mem, &OpInfo{mode: Immediate, bytes: 2, cycles: 2}),
            0xC5 => self.cmp(mem, &OpInfo{mode: ZeroPage, bytes: 2, cycles: 3}),
            0xD5 => self.cmp(mem, &OpInfo{mode: ZeroPageX, bytes: 2, cycles: 4}),
            0xCD => self.cmp(mem, &OpInfo{mode: Absolute, bytes: 3, cycles: 4}),
            0xDD => self.cmp(mem, &OpInfo{mode: AbsoluteX, bytes: 3, cycles: 4}),
            0xD9 => self.cmp(mem, &OpInfo{mode: AbsoluteY, bytes: 3, cycles: 4}),
            0xC1 => self.cmp(mem, &OpInfo{mode: IndexedIndirect, bytes: 2, cycles: 6}),
            0xD1 => self.cmp(mem, &OpInfo{mode: IndirectIndexed, bytes: 2, cycles: 5}),

            // CPX
            0xE0 => self.cpx(mem, &OpInfo{mode: Immediate, bytes: 2, cycles: 2}),
            0xE4 => self.cpx(mem, &OpInfo{mode: ZeroPage, bytes: 2, cycles: 3}),
            0xEC => self.cpx(mem, &OpInfo{mode: Absolute, bytes: 3, cycles: 4}),

            // CPY
            0xC0 => self.cpx(mem, &OpInfo{mode: Immediate, bytes: 2, cycles: 2}),
            0xC4 => self.cpx(mem, &OpInfo{mode: ZeroPage, bytes: 2, cycles: 3}),
            0xCC => self.cpx(mem, &OpInfo{mode: Absolute, bytes: 3, cycles: 4}),

            // DEC (decrement memory)
            0xC6 => self.dec(mem, &OpInfo{mode: ZeroPage, bytes: 2, cycles: 5}),
            0xD6 => self.dec(mem, &OpInfo{mode: ZeroPageX, bytes: 2, cycles: 6}),
            0xCE => self.dec(mem, &OpInfo{mode: Absolute, bytes: 3, cycles: 6}),
            0xDE => self.dec(mem, &OpInfo{mode: AbsoluteX, bytes: 3, cycles: 7}),

            // DEX (decrement x register)
            0xCA => self.dex(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 2}),

            // DEY (decrement x register)
            0x88 => self.dey(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 2}),

            // EOR (exclusive or)
            0x49 => self.eor(mem, &OpInfo{mode: Immediate, bytes: 2, cycles: 2}),
            0x45 => self.eor(mem, &OpInfo{mode: ZeroPage, bytes: 2, cycles: 3}),
            0x55 => self.eor(mem, &OpInfo{mode: ZeroPageX, bytes: 2, cycles: 4}),
            0x4D => self.eor(mem, &OpInfo{mode: Absolute, bytes: 3, cycles: 4}),
            0x5D => self.eor(mem, &OpInfo{mode: AbsoluteX, bytes: 3, cycles: 4}),
            0x59 => self.eor(mem, &OpInfo{mode: AbsoluteY, bytes: 3, cycles: 4}),
            0x41 => self.eor(mem, &OpInfo{mode: IndexedIndirect, bytes: 2, cycles: 6}),
            0x51 => self.eor(mem, &OpInfo{mode: IndirectIndexed, bytes: 2, cycles: 5}),

            // INC (increment memory)
            0xE6 => self.inc(mem, &OpInfo{mode: ZeroPage, bytes: 2, cycles: 5}),
            0xF6 => self.inc(mem, &OpInfo{mode: ZeroPage, bytes: 2, cycles: 6}),
            0xEE => self.inc(mem, &OpInfo{mode: ZeroPage, bytes: 3, cycles: 6}),
            0xFE => self.inc(mem, &OpInfo{mode: ZeroPage, bytes: 3, cycles: 7}),

            // INX (increment x register)
            0xE8 => self.dey(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 2}),

            // INY (increment y register)
            0xC8 => self.dey(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 2}),

            // JMP (jump)
            0x4C => self.jmp(mem, &OpInfo{mode: Absolute, bytes: 3, cycles: 3}),
            0x6C => self.jmp(mem, &OpInfo{mode: Indirect, bytes: 3, cycles: 5}),

            // JSR (jump to subrouting)
            0x20 => self.jsr(mem, &OpInfo{mode: Absolute, bytes: 3, cycles: 6}),

            // LDA (load accumulator)
            0xA9 => self.lda(mem, &OpInfo{mode: Immediate, bytes: 2, cycles: 2}),
            0xA5 => self.lda(mem, &OpInfo{mode: ZeroPage, bytes: 2, cycles: 3}),
            0xB5 => self.lda(mem, &OpInfo{mode: ZeroPageX, bytes: 2, cycles: 4}),
            0xAD => self.lda(mem, &OpInfo{mode: Absolute, bytes: 3, cycles: 4}),
            0xBD => self.lda(mem, &OpInfo{mode: AbsoluteX, bytes: 3, cycles: 4}),
            0xB9 => self.lda(mem, &OpInfo{mode: AbsoluteY, bytes: 3, cycles: 4}),
            0xA1 => self.lda(mem, &OpInfo{mode: IndirectIndexed, bytes: 2, cycles: 6}),
            0xB1 => self.lda(mem, &OpInfo{mode: IndexedIndirect, bytes: 2, cycles: 5}),

            // LDX (load x register)
            0xA2 => self.ldx(mem, &OpInfo{mode: Immediate, bytes: 2, cycles: 2}),
            0xA6 => self.ldx(mem, &OpInfo{mode: ZeroPage, bytes: 2, cycles: 3}),
            0xB6 => self.ldx(mem, &OpInfo{mode: ZeroPageY, bytes: 2, cycles: 4}),
            0xAE => self.ldx(mem, &OpInfo{mode: Absolute, bytes: 3, cycles: 4}),
            0xBE => self.ldx(mem, &OpInfo{mode: AbsoluteY, bytes: 3, cycles: 4}),

            // LDY (load y register)
            0xA0 => self.ldy(mem, &OpInfo{mode: Immediate, bytes: 2, cycles: 2}),
            0xA4 => self.ldy(mem, &OpInfo{mode: ZeroPage, bytes: 2, cycles: 3}),
            0xB4 => self.ldy(mem, &OpInfo{mode: ZeroPageX, bytes: 2, cycles: 4}),
            0xAC => self.ldy(mem, &OpInfo{mode: Absolute, bytes: 3, cycles: 4}),
            0xBC => self.ldy(mem, &OpInfo{mode: AbsoluteX , bytes: 3, cycles: 4}),

            // LSR (logical shift right)
            0x4A => self.lsr(mem, &OpInfo{mode: Accumulator, bytes: 1, cycles: 2}),
            0x46 => self.lsr(mem, &OpInfo{mode: ZeroPage, bytes: 2, cycles: 5}),
            0x56 => self.lsr(mem, &OpInfo{mode: ZeroPageX, bytes: 2, cycles: 6}),
            0x4E => self.lsr(mem, &OpInfo{mode: Absolute, bytes: 3, cycles: 6}),
            0x5E => self.lsr(mem, &OpInfo{mode: AbsoluteX, bytes: 3, cycles: 7}),

            // NOP (no operation)
            0xEA => self.nop(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 2}),

            // OAR (logical inclusive or)
            0x09 => self.ora(mem, &OpInfo{mode: Immediate, bytes: 2, cycles: 2}),
            0x05 => self.ora(mem, &OpInfo{mode: ZeroPage, bytes: 2, cycles: 3}),
            0x15 => self.ora(mem, &OpInfo{mode: ZeroPageX, bytes: 2, cycles: 4}),
            0x0D => self.ora(mem, &OpInfo{mode: Absolute, bytes: 3, cycles: 4}),
            0x1D => self.ora(mem, &OpInfo{mode: AbsoluteX, bytes: 3, cycles: 4}),
            0x19 => self.ora(mem, &OpInfo{mode: AbsoluteY, bytes: 3, cycles: 4}),
            0x01 => self.ora(mem, &OpInfo{mode: IndexedIndirect, bytes: 2, cycles: 6}),
            0x11 => self.ora(mem, &OpInfo{mode: IndirectIndexed, bytes: 2, cycles: 5}),

            // PHA (push accumulator)
            0x48 => self.pha(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 3}),

            // PHP (push status register)
            0x08 => self.php(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 3}),

            // PLA (pull accumulator)
            0x68 => self.pla(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 3}),

            // PLP (pull status register)
            0x28 => self.pla(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 3}),

            // ROL (rotate left)
            0x2A => self.rol(mem, &OpInfo{mode: Accumulator, bytes: 1, cycles: 2}),
            0x26 => self.rol(mem, &OpInfo{mode: ZeroPage, bytes: 2, cycles: 5}),
            0x36 => self.rol(mem, &OpInfo{mode: ZeroPageX, bytes: 2, cycles: 6}),
            0x2E => self.rol(mem, &OpInfo{mode: Absolute, bytes: 3, cycles: 6}),
            0x3E => self.rol(mem, &OpInfo{mode: AbsoluteX, bytes: 3, cycles: 7}),

            // ROR (rotate right)
            0x6A => self.ror(mem, &OpInfo{mode: Accumulator, bytes: 1, cycles: 2}),
            0x66 => self.ror(mem, &OpInfo{mode: ZeroPage, bytes: 2, cycles: 5}),
            0x76 => self.ror(mem, &OpInfo{mode: ZeroPageX, bytes: 2, cycles: 6}),
            0x6E => self.ror(mem, &OpInfo{mode: Absolute, bytes: 3, cycles: 6}),
            0x7E => self.ror(mem, &OpInfo{mode: AbsoluteX, bytes: 3, cycles: 7}),

            // RTI (return from interrupt)
            0x40 => self.rti(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 6}),

            // SBC (subtract with carry)
            0xE9 => self.sbc(mem, &OpInfo{mode: Immediate, bytes: 2, cycles: 2}),
            0xE5 => self.sbc(mem, &OpInfo{mode: ZeroPage,  bytes: 2, cycles: 3}),
            0xF5 => self.sbc(mem, &OpInfo{mode: ZeroPageX, bytes: 2, cycles: 4}),
            0xED => self.sbc(mem, &OpInfo{mode: Absolute,  bytes: 3, cycles: 4}),
            0xFD => self.sbc(mem, &OpInfo{mode: AbsoluteX, bytes: 3, cycles: 4}),
            0xF9 => self.sbc(mem, &OpInfo{mode: AbsoluteY, bytes: 3, cycles: 4}),
            0xE1 => self.sbc(mem, &OpInfo{mode: IndexedIndirect, bytes: 2, cycles: 6}),
            0xF1 => self.sbc(mem, &OpInfo{mode: IndirectIndexed, bytes: 2, cycles: 5}),

            // SEC (set carry flag)
            0x38 => self.sec(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 2}),

            // SED (set decimal flag)
            0xF8 => self.sed(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 2}),

            // SEI (set interrupt disable flag)
            0x78 => self.sei(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 2}),

            // STA (store accumulator)
            0x85 => self.sta(mem, &OpInfo{mode: ZeroPage,  bytes: 2, cycles: 3}),
            0x95 => self.sta(mem, &OpInfo{mode: ZeroPageX, bytes: 2, cycles: 4}),
            0x8D => self.sta(mem, &OpInfo{mode: Absolute,  bytes: 3, cycles: 4}),
            0x9D => self.sta(mem, &OpInfo{mode: AbsoluteX, bytes: 3, cycles: 5}),
            0x99 => self.sta(mem, &OpInfo{mode: AbsoluteY, bytes: 3, cycles: 5}),
            0x81 => self.sta(mem, &OpInfo{mode: IndexedIndirect, bytes: 2, cycles: 6}),
            0x91 => self.sta(mem, &OpInfo{mode: IndirectIndexed, bytes: 2, cycles: 6}),

            // STX (store x register)
            0x86 => self.stx(mem, &OpInfo{mode: ZeroPage,  bytes: 2, cycles: 3}),
            0x96 => self.stx(mem, &OpInfo{mode: ZeroPageX, bytes: 2, cycles: 4}),
            0x8E => self.stx(mem, &OpInfo{mode: Absolute,  bytes: 3, cycles: 4}),

            // STY (store y register)
            0x84 => self.sty(mem, &OpInfo{mode: ZeroPage,  bytes: 2, cycles: 3}),
            0x94 => self.sty(mem, &OpInfo{mode: ZeroPageX, bytes: 2, cycles: 4}),
            0x8C => self.sty(mem, &OpInfo{mode: Absolute,  bytes: 3, cycles: 4}),

            // TAX (transfer accumulator to x register)
            0xAA => self.tax(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 2}),

            // TAY (transfer accumulator to y register)
            0xA8 => self.tay(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 2}),

            // TSX (transfer stack pointer to x register)
            0xBA => self.tsx(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 2}),

            // TXA (transfer x register to accumulator)
            0x8A => self.txa(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 2}),

            // TXS (transfer x register to stack pointer)
            0x9A => self.txa(mem, &OpInfo{mode: Implicit, bytes: 1, cycles: 2}),


            // TODO: more remaining optcodes
            _ => panic!("not implemented"),
        };
    }

    /// CPU instruction: ADC (add with carry)
    ///
    /// This instruction adds the contents of a memory location to the
    /// accumulator together with the carry bit. If overflow occurs the carry
    /// bit is set, this enables multiple byte addition to be performed.
    ///
    /// Affected Registers:
    /// - zero flag: set if accumulator is zero
    /// - negative flag: set if bit 7 (highest bit) is set
    /// - overflow flag: set if sign bit is incorrect
    fn adc(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let addr = self.get_address(mem, opi.mode).unwrap();
        let m = mem.read(addr);

        let a = self.accumulator;
        let c = self.carry_flag();

        // this might overflow
        self.accumulator = a + m + c;

        // set carry flag
        if a as usize + m as usize + c as usize > 0xFF {
            self.status_register.carry_flag = true;
        } else {
            self.status_register.carry_flag = false;
        }

        self.status_register.overflow_flag = utils::calculate_overflow_bit(a, m, self.accumulator);

        let a = self.accumulator;
        self.update_zero_flag(a);
        self.update_negative_flag(a);
    }

    /// CPU instruction: AND (logical AND)
    ///
    /// A logical AND is performed, bit by bit, on the accumulator contents
    /// using the contents of a byte of memory.
    fn and(&mut self, mem: &mut Memory, mode: AddressingMode) {
        let addr = self.get_address(mem, mode).unwrap();
        let m = mem.read(addr);
        let a = self.accumulator;

        self.accumulator = a & m;

        let a = self.accumulator;
        self.update_zero_flag(a);
        self.update_negative_flag(a);
    }

    /// CPU instruction: ASL (arithmetic shift left)
    ///
    /// This operation shifts all the bits of the accumulator or memory
    /// contents one bit left. Bit 0 is set to 0 and bit 7 is placed in the
    /// carry flag. The effect of this operation is to multiply the memory
    /// contents by 2 (ignoring 2's complement considerations), setting the
    /// carry if the result will not fit in 8 bits.
    fn asl(&mut self, mem: &mut Memory, mode: AddressingMode) {
        match mode {
            AddressingMode::Accumulator => {
                let (v, c) = self.accumulator.overflowing_shl(1);
                self.accumulator = v;
                self.status_register.carry_flag = c;
            }
            _ => {
                let addr = self.get_address(mem, mode).unwrap();
                let m = mem.read(addr);
                let (v, c) = m.overflowing_shl(1);
                self.status_register.carry_flag = c;
            }
        }

        let a = self.accumulator;
        self.update_zero_flag(a);
        self.update_negative_flag(a);
    }

    /// CPU instruction: BCC (branch if carry clear)
    ///
    /// If the carry flag is clear then add the relative displacement to the
    /// program counter to cause a branch to a new location.
    ///
    /// TODO: test
    fn bcc(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let condition = self.status_register.carry_flag == false;
        self.conditional_branch(mem, opi, condition);
    }

    /// CPU instruction: BCS (branch if carry set)
    ///
    /// If the carry flag is set then add the relative displacement to the
    /// program counter to cause a branch to a new location.
    fn bcs(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let condition = self.status_register.carry_flag;
        self.conditional_branch(mem, opi, condition);
    }

    /// CPU instruction: BEQ (branch if equal)
    ///
    /// If the zero flag is set then add the relative displacement to the
    /// program counter to cause a branch to a new location.
    fn beq(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let condition = self.status_register.zero_flag;
        self.conditional_branch(mem, opi, condition);
    }

    /// CPU instruction: BIT (bit test)
    ///
    /// This instructions is used to test if one or more bits are set in a
    /// target memory location. The mask pattern in A is ANDed with the value
    /// in memory to set or clear the zero flag, but the result is not kept.
    /// Bits 7 and 6 of the value from memory are copied into the N and V flags.
    fn bit(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let addr = self.get_address(mem, opi.mode).unwrap();
        let m = mem.read(addr);
        let res = m & self.accumulator;

        if res == 0x00 {
            self.status_register.zero_flag = true;
        }

        if m >> 7 & 0x01 == 0x01 {
            self.status_register.negative_flag = true;
        }

        if m >> 6 & 0x01 == 0x01 {
            self.status_register.overflow_flag = true;
        }

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    fn conditional_branch(&mut self, mem: &mut Memory, opi: &OpInfo, condition: bool) {
        if condition {
            let addr = self.get_address(mem, opi.mode).unwrap();
            let page_crossed = memory::page_crossed(self.program_counter, addr);
            self.cycles += 1;

            // add 2 cycles if the target address is on a new page
            if page_crossed {
                self.cycles += 2;
            }

            self.program_counter = addr;
        } else {
            self.program_counter += opi.bytes as u16;
        }

        self.cycles += opi.cycles;
    }

    /// CPU instruction: BMI (branch if minus)
    ///
    /// If the negative flag is set then add the relative displacement to the
    /// program counter to cause a branch to a new location.
    fn bmi(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let condition = self.status_register.negative_flag;
        self.conditional_branch(mem, opi, condition);
    }

    /// CPU instruction: BNE (branch if not equal)
    ///
    /// If the zero flag is clear then add the relative displacement to the
    /// program counter to cause a branch to a new location.
    fn bne(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let condition = self.status_register.zero_flag == false;
        self.conditional_branch(mem, opi, condition);
    }

    /// CPU instruction: BPL (branch if positive)
    ///
    /// If the negative flag is clear then add the relative displacement to
    /// the program counter to cause a branch to a new location.
    fn bpl(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let condition = self.status_register.negative_flag == false;
        self.conditional_branch(mem, opi, condition);
    }

    /// CPU instruction: BRK (force interrupt)
    ///
    /// The BRK instruction forces the generation of an interrupt request. The
    /// program counter and processor status are pushed on the stack then the
    /// IRQ interrupt vector at $FFFE/F is loaded into the PC and the break
    /// flag in the status set to one.
    fn brk(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let sr = self.status_register.to_u8();
        self.push(mem, sr);
        let pc = self.program_counter;
        self.push16(mem, pc);

        // TODO: load interrupt vector
        self.status_register.break_command = true;
    }

    /// CPU instruction: BVC (branch if overflow clear)
    ///
    /// If the overflow flag is clear then add the relative displacement to the
    /// program counter to cause a branch to a new location.
    fn bvc(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let condition = self.status_register.overflow_flag == false;
        self.conditional_branch(mem, opi, condition);
    }

    /// CPU instruction: BVS (branch if overflow set)
    ///
    /// If the overflow flag is set then add the relative displacement to the
    /// program counter to cause a branch to a new location.
    fn bvs(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let condition = self.status_register.overflow_flag;
        self.conditional_branch(mem, opi, condition);
    }

    /// CPU instruction: CLC (clear carry flag)
    ///
    /// Set the carry flag to zero.
    fn clc(&mut self, mem: &mut Memory, opi: &OpInfo) {
        self.status_register.carry_flag = false;
        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: CLD (clear decimal mode)
    ///
    /// Sets the decimal mode flag to zero.
    fn cld(&mut self, mem: &mut Memory, opi: &OpInfo) {
        self.status_register.decimal_mode = false;
        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: CLI (clear interrupt disable)
    ///
    /// Clears the interrupt disable flag allowing normal interrupt requests to
    /// be serviced.
    fn cli(&mut self, mem: &mut Memory, opi: &OpInfo) {
        self.status_register.interrupt_disable = false;
        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: CLV (clear overflow flag)
    ///
    /// Clears the overflow flag.
    fn clv(&mut self, mem: &mut Memory, opi: &OpInfo) {
        self.status_register.overflow_flag = false;
        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: CMP (compare)
    ///
    /// This instruction compares the contents of the accumulator with another
    /// memory held value and sets the zero and carry flags as appropriate.
    ///
    /// TODO: only one opcode tested
    fn cmp(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let addr = self.get_address(mem, opi.mode).unwrap();
        let m = mem.read(addr);

        let r = self.accumulator.wrapping_sub(m);
        self.update_negative_flag(r);

        if self.accumulator > m {
            self.status_register.carry_flag = true;
        } else if self.accumulator == m {
            self.status_register.carry_flag = true;
            self.status_register.zero_flag = true;
        }

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction CPX (compare x register)
    ///
    /// This instruction compares the contents of the X register with another
    /// memory held value and sets the zero and carry flags as appropriate.
    ///
    /// TODO: tests
    /// TODO: refactor: pull duplicate code out
    fn cpx(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let addr = self.get_address(mem, opi.mode).unwrap();
        let m = mem.read(addr);

        let r = self.accumulator.wrapping_sub(m);
        self.update_negative_flag(r);

        if self.index_x > m {
            self.status_register.carry_flag = true;
        } else if self.index_x == m {
            self.status_register.carry_flag = true;
            self.status_register.zero_flag = true;
        }

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: CPY (compare y register)
    ///
    /// This instruction compares the contents of the Y register with another
    /// memory held value and sets the zero and carry flags as appropriate.
    ///
    /// TODO: tests
    /// TODO: refactor: pull duplicate code out
    fn cpy(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let addr = self.get_address(mem, opi.mode).unwrap();
        let m = mem.read(addr);

        let r = self.accumulator.wrapping_sub(m);
        self.update_negative_flag(r);

        if self.index_y > m {
            self.status_register.carry_flag = true;
        } else if self.index_y == m {
            self.status_register.carry_flag = true;
            self.status_register.zero_flag = true;
        }

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: DEC (decrememt memory)
    ///
    /// Subtracts one from the value held at a specified memory location
    /// setting the zero and negative flags as appropriate.
    fn dec(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let addr = self.get_address(mem, opi.mode).unwrap();
        let m = mem.read(addr);
        let r = m.wrapping_sub(1);

        mem.write(addr, r);

        self.update_negative_flag(r);
        self.update_zero_flag(r);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: DEX (decrement x register)
    ///
    /// Subtracts one from the X register setting the zero and negative flags
    /// as appropriate.
    fn dex(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let r = self.index_x.wrapping_sub(1);

        self.index_x = r;
        self.update_negative_flag(r);
        self.update_zero_flag(r);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: DEY (decrement y register)
    ///
    /// Subtracts one from the Y register setting the zero and negative flags
    /// as appropriate.
    fn dey(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let r = self.index_y.wrapping_sub(1);

        self.index_y = r;
        self.update_negative_flag(r);
        self.update_zero_flag(r);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: EOR (exclusive or)
    ///
    /// An exclusive OR is performed, bit by bit, on the accumulator contents
    /// using the contents of a byte of memory.
    fn eor(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let addr = self.get_address(mem, opi.mode).unwrap();
        let m = mem.read(addr);

        let r = self.accumulator ^ m;
        self.accumulator = r;

        self.update_zero_flag(r);
        self.update_negative_flag(r);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: INC (increment memory)
    ///
    /// Adds one to the value held at a specified memory location setting the
    /// zero and negative flags as appropriate.
    fn inc(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let addr = self.get_address(mem, opi.mode).unwrap();
        let m = mem.read(addr);
        let r = m.wrapping_add(1);

        mem.write(addr, r);

        self.update_negative_flag(r);
        self.update_zero_flag(r);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: INX (increment x register)
    ///
    /// Adds one to the X register setting the zero and negative flags as
    /// appropriate.
    fn inx(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let r = self.index_x.wrapping_add(1);

        self.index_x = r;
        self.update_negative_flag(r);
        self.update_zero_flag(r);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: INY (increment x register)
    ///
    /// Adds one to the Y register setting the zero and negative flags as
    /// appropriate.
    fn iny(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let r = self.index_y.wrapping_add(1);

        self.index_y = r;
        self.update_negative_flag(r);
        self.update_zero_flag(r);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: JMP (jump)
    ///
    /// Sets the program counter to the address specified by the operand.
    fn jmp(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let addr = self.get_address(mem, opi.mode).unwrap();

        self.program_counter = self.read16(mem, addr);

        self.cycles += opi.cycles;
    }

    /// CPU instruction: JSR (jump to subroutine)
    ///
    /// The JSR instruction pushes the address (minus one) of the return point
    /// on to the stack and then sets the program counter to the target memory address.
    fn jsr(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let return_address = self.program_counter - 1;
        self.push16(mem, return_address);

        let addr = self.get_address(mem, opi.mode).unwrap();
        self.program_counter = self.read16(mem, addr);

        self.cycles += opi.cycles;
    }

    /// CPU instruction: LDA (load accumulator)
    ///
    /// Loads a byte of memory into the accumulator setting the zero and
    /// negative flags as appropriate.
    fn lda(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let addr = self.get_address(mem, opi.mode).unwrap();
        let m = mem.read(addr);

        self.accumulator = m;
        self.update_zero_flag(m);
        self.update_negative_flag(m);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: LDX (load x register)
    ///
    /// Loads a byte of memory into the X register setting the zero and
    /// negative flags as appropriate.
    fn ldx(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let addr = self.get_address(mem, opi.mode).unwrap();
        let m = mem.read(addr);

        self.index_x = m;
        self.update_zero_flag(m);
        self.update_negative_flag(m);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: LDY (load y register)
    ///
    /// Loads a byte of memory into the Y register setting the zero and
    /// negative flags as appropriate.
    fn ldy(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let addr = self.get_address(mem, opi.mode).unwrap();
        let m = mem.read(addr);

        self.index_y = m;
        self.update_zero_flag(m);
        self.update_negative_flag(m);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: LSR (logical shift left)
    ///
    /// Each of the bits in A or M is shift one place to the right. The bit
    /// that was in bit 0 is shifted into the carry flag. Bit 7 is set to zero.
    fn lsr(&mut self, mem: &mut Memory, opi: &OpInfo) {
        match opi.mode {
            AddressingMode::Accumulator => {
                self.status_register.carry_flag = (self.accumulator & 0x01) == 0x01;
                let r = self.accumulator >> 1;
                self.update_negative_flag(r);
                self.accumulator = r;
            }
            _ => {
                let addr = self.get_address(mem, opi.mode).unwrap();
                let m = mem.read(addr);

                let r = m >> 1;
                self.update_negative_flag(r);
                mem.write(addr, r);
            }
        }

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: NOP (no operation)
    ///
    /// The NOP instruction causes no changes to the processor other than the
    /// normal incrementing of the program counter to the next instruction.
    fn nop(&mut self, mem: &mut Memory, opi: &OpInfo) {
        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: ORA (logical inclusive or)
    ///
    /// An inclusive OR is performed, bit by bit, on the accumulator contents
    /// using the contents of a byte of memory.
    fn ora(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let addr = self.get_address(mem, opi.mode).unwrap();
        let m = mem.read(addr);

        let r = self.accumulator | m;
        self.accumulator = r;

        self.update_zero_flag(r);
        self.update_negative_flag(r);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: PHA (push accumulator)
    ///
    /// Pushes a copy of the accumulator on to the stack.
    fn pha(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let a = self.accumulator;
        self.push(mem, a);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: PHP (push processor status)
    ///
    /// Pushes a copy of the status flags on to the stack.
    fn php(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let p = self.status_register.to_u8();
        self.push(mem, p);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: PLA (pull accumulator)
    ///
    /// Pulls an 8 bit value from the stack and into the accumulator. The zero
    /// and negative flags are set as appropriate.
    fn pla(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let a = self.pop(mem);
        self.accumulator = a;
        self.update_zero_flag(a);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: PLP (pull status register)
    ///
    /// Pulls an 8 bit value from the stack and into the processor flags. The
    /// flags will take on new states as determined by the value pulled.
    fn plp(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let p = StatusRegister::from_u8(self.pop(mem));
        self.status_register = p;

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: ROL (rotate left)
    ///
    /// Move each of the bits in either A or M one place to the left. Bit 0 is
    /// filled with the current value of the carry flag whilst the old bit 7
    /// becomes the new carry flag value.
    fn rol(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let (r, c) = match opi.mode {
            AddressingMode::Accumulator => {
                let (r, c) = self.accumulator.overflowing_shl(1);

                self.accumulator = r;
                (r, c)
            }
            _ => {
                let addr = self.get_address(mem, opi.mode).unwrap();
                let (r, c) = mem.read(addr).overflowing_shl(1);

                mem.write(addr, r);
                (r, c)
            }
        };

        self.status_register.carry_flag = c;
        self.update_negative_flag(r);
        self.update_zero_flag(r);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: ROR (rotate right)
    ///
    /// Move each of the bits in either A or M one place to the right. Bit 7 is
    /// filled with the current value of the carry flag whilst the old bit 0
    /// becomes the new carry flag value.
    fn ror(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let (r, c) = match opi.mode {
            AddressingMode::Accumulator => {
                let (r, c) = self.accumulator.overflowing_shr(1);

                self.accumulator = r;
                (r, c)
            }
            _ => {
                let addr = self.get_address(mem, opi.mode).unwrap();
                let (r, c) = mem.read(addr).overflowing_shr(1);

                mem.write(addr, r);
                (r, c)
            }
        };

        self.status_register.carry_flag = c;
        self.update_negative_flag(r);
        self.update_zero_flag(r);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: RTI (return from interrupt)
    ///
    /// The RTI instruction is used at the end of an interrupt processing
    /// routine. It pulls the processor flags from the stack followed by the
    /// program counter.
    fn rti(&mut self, mem: &mut Memory, opi: &OpInfo) {
        self.status_register = StatusRegister::from_u8(self.pop(mem));
        self.program_counter = self.pop16(mem);

        self.cycles += opi.cycles;
    }

    /// CPU instruction: RTS (return from subroutine)
    ///
    /// The RTS instruction is used at the end of a subroutine to return to the
    /// calling routine. It pulls the program counter (minus one) from the stack.
    fn rts(&mut self, mem: &mut Memory, opi: &OpInfo) {
        self.program_counter = self.pop16(mem) + 1;

        self.cycles += opi.cycles;
    }

    /// CPU instruction: SBC (subtract with carry)
    ///
    /// This instruction subtracts the contents of a memory location to the
    /// accumulator together with the not of the carry bit. If overflow occurs
    /// the carry bit is clear, this enables multiple byte subtraction to be
    /// performed.
    fn sbc(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let addr = self.get_address(mem, opi.mode).unwrap();
        let m = mem.read(addr);
        let a = self.accumulator;

        let r = self.accumulator - m - (1 - self.status_register.carry_flag as u8);
        self.accumulator = r;

        self.status_register.overflow_flag = utils::calculate_overflow_bit(a, m, r);
        self.update_zero_flag(r);
        self.update_negative_flag(r);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: SEC (set carry flag)
    ///
    /// Set the carry flag to one.
    fn sec(&mut self, mem: &mut Memory, opi: &OpInfo) {
        self.status_register.carry_flag = true;

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: SED (set decimal flag)
    ///
    /// Set the decimal mode flag to one.
    fn sed(&mut self, mem: &mut Memory, opi: &OpInfo) {
        self.status_register.decimal_mode = true;

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: SEI (set interrupt disable)
    ///
    /// Set the interrupt disable flag to one.
    fn sei(&mut self, mem: &mut Memory, opi: &OpInfo) {
        self.status_register.interrupt_disable = true;

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: STA (store accumulator)
    ///
    /// Stores the contents of the accumulator into memory.
    fn sta(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let addr = self.get_address(mem, opi.mode).unwrap();

        mem.write(addr, self.accumulator);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: STX (store x register)
    ///
    /// Stores the contents of the X register into memory.
    fn stx(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let addr = self.get_address(mem, opi.mode).unwrap();

        mem.write(addr, self.index_x);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: STY (store y register)
    ///
    /// Stores the contents of the Y register into memory.
    fn sty(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let addr = self.get_address(mem, opi.mode).unwrap();

        mem.write(addr, self.index_y);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: TAX (transfer accumulator to x register)
    ///
    /// Copies the current contents of the accumulator into the X register and
    /// sets the zero and negative flags as appropriate.
    fn tax(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let r = self.accumulator;

        self.index_x = r;
        self.update_zero_flag(r);
        self.update_negative_flag(r);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: TAY (transfer accumulator to y register)
    ///
    /// Copies the current contents of the accumulator into the Y register and
    /// sets the zero and negative flags as appropriate.
    fn tay(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let r = self.accumulator;

        self.index_y = r;
        self.update_zero_flag(r);
        self.update_negative_flag(r);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: TSX (transfer stack pointer to x register)
    ///
    /// Copies the current contents of the stack register into the X register
    /// and sets the zero and negative flags as appropriate.
    fn tsx(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let r = self.stack_pointer;

        self.index_x = r;
        self.update_zero_flag(r);
        self.update_negative_flag(r);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: TXA (transfer x register to accumulator)
    ///
    /// Copies the current contents of the X register into the accumulator and
    /// sets the zero and negative flags as appropriate.
    fn txa(&mut self, mem: &mut Memory, opi: &OpInfo) {
        let r = self.index_x;

        self.accumulator = r;
        self.update_zero_flag(r);
        self.update_negative_flag(r);

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    /// CPU instruction: TXS (transfer x register to stack pointer)
    ///
    /// Copies the current contents of the X register into the stack register.
    fn txs(&mut self, mem: &mut Memory, opi: &OpInfo) {
        self.stack_pointer = self.index_x;

        self.cycles += opi.cycles;
        self.program_counter += opi.bytes as u16;
    }

    fn tya(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_powerup_state() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);

        assert_eq!(cpu.accumulator, 0);
        assert_eq!(cpu.index_x, 0);
        assert_eq!(cpu.index_y, 0);
        assert_eq!(cpu.stack_pointer, 0xFD);

        assert_eq!(0x34, 52);
        assert_eq!(cpu.status_register.unused_bit, true);
        assert_eq!(cpu.status_register.break_command, true);
        assert_eq!(cpu.status_register.interrupt_disable, true);

        assert_eq!(cpu.status_register.carry_flag, false);
        assert_eq!(cpu.status_register.zero_flag, false);
        assert_eq!(cpu.status_register.decimal_mode, false);
        assert_eq!(cpu.status_register.overflow_flag, false);
        assert_eq!(cpu.status_register.negative_flag, false);

        // memory
        assert_eq!(mem.read(0x4017), 0);
    }

    #[test]
    fn cpu_reset_state() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();
        cpu.powerup(&mut mem);

        cpu.status_register.interrupt_disable = false;
        let sp_before = cpu.stack_pointer;
        println!("{}", sp_before);

        cpu.reset(&mut mem);

        assert_eq!(cpu.stack_pointer, sp_before - 3);
        assert_eq!(cpu.status_register.interrupt_disable, true);
        assert_eq!(mem.read(0x4015), 0);
        // TODO: test remaining memory addresses
    }

    #[test]
    fn cpu_set_negative_true() {
        let mut cpu = CPU::new();

        cpu.accumulator = 0xFF;
        assert_eq!(cpu.status_register.negative_flag, false);
        let a = cpu.accumulator;
        cpu.update_negative_flag(a);
        assert_eq!(cpu.status_register.negative_flag, true);
    }

    #[test]
    fn cpu_set_negative_false() {
        let mut cpu = CPU::new();

        cpu.accumulator = 0x00;
        assert_eq!(cpu.status_register.negative_flag, false);
        let a = cpu.accumulator;
        cpu.update_negative_flag(a);
        assert_eq!(cpu.status_register.negative_flag, false);
    }

    #[test]
    fn test_addressing_absolute() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);
        cpu.program_counter = 0x0000;

        mem.write(0x0001, 0xFF);
        mem.write(0x0002, 0xAA);

        let result = cpu.get_address(&mut mem, AddressingMode::Absolute);
        let expected = Some(0xAAFF);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_addressing_absolute_x() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);
        cpu.program_counter = 0x0000;
        cpu.index_x = 0x02;

        mem.write(0x0001, 0xBB);
        mem.write(0x0002, 0xAA);

        let result = cpu.get_address(&mut mem, AddressingMode::AbsoluteX);
        let expected = Some(0xAABD);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_addressing_absolute_y() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);
        cpu.program_counter = 0x0000;
        cpu.index_y = 0x04;

        mem.write(0x0001, 0xBB);
        mem.write(0x0002, 0xAA);

        let result = cpu.get_address(&mut mem, AddressingMode::AbsoluteY);
        let expected = Some(0xAABF);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_addressing_indexed_indirect() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);
        cpu.program_counter = 0x0000;
        cpu.index_x = 0x04;

        mem.write(0x0001, 0xA0);
        mem.write(0x00A4, 0xF0);
        mem.write(0x00A5, 0xDA);

        let expected = Some(0xDAF0);
        let result = cpu.get_address(&mut mem, AddressingMode::IndexedIndirect);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_addressing_indirect_indexed() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);
        cpu.program_counter = 0x0000;
        cpu.index_y = 0x04;

        mem.write(0x0001, 0xA0);
        mem.write(0x00A0, 0xF0);
        mem.write(0x00A1, 0xDA);

        let expected = Some(0xDAF0 + 0x04);
        let result = cpu.get_address(&mut mem, AddressingMode::IndirectIndexed);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_read16() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);
        mem.write(0x0004, 0xAA);
        mem.write(0x0005, 0xCC);

        let result = cpu.read16(&mem, 0x0004);
        let expected = 0xAACC;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_bmi_opcode_30_do_branch_to_same_page() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);

        // set initial conditions
        cpu.status_register.negative_flag = true;
        cpu.program_counter = 0x05;
        mem.write(0x05, 0x30); // next operation is 0x30
        mem.write(0x06, 0x04); // relative displacement is 2
        let expected = 0x09; // expect program counter to have this value

        // execute
        let cycles_before = cpu.cycles;
        cpu.step(&mut mem);
        let cycles_after = cpu.cycles;

        // assert
        let result = cpu.program_counter;
        let cycles_spent = cycles_after - cycles_before;

        assert_eq!(result, expected);
        assert_eq!(cycles_spent, 3);
    }

    #[test]
    fn test_bmi_opcode_30_do_branch_to_new_page() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);

        // set initial conditions
        cpu.status_register.negative_flag = true;
        cpu.program_counter = 0xEE;
        mem.write(0xEE, 0x30); // next operation is 0x30
        mem.write(0xEF, 0xDD); // relative displacement is 2
        let expected = 0xEE + 0xDD; // expect program counter to have this value

        // execute
        let cycles_before = cpu.cycles;
        cpu.step(&mut mem);
        let cycles_after = cpu.cycles;

        // assert
        let result = cpu.program_counter;
        let cycles_spent = cycles_after - cycles_before;

        assert_eq!(result, expected);
        assert_eq!(cycles_spent, 5);
    }

    #[test]
    fn test_bmi_opcode_30_no_branch() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);

        // set initial conditions
        cpu.status_register.negative_flag = false;
        cpu.program_counter = 0x05;
        mem.write(0x05, 0x30); // next operation is 0x30
        mem.write(0x06, 0x02); // relative displacement is 2
        let expected = 0x07; // expect program counter to have this value

        // execute
        let cycles_before = cpu.cycles;
        cpu.step(&mut mem);
        let cycles_after = cpu.cycles;

        // assert
        let result = cpu.program_counter;
        let cycles_spent = cycles_after - cycles_before;

        assert_eq!(result, expected);
        assert_eq!(cycles_spent, 2);
    }

    #[test]
    fn test_bit_opcode_24() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);

        // set initial conditions
        cpu.program_counter = 0x05;
        mem.write(0x05, 0x24);
        mem.write(0x06, 0x00); // memory address is 0x00
        mem.write(0x00, 0b11000000);

        let cycles_before = cpu.cycles;
        let pc_before = cpu.program_counter;
        cpu.step(&mut mem);
        let cycles_after = cpu.cycles;
        let pc_after = cpu.program_counter;

        assert_eq!(cycles_after - cycles_before, 3);
        assert_eq!(pc_after - pc_before, 2);
        assert_eq!(cpu.status_register.overflow_flag, true);
        assert_eq!(cpu.status_register.negative_flag, true);
    }

    #[test]
    fn test_instruction_cmp_with_opcode_C9() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        // a < m
        cpu.powerup(&mut mem);
        mem.write(cpu.program_counter, 0xC9);
        mem.write(cpu.program_counter + 1, 0x04);
        cpu.accumulator = 0x02;

        cpu.step(&mut mem);

        assert_eq!(cpu.status_register.carry_flag, false);
        assert_eq!(cpu.status_register.zero_flag, false);
        assert_eq!(cpu.status_register.negative_flag, true);

        // a > m
        cpu.powerup(&mut mem);
        mem.write(cpu.program_counter, 0xC9);
        mem.write(cpu.program_counter + 1, 0x04);
        cpu.accumulator = 0x06;

        cpu.step(&mut mem);

        assert_eq!(cpu.status_register.carry_flag, true);
        assert_eq!(cpu.status_register.zero_flag, false);
        assert_eq!(cpu.status_register.negative_flag, false);

        // a == m
        cpu.powerup(&mut mem);
        mem.write(cpu.program_counter, 0xC9);
        mem.write(cpu.program_counter + 1, 0x04);
        cpu.accumulator = 0x04;

        cpu.step(&mut mem);

        assert_eq!(cpu.status_register.carry_flag, true);
        assert_eq!(cpu.status_register.zero_flag, true);
        assert_eq!(cpu.status_register.negative_flag, false);
    }

    #[test]
    fn test_push_and_pop() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);

        let sp_before = cpu.stack_pointer;
        cpu.push(&mut mem, 0x42);
        assert_eq!(cpu.stack_pointer, sp_before.wrapping_sub(1));

        assert_eq!(cpu.pop(&mut mem), 0x42);
        assert_eq!(cpu.stack_pointer, sp_before);
    }

    #[test]
    fn test_push16_and_pop16() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new();

        cpu.powerup(&mut mem);

        let sp_before = cpu.stack_pointer;
        cpu.push16(&mut mem, 0xAA42);
        assert_eq!(cpu.stack_pointer, sp_before.wrapping_sub(2));

        assert_eq!(cpu.pop16(&mut mem), 0xAA42);
        assert_eq!(cpu.stack_pointer, sp_before);
    }

    #[test]
    fn test_cpu_display() {
        let mut cpu = CPU::new();
        cpu.accumulator = 0x01;
        cpu.stack_pointer = 0x02;
        cpu.program_counter = 0x03;
        cpu.index_x = 0x04;
        cpu.index_y = 0x05;

        let res = format!("{}", cpu);
        let exp = "CPU: a:0x01, s:0x02, p:0x03, x:0x04, y:0x05";

        assert_eq!(res, exp);
    }

}
