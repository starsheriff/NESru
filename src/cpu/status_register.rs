pub enum StatusRegisterBits {
    CarryFlag = 0,        // 0
    ZeroFlag = 1,         // 2
    InterruptDisable = 2, // 4
    DecimalMode = 3,      // 8
    BreakCommand = 4,     // 16
    UnusedBit = 5,        // 32
    OverflowFlag = 6,     // 64
    NegativeFlag = 7,     // 128
}

#[derive(Debug)]
pub struct StatusRegister {
    pub carry_flag: bool,
    pub zero_flag: bool,
    pub interrupt_disable: bool,
    pub decimal_mode: bool,
    pub break_command: bool,
    pub overflow_flag: bool,
    pub negative_flag: bool,
    pub unused_bit: bool,
}

impl StatusRegister {
    pub fn new() -> StatusRegister {
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

    pub fn from_u8() -> StatusRegister {
        //TODO: implementation
        StatusRegister::new()
    }

    pub fn set_all(&mut self, b: u8) {
        self.carry_flag = (b >> StatusRegisterBits::CarryFlag as u8) & 0x01 == 1;
        self.zero_flag = (b >> StatusRegisterBits::ZeroFlag as u8) & 0x01 == 1;
        self.interrupt_disable = (b >> StatusRegisterBits::InterruptDisable as u8) & 0x01 == 1;
        self.decimal_mode = (b >> StatusRegisterBits::DecimalMode as u8) & 0x01 == 1;
        self.break_command = (b >> StatusRegisterBits::BreakCommand as u8) & 0x01 == 1;
        self.overflow_flag = (b >> StatusRegisterBits::OverflowFlag as u8) & 0x01 == 1;
        self.negative_flag = (b >> StatusRegisterBits::NegativeFlag as u8) & 0x01 == 1;
        self.unused_bit = (b >> StatusRegisterBits::UnusedBit as u8) & 0x01 == 1;
    }

    pub fn to_u8(&self) -> u8 {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
