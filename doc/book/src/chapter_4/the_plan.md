# The Plan

I always like to start with a sketch of what I am designing.  In this case, it's simple:

```badascii
          +--+Count Ones+----+      
          |                  |      
          |    +-------+     |      
  u8      |    |       |     | b4   
+---------+--->|Kernel +-----+----->
          |    |       |     |      
          |    +-------+     |      
          |                  |      
          +------------------+      
```

When fixtured, this will count the number of DIP switches in a bank of the IO board that are set, and display the result as a binary nibble.

The input to the circuit will be a `Signal<u8, Red>`, and the output will be something called a `b4` (or more precisely a `Signal<b4, Red>`).  This is a nibble that counts how many bits of the input `u8` are equal to `1`.  The maximum number of ones is 8, which requires 4 bits to represent, so twe need a 4 bit value on the output.  It's time to look at the `Bits` type and the `BitWidth` trait.

