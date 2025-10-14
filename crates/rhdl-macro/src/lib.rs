use proc_macro::TokenStream;

#[proc_macro_derive(Digital, attributes(rhdl))]
pub fn digital(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::derive_digital(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Timed)]
pub fn timed(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::derive_timed(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn kernel(attr: TokenStream, input: TokenStream) -> TokenStream {
    match rhdl_macro_core::hdl_kernel(attr.into(), input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Circuit, attributes(rhdl))]
pub fn circuit(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::derive_circuit(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(CircuitDQ)]
pub fn circuit_dq(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::derive_circuit_dq(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Synchronous, attributes(rhdl))]
pub fn synchronous(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::derive_synchronous(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(SynchronousDQ)]
pub fn synchronous_dq(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::derive_synchronous_dq(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn export(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::export_macro(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn path(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::path_macro(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn vlog_modules(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::vlog::modules(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn vlog_module(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::vlog::module(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
