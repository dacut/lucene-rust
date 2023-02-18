use {
    std::fmt::{Debug, Display, Formatter, Result as FmtResult},
};

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct State(pub u32);

impl State {
    #[inline]
    pub fn usize(self) -> usize {
        self.0 as usize
    }
}

impl Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "State({})", self.0)
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "State({})", self.0)
    }
}
