# Chapter 1 - The Basics
Here is a good example:
```pikchr
S1: move; 
B1: box radius 2mm "Kernel"; 
down;
move 4mm;
C1: box radius 1mm "Child 1" height 6mm;
move 4mm;
C2: box radius 1mm "Child 2" height 6mm;
arrow "input" above from 3mm above S1.w to 3mm above B1.w;
arrow "output" above from 3mm above B1.e right;
arrow  \
   from 3mm below B1.e \
   right 1cm \
   then down until even with C1.e \
   then left to C1.e;
arrow  \
   from 3mm below B1.e \
   right 1cm \
   then down until even with C2.e \
   then left to C2.e;
arrow \
   from C1.w \
   left 1cm \
   then up until even with 3mm below B1.w \
   then right to 3mm below B1.w;
arrow \
   from C2.w \
   left 1cm \
   then up until even with 3mm below B1.w \
   then right to 3mm below B1.w;
line from 3mm below B1.e "D" above invisible right 1cm;
line from 3mm below B1.w left 1cm "Q" above right 1cm;
dot at 3mm below C2.s;
dot at 5mm below C2.s;
dot at 7mm below C2.s;
```