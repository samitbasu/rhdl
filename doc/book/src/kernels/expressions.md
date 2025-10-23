# Expressions

Because almost everything in Rust is an expression, it makes the most sense to cover the RHDL subset of Rust in terms of allowable expression types.  In each of the following, I will try to highlight what limits there are (if any) to what you can use and still create things that are synthesizable in hardware.



            syn::Expr::Try(expr) => self.try_ex(expr),
            syn::Expr::Return(expr) => self.ret(expr),

            syn::Expr::MethodCall(expr) => self.method_call(expr),

