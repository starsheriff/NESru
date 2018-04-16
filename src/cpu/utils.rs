/// most significatn bit
const msb: u8 = 0x01 << 7;

/// Checks if the overflow bit has to be set. `A` and `B` denote the values of
/// the operands of an arithmetic operation, `C` the result of the operation.
///
///
/// num1sign num2sign sumsign
/// A B C     ^
/// ---------------------------
/// 0 0 0 | 0
/// 0 0 1 | 1 (adding two positives should be positive)
/// 0 1 0 | 0
/// 0 1 1 | 0
/// 1 0 0 | 0
/// 1 0 1 | 0
/// 1 1 0 | 1 (adding two negatives should be negative)
/// 1 1 1 | 0
///
/// only two rows generate a positive output, that means a _sum of
/// product_ appreach is well suited to build the binary expression.
///
/// 0 0 1 | 1 !A !B  C
/// 1 1 0 | 1  A  B !C
///
/// !A !B C + A B !C = (!A & !B & C) | (A & B & !C)
///
/// In our case we have a 8bit value and are only interested in the most
/// significant bit. Therefore we take the bitwise _and_ of the last expression
/// with the bit pattern for the most significant bit and then test if it is
/// equal to the value of the most significant bit.
///
///
///
///
pub fn calculate_overflow_bit(a: u8, b: u8, c: u8) -> bool {
    return ((!a & !b & c) | (a & b & !c)) & msb == msb;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn msb_value() {
        assert_eq!(msb, 128);
    }

    #[test]
    fn overflow_case_1() {
        let a: u8 = 80;
        let b: u8 = 80;
        let c = a + b;

        println!("c as i8 is: {}", c as i8);
        assert_eq!(c as i8, -96);
        assert!(calculate_overflow_bit(a, b, c))
    }

    #[test]
    fn overflow_case_2() {
        let a: u8 = 150;
        let b: u8 = 150;
        let c = a.wrapping_add(b);

        assert_eq!(c as i8, 44);
        assert!(calculate_overflow_bit(a, b, c))
    }

    #[test]
    fn non_overlfow_cases() {
        assert!(!calculate_overflow_bit(1, 1, 2));
        assert!(!calculate_overflow_bit(128, 2, 130));
        assert!(!calculate_overflow_bit(256, 1, 0));
        assert!(!calculate_overflow_bit(2, 128, 130));
        assert!(!calculate_overflow_bit(4, 256, 3));
        assert!(!calculate_overflow_bit(200, 200, 144));
    }
}
