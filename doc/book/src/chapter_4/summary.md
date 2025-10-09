# Chapter 4 - Count Ones

We will now level up a bit in complexity.  We have a bunch of DIP switches, and we want to know how many are set to 1.  Why?  For Reasons™.  This is interesting, because doing it with plain gates is painful.  But it only takes a few lines of RHDL to make a functioning circuit.  

## Key Concepts
The key concepts of this chapter:

- The `Bits` type and the `BitWidth` trait
- Using `for` loops in synthesizable code
- Estimating timing
- Synthesizable pure functions
- Using generic bit widths

