use std::fmt::Debug;

pub type LurkResult<T> = Result<T, String>;

//////////////////////////////////////////////////////////////////////////////////////
// Result checking
//////////////////////////////////////////////////////////////////////////////////////
pub trait ResultLinkChecker {
    fn check<T: Debug, E>(&mut self, result: &Result<T, E>) -> &mut Self;
}

pub struct ResultChainChecker {
    error_found: bool,
}

impl ResultChainChecker {
    pub fn new() -> ResultChainChecker {
        ResultChainChecker { error_found: false }
    }

    pub fn success(&self) -> bool {
        !self.error_found
    }
}

impl ResultLinkChecker for ResultChainChecker {
    fn check<F: Debug, E>(&mut self, result: &Result<F, E>) -> &mut Self {
        if result.is_err() {
            self.error_found = true;
        }

        self
    }
}
//////////////////////////////////////////////////////////////////////////////////////
// Byte to bit field
//////////////////////////////////////////////////////////////////////////////////////
pub struct BitField {
    pub field: u8,
}

impl BitField {
    pub fn get(&self, index: u8) -> bool {
        self.field & (1 << index) != 0
    }

    pub fn set(&mut self, index: u8) {
        self.field |= 1 << index
    }

    pub fn clear(&mut self, index: u8) {
        self.field &= !(1 << index)
    }

    pub fn configure(&mut self, index: u8, status: bool) {
        if status {
            self.set(index);
        } else {
            self.clear(index);
        }
    }
}
//////////////////////////////////////////////////////////////////////////////////////
