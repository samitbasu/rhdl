# Dynamic Operators

Here are the dynamic operators that either produce `DynBits` or work on `DynBits` arguments (or their signed variants).  You can find more detail in the `rhdl_bits` crate documentation.  


| Operator | Size Rule                  | Syntax      | Notes                                 |
|--------|----------------------------|-------------|---------------------------------------|
| `XAdd` | `N XAdd M -> N.Max(M) + 1` | `a.xadd(b)` | Output is signed if inputs are signed |
| `XSub` | `N XSub M -> N.max(M) + 1` | `a.xsub(b)` | Output is always signed | 
| `XMul` | `N XMul M -> N + M` | `a.xmul(b)` | Output is signed if inputs are signed |
| `XShl` | `N XShl K -> N + K` | `a.shl::<K>()` | Panics if `N+K > 128` |
| `XShr` | `N XShr K -> N - K` | `a.shr::<K>()` | Panics if `N-K < 1` |
| `XSgn` | `N -> N + 1` | `a.xsgn()` | Promotes unsigned to signed safely |
| `XNeg` | `N -> N + 1` | `a.xneg()` | Negates a value safely.  Also promotes an unsigned to signed. |
| `XExt` | `N Xext K -> N + K` | `a.xext::<K>()` | Pads the value by `K` bits (with sign extension) |




