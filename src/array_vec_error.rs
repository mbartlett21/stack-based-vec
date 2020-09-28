use core::fmt;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ArrayVecError {
    CapacityOverflow,
}

impl fmt::Display for ArrayVecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match *self {
            Self::CapacityOverflow => "It is not possible to add more elements",
        };
        write!(f, "{}", s)
    }
}
