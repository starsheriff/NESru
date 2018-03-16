enum StatusRegisterBits {
    CarryFlag = 0, // 0
    ZeroFlag = 1, // 2
    InterruptDisable = 2, // 4
    DecimalMode = 3, // 8
    BreakCommand = 4, // 16
    UnusedBit = 5, // 32
    OverflowFlag = 6, // 64
    NegativeFlag = 7, // 128
}


#[derive(Debug)]
struct StatusRegister {
    carry_flag: bool,
    zero_flag: bool,
    interrupt_disable: bool,
    decimal_mode: bool,
    break_command: bool,
    overflow_flag: bool,
    negative_flag: bool,
    unused_bit: bool,
}

impl StatusRegister {
    fn new() -> StatusRegister {
        StatusRegister {
            carry_flag: false,
            zero_flag: false,
            interrupt_disable: false,
            decimal_mode: false,
            break_command: false,
            overflow_flag: false,
            negative_flag: false,
            unused_bit: false,
        }
    }

    fn from_u8() -> StatusRegister {
        //TODO: implementation
        StatusRegister::new()
    }

    fn set_all(&mut self, b: u8) {
        self.carry_flag = (b >> StatusRegisterBits::CarryFlag as u8) & 0x01 == 1;
        self.zero_flag = (b >> StatusRegisterBits::ZeroFlag as u8) & 0x01 == 1;
        self.interrupt_disable = (b >> StatusRegisterBits::InterruptDisable as u8) & 0x01 == 1;
        self.decimal_mode = (b >> StatusRegisterBits::DecimalMode as u8) & 0x01 == 1;
        self.break_command = (b >> StatusRegisterBits::BreakCommand as u8) & 0x01 == 1;
        self.overflow_flag = (b >> StatusRegisterBits::OverflowFlag as u8) & 0x01 == 1;
        self.negative_flag = (b >> StatusRegisterBits::NegativeFlag as u8) & 0x01 == 1;
        self.unused_bit = (b >> StatusRegisterBits::UnusedBit as u8) & 0x01 == 1;
    }

    fn to_u8(&self) -> u8 {
        let mut sr = 0x00; // inital return value set to 0b00000000
        sr |= (self.carry_flag as u8) << StatusRegisterBits::CarryFlag as u8;
        sr |= (self.zero_flag as u8) << StatusRegisterBits::ZeroFlag as u8;
        sr |= (self.interrupt_disable as u8) << StatusRegisterBits::InterruptDisable as u8;
        sr |= (self.decimal_mode as u8) << StatusRegisterBits::DecimalMode as u8;
        sr |= (self.break_command as u8) << StatusRegisterBits::BreakCommand as u8;
        sr |= (self.overflow_flag as u8) << StatusRegisterBits::OverflowFlag as u8;
        sr |= (self.negative_flag as u8) << StatusRegisterBits::NegativeFlag as u8;
        sr |= (self.unused_bit as u8) << StatusRegisterBits::UnusedBit as u8;
        sr
    }
}


// TODO: size?
const MEM_SIZE: usize = 0x10000;

struct Memory {
    mem: [u8; MEM_SIZE],
}

impl Memory {
    pub fn new() -> Memory {
        Memory{mem: [0;MEM_SIZE]}
    }

    pub fn read(&self, addr: usize) -> u8 {
        self.mem[addr]
    }

    pub fn write(&mut self, addr:usize, val: u8) {
        self.mem[addr] = val
    }
}

