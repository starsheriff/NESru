
enum StatusRegisterBits {
    CarryFlag = 0,
    ZeroFlag = 1,
    InterruptDisable = 2,
    DecimalMode = 3,
    BreakCommand = 4,
    // 5 is unused
    OverflowFlag = 6,
    NegativeFlag = 7,
}

struct StatusRegister {
    carry_flag: bool,
    zero_flag: bool,
    interrupt_disable: bool,
    decimal_mode: bool,
    break_command: bool,
    overflow_flag: bool,
    negative_flag: bool,
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
        }
    }

    fn from_u8() -> StatusRegister {
        //TODO: implementation
        StatusRegister::new()
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
        sr
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
        CPU{
            accumulator: 0,
            stack_pointer: 0,
            program_counter: 0,
            index_x: 0,
            index_y: 0,
            status_register: StatusRegister::new(),
        }
    }

    pub fn powerup(&mut self) {
    }

    fn run(&mut self) {
        loop {
            self.step();
        }
    }

    fn step(&mut self) {
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
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
        assert_eq!(sr.to_u8(), 4+1);

    }
}
