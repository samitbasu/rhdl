use rhdl::prelude::*;

#[allow(clippy::let_and_return)]
pub mod step_1 {
    use super::*;
    // ANCHOR: step_1
    #[kernel]
    fn kernel(a: b8, b: bool) -> b8 {
        let c = a + 1;
        if b {
            return c; // 👈 Early return
        }
        let c = c + a;
        c
    }
    // ANCHOR_END: step_1
}

pub mod step_2 {
    use super::*;
    // ANCHOR: step_2
    #[derive(Copy, Clone, PartialEq, Digital, Default)]
    pub enum InputError {
        TooBig,
        TooSmall,
        #[default]
        UnknownError,
    }

    #[kernel]
    pub fn validate_input(a: b8) -> Result<b8, InputError> {
        if a < 10 {
            Err(InputError::TooSmall)
        } else if a > 200 {
            Err(InputError::TooBig)
        } else {
            Ok(a)
        }
    }

    #[kernel]
    pub fn kernel(a: b8, b: b8) -> Result<b8, InputError> {
        let a = validate_input(a)?;
        let b = validate_input(b)?;
        Ok(a + b)
    }
    // ANCHOR_END: step_2
}

pub mod step_3 {
    use super::*;
    // ANCHOR: step_3
    #[kernel]
    pub fn kernel(a: Option<b8>) -> Option<b8> {
        let a = a?;
        Some(a + 1)
    }
    // ANCHOR_END: step_3
}
