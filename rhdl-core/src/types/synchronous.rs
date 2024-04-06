// A Synchronous design consists of the following pieces:
//
//  Input   - a Digital type that describes the input to the design
//  Output   - a Digital type that describes the output of the design
//  State   - a Digital type that describes the state of the design
//  Initial - the initial state of the design
//  Update  - the update function for the design.
//  Params  - the parameters of the design (held constant)

use crate::{Digital, DigitalFn};

pub trait Synchronous: Digital {
    type Input: Digital;
    type Output: Digital;
    type State: Digital + Default;
    type Update: DigitalFn;

    const INITIAL_STATE: Self::State;
    const UPDATE: UpdateFn<Self>;
}

pub type UpdateFn<T> = fn(
    T,
    <T as Synchronous>::State,
    <T as Synchronous>::Input,
) -> (<T as Synchronous>::State, <T as Synchronous>::Output);
