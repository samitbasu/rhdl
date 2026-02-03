# Tracing

Kernel functions are _almost_ pure.  The exception is tracing, which is a write-only side effect that allows you to expose variables expressions from inside your design to RHDL's simulation engine.  These are useful when you are trying to debug issues in your design, and simply seeing the inputs and outputs of the kernel are not sufficient.

In these cases, you can use the tracing functionality to add any value of type `T: Digital` to your trace output.  You can even trace values conditionally, so that they appear in the trace output only when needed.  

```admonish note
When working with `Circuit` and `Synchronous`, inputs and outputs to the kernels are automatically traced in the generated code.  You don't need to explicitly trace unless you are interested in some internal detail of a kernel that isn't exposed via its arguments or return types.
```
