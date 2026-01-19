# Operator Expressions

The operators supported by RHDL are a (large) subset of the normal Rust operators, and follow Rust's operator precedence.  The following table summarizes the supported operators in descending precedence order, along with the associativity.

|Operator | Associativity |
|---|---|
| Unary `-` `!` | |
| `*` | left to right | 
| `+` `-` | left to right | 
| `<<` `>>` | left to right |
| `&` | left to right |
| `^` | left to right |
| `\|` | left to right |
| `==` `!=` `<` `>` `<=` `>=` | require parentheses |
| `&&` | left to right |
| `\|\|` | left to right |
| `=` `+=` `-=` `*=` `&=` `\|=` `^=` `<<=` `>>=` | right to left |

```admonish note
The missing operators are either division type operators `%` and `/` or the range operators, which are associated primarily with iteration.  None of these are straightforward in hardware, so we require extra work to invoke them.  This is mostly to ensure that you do not accidently write code that ends up requiring large and unwieldy circuitry.  Even the multiplication operator falls into that category, so be careful.
```

Because the operators behave the same in RHDL and rust, there isn't much more to explain.  The primary caveats around edge cases have already been covered in the [bits] section.  The following example shows some basic expression examples:

```rust
{{#rustdoc_include ../code/src/kernels/binary.rs:step_1}}
```
