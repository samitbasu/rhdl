use crate::cst::{
    ModuleList,
    tests::common::{test_compilation, test_parse},
};

/// Test that an empty module declaration generates compilable code
#[test]
fn test_empty_module() {
    let module = syn::parse_str::<ModuleList>(
        "
            module foo;
            endmodule
    ",
    )
    .unwrap();
    test_compilation("empty", module);
}

#[test]
fn test_module_with_ports() {
    let module = syn::parse_str::<ModuleList>(
        "
            module foo(input wire[1:0] a, output wire[1:0] b);
            endmodule
    ",
    )
    .unwrap();
    test_compilation("module_with_ports", module);
}

#[test]
fn test_multiple_modules_with_ports() {
    let modules = syn::parse_str::<ModuleList>(
        "
            module foo(input wire[1:0] a, output wire[1:0] b);
            endmodule

            module bar(input wire[1:0] c, output wire[1:0] d);
            endmodule
    ",
    )
    .unwrap();
    test_compilation("multiple_modules_with_ports", modules);
}

#[test]
// Test with and without signed, with widths and both reg and wire
fn test_module_with_different_port_types() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output wire[1:0] b, inout reg[3:0] c);
            endmodule

            module bar(input wire signed[1:0] c, output reg signed [1:0] d);
            endmodule
    ",
    )?;
    test_compilation("module_with_different_port_types", modules);
    Ok(())
}

#[test]
fn test_if_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output wire[1:0] b);
            if (a) begin
                b = 1;
            end else begin
                b = 0;
            end
            endmodule
    ",
    )?;
    test_compilation("if_statement", modules);
    Ok(())
}

#[test]
fn test_if_else_if_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output wire[1:0] b);
            if (a) begin
                b = 1;
            end else if (a == 1) begin
                b = 2;
            end else begin
                b = 0;
            end
            endmodule
    ",
    )?;
    test_compilation("if_else_if_statement", modules);
    Ok(())
}

#[test]
fn test_if_no_else_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output wire[1:0] b);
            if (a) begin
                b = 1;
            end
            endmodule
    ",
    )?;
    test_compilation("if_no_else_statement", modules);
    Ok(())
}