struct CPU {
    accumulator: u8,
    stack_pointer: u8,
    program_counter: u8,
    index_x: u8,
    index_y: u8,
    status_register: StatusRegister,
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
        }
    }

    pub fn powerup(&mut self, mem: &mut Memory) {
        self.status_register.set_all(0x34);
        self.accumulator = 0;
        self.index_x = 0;
        self.index_y = 0;
        self.stack_pointer = 0xFD;

        // Remaining tasks: set memory
        // TODO:    $4017 = 0x00
        //          $4000-$400F = 0x00
        // LSFR = 0x00
        mem.write(0x4017, 0x00);
    }

    pub fn reset(&mut self, mem: &mut Memory) {
        self.stack_pointer -= 3;
        self.status_register.interrupt_disable = true;

        // Remaining tasks: set memory
        mem.write(0x4015, 0x00);
    }

    fn step(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn set_carry_flag() {
        let mut sr = StatusRegister::new();
        assert_eq!(sr.carry_flag, false);
        sr.set_all(0x01);
        assert_eq!(sr.carry_flag, true);
    }

    #[test]
    fn set_zero_flag() {
        let mut sr = StatusRegister::new();
        assert_eq!(sr.zero_flag, false);
        sr.set_all(0x02);
        assert_eq!(sr.zero_flag, true);
    }

    #[test]
    fn set_interrupt_disable() {
        let mut sr = StatusRegister::new();
        assert_eq!(sr.interrupt_disable, false);
        sr.set_all(0x04);
        assert_eq!(sr.interrupt_disable, true);
    }

    #[test]
    fn set_decimal_mode() {
        let mut sr = StatusRegister::new();
        assert_eq!(sr.decimal_mode, false);
        sr.set_all(0x08);
        assert_eq!(sr.decimal_mode, true);
    }

    #[test]
    fn set_break_command() {
        let mut sr = StatusRegister::new();
        assert_eq!(sr.break_command, false);
        sr.set_all(0x10);
        assert_eq!(sr.break_command, true);
    }

    #[test]
    fn set_overflow_flag() {
        let mut sr = StatusRegister::new();
        assert_eq!(sr.overflow_flag, false);
        sr.set_all(0x40);
        assert_eq!(sr.overflow_flag, true);
    }

    #[test]
    fn set_negative_flag() {
        let mut sr = StatusRegister::new();
        assert_eq!(sr.negative_flag, false);
        sr.set_all(0x80);
        assert_eq!(sr.negative_flag, true);
    }

    #[test]
    fn set_all_0000_0011() {
        let mut sr = StatusRegister::new();
        assert_eq!(sr.carry_flag, false);
        assert_eq!(sr.zero_flag, false);
        sr.set_all(0x03);
        assert_eq!(sr.carry_flag, true);
        assert_eq!(sr.zero_flag, true);
    }

    #[test]
    fn set_all_0xFF() {
        let mut sr = StatusRegister::new();
        sr.set_all(0xFF);
        assert_eq!(sr.to_u8(), 0xFF);
    }

    #[test]
    fn status_register_to_u8() {
        let mut sr = StatusRegister::new();
        assert_eq!(sr.to_u8(), 0x00);

        sr.carry_flag = true;
        assert_eq!(sr.to_u8(), 1);

        sr = StatusRegister::new();
        sr.zero_flag = true;
        assert_eq!(sr.to_u8(), 2);

        sr = StatusRegister::new();
        sr.interrupt_disable = true;
        assert_eq!(sr.to_u8(), 4);

        sr = StatusRegister::new();
        sr.decimal_mode = true;
        assert_eq!(sr.to_u8(), 8);

        sr = StatusRegister::new();
        sr.break_command = true;
        assert_eq!(sr.to_u8(), 16);

        sr = StatusRegister::new();
        sr.overflow_flag = true;
        assert_eq!(sr.to_u8(), 64);

        sr = StatusRegister::new();
        sr.negative_flag = true;
        assert_eq!(sr.to_u8(), 128);

        sr = StatusRegister::new();
        sr.carry_flag = true;
        sr.interrupt_disable = true;
        assert_eq!(sr.to_u8(), 4 + 1);
    }

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

        assert_eq!(cpu.stack_pointer, sp_before-3);
        assert_eq!(cpu.status_register.interrupt_disable, true);
        assert_eq!(mem.read(0x4015), 0);
        // TODO: test remaining memory addresses
    }
}
