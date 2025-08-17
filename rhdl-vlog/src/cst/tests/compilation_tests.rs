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

#[test]
fn test_always_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output reg[1:0] b);
            always @(posedge a, b) begin
                b <= 1;
            end
            endmodule
    ",
    )?;
    test_compilation("always_statement", modules);
    Ok(())
}

#[test]
fn test_case_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output reg[1:0] b);
            case (a)
                2'b00: b = 1;
                2'b01: b = 2;
                2'b10: b = 3;
                default: b = 4;
            endcase
            endmodule
    ",
    )?;
    test_compilation("case_statement", modules);
    Ok(())
}

#[test]
fn test_local_param_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output reg[1:0] b);
            localparam my_param = 5'b1_1001;
            always @(posedge a) begin
                b <= my_param;
            end
            endmodule
    ",
    )?;
    test_compilation("local_param_statement", modules);
    Ok(())
}

#[test]
fn test_continuous_assign_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output wire[1:0] b);
            assign b = a + 1;
            endmodule
    ",
    )?;
    test_compilation("continuous_assign_statement", modules);
    Ok(())
}

#[test]
fn test_non_block_assignment_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output reg[1:0] b);
            always @(posedge a) begin
                b <= 1;
            end
            endmodule
    ",
    )?;
    test_compilation("non_block_assignment_statement", modules);
    Ok(())
}

#[test]
fn test_instance_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output reg[1:0] b);
                bar_0 bar(.c(a), .d(b));
            endmodule
    ",
    )?;
    test_compilation("instance_statement", modules);
    Ok(())
}

#[test]
fn test_delay_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output reg[1:0] b);
            always @(posedge a, negedge b, *) begin
                b <= 1;
                # 10;
            end
            endmodule
    ",
    )?;
    test_compilation("delay_statement", modules);
    Ok(())
}

#[test]
fn test_concat_assignment_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, input wire[1:0] c, output reg[1:0] b);
                {a, c} = {1'b0, a};
            endmodule
    ",
    )?;
    test_compilation("concat_assignment_statement", modules);
    Ok(())
}

#[test]
fn test_function_call_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output reg[1:0] b);
            $my_function(a, b);
            $finish;
            endmodule
    ",
    )?;
    test_compilation("function_call_statement", modules);
    Ok(())
}

#[test]
fn test_splice_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            a[1] = b;
            a[1:0] = {b, b};
            endmodule
    ",
    )?;
    test_compilation("splice_statement", modules);
    Ok(())
}

#[test]
fn test_dynamic_splice_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            a[b +: 1] = 1;
            endmodule
    ",
    )?;
    test_compilation("dynamic_splice_statement", modules);
    Ok(())
}

#[test]
fn test_local_declaration_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            wire [4:0] val1;
            reg signed [3:0] val2;
            endmodule
    ",
    )?;
    test_compilation("local_declaration_statement", modules);
    Ok(())
}

#[test]
fn test_function_def_in_module() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            function [1:0] my_function(input wire[1:0] x);
                assign my_function = x + 1;
            endfunction
            b = my_function(a);
            endmodule
    ",
    )?;
    test_compilation("function_def_in_module", modules);
    Ok(())
}

#[test]
fn test_initial_statement_in_module() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            initial begin
                b = 0;
            end
            endmodule
    ",
    )?;
    test_compilation("initial_statement_in_module", modules);
    Ok(())
}

#[test]
fn test_unary_expressions() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            always @(*) begin
                b = +a;
                b = -a;
                b = ~a;
                b = !a;
                b = &a;
                b = ^a;
                b = |a;
            end
            endmodule
    ",
    )?;
    test_compilation("unary_expressions", modules);
    Ok(())
}

#[test]
fn test_ternary_expression() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            always @(*) begin
                b = (a == 1) ? 2 : 3;
            end
            endmodule
    ",
    )?;
    test_compilation("ternary_expression", modules);
    Ok(())
}

#[test]
fn test_replica_expression() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            always @(*) begin
                b = {3{a}};
            end
            endmodule
    ",
    )?;
    test_compilation("replica_expression", modules);
    Ok(())
}

#[test]
fn test_index_expressions() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            always @(*) begin
                b = a[1];
                b = a[1:0];
                b = a[b+:2];
            end
            endmodule
    ",
    )?;
    test_compilation("index_expressions", modules);
    Ok(())
}

#[test]
fn test_string_expression() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        r#"
            module foo(input wire[2:0] a, output reg[1:0] b);
            always @(*) begin
                $display("Hello, World!");
            end
            endmodule
            "#,
    )?;
    test_compilation("string_expression", modules);
    Ok(())
}

#[test]
fn test_binary_expressions() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            always @(*) begin
                b = a << 1;
                b = a >>> 2;
                b = a >> 2;
                b = a && a;
                b = a || a;
                b = a === 1;
                b = a !== 1;
                b = a != 1;
                b = a == 1;
                b = a >= 1;
                b = a <= 1;
                b = a > 1;
                b = a < 1;
                b = a + b;
                b = a - b;
                b = a & b;
                b = a | b;
                b = a ^ b;
                b = a % b;
                b = a * b;
            end
            endmodule
    ",
    )?;
    test_compilation("binary_expressions", modules);
    Ok(())
}
