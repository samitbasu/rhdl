use crate::cst::tests::common::test_parse_quote;
use crate::cst::{
    Declaration, Direction, Expr, HDLKind, LitVerilog, ModuleDef, Port, SignedWidth, Stmt,
    WidthSpec,
};

#[test]
fn test_to_tokens_signed_width() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/signed_width.expect"];
    let synth = test_parse_quote::<SignedWidth>("signed [4:0]")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_hdl() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/hdl_kind.expect"];
    let synth = test_parse_quote::<HDLKind>("reg")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_direction() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/direction.expect"];
    let synth = test_parse_quote::<Direction>("inout")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_signed_width() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/signed_width.expect"];
    let synth = test_parse_quote::<SignedWidth>("signed [4:0]")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_declaration() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/declaration.expect"];
    let synth = test_parse_quote::<Declaration>("wire signed [4:0] baz")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_literal() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/literal.expect"];
    let synth = test_parse_quote::<LitVerilog>("41'sd2332")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_width_spec() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/width_spec.expect"];
    let synth = test_parse_quote::<WidthSpec>("[4:0]")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_port() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/port.expect"];
    let synth = test_parse_quote::<Port>("input reg signed [3:0] nibble")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_expr() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/expr_complex.expect"];
    let synth = test_parse_quote::<Expr>("d+3+~(^4+4*c*6%8&5)")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_ternary() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/expr_ternary.expect"];
    let synth = test_parse_quote::<Expr>("a > 3 ? 1 : 7")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_concat() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/expr_concat.expect"];
    let synth = test_parse_quote::<Expr>("{a, 3, 1}")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_index() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/expr_index.expect"];
    let synth = test_parse_quote::<Expr>("a[3] + b[5:2] - {4 {w}}")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_higher_order() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/expr_higher_order.expect"];
    let synth = test_parse_quote::<Expr>("h[a +: 3]")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_signed() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/expr_signed.expect"];
    let synth = test_parse_quote::<Expr>("$signed(a)")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_stmt_if() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/stmt_if.expect"];
    let synth = test_parse_quote::<Stmt>(
        r"
begin
   if (a > 3)
      b = 4;
   else
      c = b;
end    
",
    )?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_stmt_case() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/stmt_case.expect"];
    let synth = test_parse_quote::<Stmt>(
        r"
case (rega)
16'd0: result = 10'b0111111111;
16'd1: result = 10'b1011111111;
16'd2: result = 10'b1101111111;
16'd3: result = 10'b1110111111;
16'd4: result = 10'b1111011111;
16'd5: result = 10'b1111101111;
16'd6: result = 10'b1111110111;
16'd7: result = 10'b1111111011;
16'd8: result = 10'b1111111101;
16'd9: result = 10'b1111111110;
default: result = 10'bx;
endcase
        ",
    )?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_module_def_empty() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/module_def_empty.expect"];
    let synth = test_parse_quote::<ModuleDef>("module foo; endmodule")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_module_def() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/module_def.expect"];
    let synth = test_parse_quote::<ModuleDef>(
        "
        module foo(input wire[2:0] clock_reset, input wire[7:0] i, output wire[7:0] o);
        endmodule        
",
    )?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_assignment_with_index() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/assignment_with_index.expect"];
    let synth = test_parse_quote::<ModuleDef>(
        "
      module foo(
          input wire [7:0] a,
          input wire [7:0] b,
          output wire [7:0] c
      );
      c = a[3] + b[5:2];
      c <= a[3] + b[5:2];
      c[a] <= b;
      c[a[3:0]] <= b;
      c[3:0] <= a;
      endmodule
    ",
    )?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_dff_definition() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/dff_definition.expect"];
    let synth = test_parse_quote::<ModuleDef>(
        "
        module foo(input wire[2:0] clock_reset, input wire[7:0] i, output wire[7:0] o);
           wire [0:0] clock;
           wire [0:0] reset;
           assign clock = clock_reset[0];
           assign wire = clock_reset[1];
           always @(posedge clock) begin
               if (reset) begin
                  o <= 8'b0;
                end else begin
                   o <= i;
                end
           end
        endmodule        
",
    )?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_dut() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/dut.expect"];
    let synth = test_parse_quote::<ModuleDef>(
        r"
module dut(input wire [7:0] arg_0, output reg [7:0] out);
    reg [0:0] r0;
    reg [0:0] r1;
    reg [0:0] r2;
    reg [0:0] r3;
    reg [0:0] r4;
    reg [0:0] r5;
    reg [0:0] r6;
    reg [0:0] r7;
    reg [0:0] r8;
    reg [0:0] r9;
    reg [0:0] r10;
    reg [0:0] r11;
    reg [0:0] r12;
    reg [0:0] r13;
    reg [0:0] r14;
    reg [0:0] r15;
    always @(*) begin
        r0 = arg_0[0];
        r1 = arg_0[1];
        r2 = arg_0[2];
        r3 = arg_0[3];
        r4 = arg_0[4];
        r5 = arg_0[5];
        r6 = arg_0[6];
        r7 = arg_0[7];
        // let b = Bar/* tuple::Bar */ {a: bits(1), b: Foo/* tuple::Foo */ {a: bits(2), b: bits(3),},};
        //
        // let Bar {a: a, b: Foo {a: c, b: d,},} = b;
        //
        // signal((a + c + d + a0.val()).resize())
        //
        { r15,r14,r13,r12,r11,r10,r9,r8 } = { 1'b0, 1'b0, 1'b0, 1'b0, 1'b0, 1'b1, 1'b1, 1'b0 } + { r7, r6, r5, r4, r3, r2, r1, r0 };
        out = { r15, r14, r13, r12, r11, r10, r9, r8 };
    end
endmodule
    ",
    )?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_dut_signed_compare() -> miette::Result<()> {
    let expect = expect_test::expect_file!["../expect/dut_signed_compare.expect"];
    let synth = test_parse_quote::<ModuleDef>(
        r"
    module dut(input wire [7:0] arg_0, input wire [7:0] arg_1, output reg [0:0] out);
    reg [0:0] r0;
    reg [0:0] r1;
    reg [0:0] r2;
    reg [0:0] r3;
    reg [0:0] r4;
    reg [0:0] r5;
    reg [0:0] r6;
    reg [0:0] r7;
    reg [0:0] r8;
    reg [0:0] r9;
    reg [0:0] r10;
    reg [0:0] r11;
    reg [0:0] r12;
    reg [0:0] r13;
    reg [0:0] r14;
    reg [0:0] r15;
    reg [0:0] r16;
    always @(*) begin
        r0 = arg_0[0];
        r1 = arg_0[1];
        r2 = arg_0[2];
        r3 = arg_0[3];
        r4 = arg_0[4];
        r5 = arg_0[5];
        r6 = arg_0[6];
        r7 = arg_0[7];
        r8 = arg_1[0];
        r9 = arg_1[1];
        r10 = arg_1[2];
        r11 = arg_1[3];
        r12 = arg_1[4];
        r13 = arg_1[5];
        r14 = arg_1[6];
        r15 = arg_1[7];
        // signal(a.val() >= b.val())
        //
        { r16 } = $signed({ r7, r6, r5, r4, r3, r2, r1, r0 }) >= $signed({ r15, r14, r13, r12, r11, r10, r9, r8 });
        out = { r16 };
    end
endmodule",
    )?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}
