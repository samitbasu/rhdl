The missing piece is for the case that you want to define your own D and Q structs.
There are reasons why you would want to do this.  For example, constants do not require
a D signal.  So suppose the end user writes their own D and Q structs.  The place 
where this fails to work is in the `sim` method.   In the `sim` method, the 
simulation loop needs to know how to extract a given input from the struct.  My first
thought is that the `sim` method use `.into()`.   So

trait CircuitDQ {
    type D = MyD;
    type Q = MyQ;
}

trait Circuit : CircuitDQ {

    type AutoD = (c0::I, ... )
    type AutoQ = (c0::O, ... )

    fn sim() {
        let MyQ : Self::Q = autoq.into();
        let (output, MyD) = UPDATE(inputs, MyQ)
        let child_outputs: AutoD = MyD.into();
    }

}

This would allow you to (for example) have a single clock element in the D struct and 
share it across all of the child circuits.  That's neat.  But it means you need to write
a nasty function like:

impl From<MyQ> for AutoQ {
    // What goes here?
}

And

impl From<AutoD> for MyD {
    // What goes here?
}

The automatically generated versions of these _could_ be written on the assumption that
field names are the same.

impl From<MyQ> for AutoQ {
    fn from(val: MyQ) -> AutoQ {
        (val.c0, val.c1, ...) // Assumes each field is present
    }
}

impl From<AutoD> for MyD {
    fn from(val: AutoD) -> MyD {
        MyD {
            c0: val.0,
            c1: val.1,
            ...
        }
    }
}

You _could_ then opt out of using these if you want to customize the behavior somehow.

This at least has the benefit of not writing the D and Q structs for the end user.  Which
seems kind of wrong somehow.

BUT - that means that in the HDL generation, additional code needs to be generated to map
the child inputs/outputs to the provided D/Q structs.  That is definitely not going to work.


