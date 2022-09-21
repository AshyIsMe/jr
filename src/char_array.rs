use num_traits::Zero;
use std::ops::Add;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Char(char);

impl Char {
    pub fn new(c: char) -> Self {
        Char(c)
    }
}

impl Add<Self> for Char {
    type Output = Char;

    fn add(self, rhs: Self) -> Self::Output {
        Char((self.0 as u8 + rhs.0 as u8) as char)
    }
}

impl Zero for Char {
    fn zero() -> Self {
        Char('\0')
    }
    fn is_zero(&self) -> bool {
        self.0 == '\0'
    }
}
