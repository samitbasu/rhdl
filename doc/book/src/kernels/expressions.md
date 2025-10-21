# Expressions

Because almost everything in Rust is an expression, it makes the most sense to cover the RHDL subset of Rust in terms of allowable expression types.  In each of the following, I will try to highlight what limits there are (if any) to what you can use and still create things that are synthesizable in hardware.


            syn::Expr::Path(expr) => self.path(&expr.path),
            syn::Expr::Struct(expr) => self.struct_ex(expr),
            syn::Expr::Field(expr) => self.field_expression(expr),
            syn::Expr::If(expr) => self.if_ex(expr),
            syn::Expr::Match(expr) => self.match_ex(expr),
            syn::Expr::Range(expr) => self.range(expr),
            syn::Expr::Try(expr) => self.try_ex(expr),
            syn::Expr::Return(expr) => self.ret(expr),
            syn::Expr::Tuple(expr) => self.tuple(expr),
            syn::Expr::Repeat(expr) => self.repeat(expr),
            syn::Expr::ForLoop(expr) => self.for_loop(expr),
            syn::Expr::Call(expr) => self.call(expr),
            syn::Expr::Array(expr) => self.array(expr),
            syn::Expr::Index(expr) => self.index(expr),
            syn::Expr::MethodCall(expr) => self.method_call(expr),
            syn::Expr::Cast(expr) => self.cast(expr),
