use strum::{EnumIter, IntoEnumIterator};

#[derive(Clone, Copy, EnumIter, PartialEq, Eq, PartialOrd, Ord)]
#[repr(usize)]
pub enum Precedence {
    None = 0,
    Assignment = 1,  // =
    Or = 2,          // or
    And = 3,         // and
    Equality = 4,    // == !=
    Ccomparison = 5, // < > <= >=
    Term = 6,        // + -
    Factor = 7,      // * /
    Unary = 8,       // ! -
    Call = 9,        // . ()
    Primary = 10,
}

impl Into<usize> for Precedence {
    fn into(self) -> usize {
        return self as usize;
    }
}

impl From<usize> for Precedence {
    fn from(value: usize) -> Self {
        for p in Precedence::iter() {
            if value == p.clone().into() {
                return p;
            }
        }

        return Precedence::Primary;
    }
}

impl Precedence {
    pub fn higher(&self) -> Self {
        let a = *self as usize + 1;
        return a.into();
    }
}
