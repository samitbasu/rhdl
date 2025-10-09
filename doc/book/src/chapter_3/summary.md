# Chapter 3 - A Half Adder

Our next step will be to construct a half adder.  While not particularly exciting in and of itself, it will demonstrate the vital importance of circuit _composition_.  In RHDL, we build circuits up from smaller components.  And that hierarchical implementation imposes some structure on our design.  Due to some issues in how the code generation works, there are also some silent conventions that need to be followed.  We can explain all of that as we go.

## Key Concepts
The key concepts of this chapter:

- Circuit composition via data structures
- Feedback types `D` and `Q`
- Custom types for `O` to improve readability

