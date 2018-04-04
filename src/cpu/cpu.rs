use cpu::opinfo::{OpInfo, OP_INFO};
use cpu::status_register::StatusRegister;
use cpu::utils;

use memory::{self, Memory};

type MemoryAddress = u16;
type PageCrossed = bool;

// TODO: set correct address
static STACK_BASE_ADDRESS: u16 = 0x4000;

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
    fn update_zero_flag(&mut self) {
        if self.accumulator == 0x00 {
            self.status_register.zero_flag = true;
        }
    }

    fn update_negative_flag(&mut self) {
        // TODO: implement
        if self.accumulator >> 7 & 0x01 == 0x01 {
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
        }
    }

    fn read16(&self, mem: &Memory, addr: u16) -> u16 {
        let msb = mem.read(addr) as u16;
        let lsb = mem.read(addr + 1) as u16;

        (msb << 8) + lsb
    }

    fn push(&mut self, mem: &mut Memory, val: u8) {
        let addr = STACK_BASE_ADDRESS + self.stack_pointer as u16;
        mem.write(addr, val);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn push16(&mut self, mem: &mut Memory, val: u16) {
        let msb = ((val >> 8) as u8) & 0xFF;
        let lsb = (val as u8) & 0xFF;

        self.push(mem, msb);
        self.push(mem, lsb);
    }

    fn pop(&mut self, mem: &mut Memory) -> u8 {
        let addr = STACK_BASE_ADDRESS + self.stack_pointer as u16 + 1;
        let val = mem.read(addr);

        self.stack_pointer = self.stack_pointer.wrapping_add(1);

        val
    }

    fn pop16(&mut self, mem: &mut Memory) -> u16 {
        let lsb = self.pop(mem) as u16;
        let msb = self.pop(mem) as u16;

        (msb << 8) + lsb
    }

    fn step(&mut self, mem: &mut Memory) {
        // TODO: really required?
        //self.cycles += 1;

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

        self.update_zero_flag();
        self.update_negative_flag();
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

        self.update_zero_flag();
        self.update_negative_flag();
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

        self.update_zero_flag();
        self.update_negative_flag();
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

    fn bit(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
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

    fn bvc(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn bvs(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn clc(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn cld(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn cli(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn clv(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn cmp(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn cpx(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn cpy(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn dec(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn dex(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn dey(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn eor(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn inc(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn inx(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn iny(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn jmp(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn jsr(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn lda(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn ldx(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn ldy(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn lsr(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn nop(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn ora(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn pha(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn php(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn pla(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn plp(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn rol(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn ror(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn rti(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn rts(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn sbc(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn sec(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn sed(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn sei(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn sta(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn stx(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn sty(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn tax(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn tay(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn tsx(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn txa(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
    }

    fn txs(&mut self, mem: &mut Memory, opi: &OpInfo) {
        // TODO
        panic!("not implemented");
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
        cpu.update_negative_flag();
        assert_eq!(cpu.status_register.negative_flag, true);
    }

    #[test]
    fn cpu_set_negative_false() {
        let mut cpu = CPU::new();

        cpu.accumulator = 0x00;
        assert_eq!(cpu.status_register.negative_flag, false);
        cpu.update_negative_flag();
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

}
