use crate::{
    Declaration, Direction, Expr, HDLKind, LitVerilog, ModuleDef, ModuleList, Port, SignedWidth,
    Stmt, WidthSpec,
    formatter::Pretty,
    tests::{test_parse, test_parse_quote},
};

use test_log::test;

#[test]
fn test_to_tokens_signed_width() -> miette::Result<()> {
    let expect = expect_test::expect!["signed [4:0]"];
    let synth = test_parse_quote::<SignedWidth>("signed [4:0]")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_hdl() -> miette::Result<()> {
    let expect = expect_test::expect!["reg"];
    let synth = test_parse_quote::<HDLKind>("reg")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_direction() -> miette::Result<()> {
    let expect = expect_test::expect!["inout"];
    let synth = test_parse_quote::<Direction>("inout")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_signed_width() -> miette::Result<()> {
    let expect = expect_test::expect!["signed [4:0]"];
    let synth = test_parse_quote::<SignedWidth>("signed [4:0]")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_declaration() -> miette::Result<()> {
    let expect = expect_test::expect!["wire signed [4:0] baz"];
    let synth = test_parse_quote::<Declaration>("wire signed [4:0] baz")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_literal() -> miette::Result<()> {
    let expect = expect_test::expect!["41'sd2332"];
    let synth = test_parse_quote::<LitVerilog>("41'sd2332")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_width_spec() -> miette::Result<()> {
    let expect = expect_test::expect!["[4:0]"];
    let synth = test_parse_quote::<WidthSpec>("[4:0]")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_port() -> miette::Result<()> {
    let expect = expect_test::expect!["input reg signed [3:0] nibble"];
    let synth = test_parse_quote::<Port>("input reg signed [3:0] nibble")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_expr() -> miette::Result<()> {
    let expect = expect_test::expect!["d + 3 + ~(^4 + 4 * c * 6 % 8 & 5)"];
    let synth = test_parse_quote::<Expr>("d+3+~(^4+4*c*6%8&5)")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_ternary() -> miette::Result<()> {
    let expect = expect_test::expect!["a > 3 ? 1 : 7"];
    let synth = test_parse_quote::<Expr>("a > 3 ? 1 : 7")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_concat() -> miette::Result<()> {
    let expect = expect_test::expect!["{a, 3, 1}"];
    let synth = test_parse_quote::<Expr>("{a, 3, 1}")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_index() -> miette::Result<()> {
    let expect = expect_test::expect!["a[3] + b[5:2] - {4{w}}"];
    let synth = test_parse_quote::<Expr>("a[3] + b[5:2] - {4 {w}}")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_higher_order() -> miette::Result<()> {
    let expect = expect_test::expect!["h[a+:3]"];
    let synth = test_parse_quote::<Expr>("h[a +: 3]")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_signed() -> miette::Result<()> {
    let expect = expect_test::expect!["$signed(a)"];
    let synth = test_parse_quote::<Expr>("$signed(a)")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_equality() -> miette::Result<()> {
    let expect = expect_test::expect!["or2 = or1 == or0"];
    let synth = test_parse_quote::<Stmt>("or2 = or1 == or0;")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_equality_expression() -> miette::Result<()> {
    let expect = expect_test::expect!["or1 == or0"];
    let synth = test_parse_quote::<Expr>("or1 == or0")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_stmt_if() -> miette::Result<()> {
    let expect = expect_test::expect![[r#"
        begin
           if (a > 3) b = 4 else if (a < 7) b = 7 else c = b;
        end"#]];
    let synth = test_parse_quote::<Stmt>(
        r"
begin
   if (a > 3)
      b = 4;
   else if (a < 7)
      b = 7;
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
    let expect = expect_test::expect![[r#"
        case (rega)
           16'd0 : result = 10'b0111111111;
           16'd1 : result = 10'b1011111111;
           16'd2 : result = 10'b1101111111;
           16'd3 : result = 10'b1110111111;
           16'd4 : result = 10'b1111011111;
           16'd5 : result = 10'b1111101111;
           16'd6 : result = 10'b1111110111;
           16'd7 : result = 10'b1111111011;
           16'd8 : result = 10'b1111111101;
           16'd9 : result = 10'b1111111110;
           default : result = 10'bx;
        endcase"#]];
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
    let expect = expect_test::expect![[r#"
        module foo();
        endmodule"#]];
    let synth = test_parse_quote::<ModuleDef>("module foo; endmodule")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_module_def() -> miette::Result<()> {
    let expect = expect_test::expect![[r#"
        module foo(input wire [2:0] clock_reset, input wire [7:0] i, output wire [7:0] o);
        endmodule"#]];
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
fn test_case_uneq() -> miette::Result<()> {
    let expect = expect_test::expect![[r#"
        if ((32'b123) !== (junk)) begin
           $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE foo", 32'b123, junk);
           $finish;
        end"#]];
    let synth = test_parse_quote::<Stmt>(
        "
    if ((32'b123) !== (junk)) begin
        $display(\"ASSERTION FAILED 0x%0h !== 0x%0h CASE foo\", 32'b123, junk);
        $finish;
    end
    ",
    )?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_module_arg_issue() -> miette::Result<()> {
    let expect = expect_test::expect![[r#"
        module uut(input wire [1:0] clock_reset, output wire [0:0] o);
           wire [4:0] od;
           wire [3:0] d;
           wire [0:0] q;
           assign o = od[0:0];
           uut_anyer c0(.clock_reset(clock_reset), .i(d[3:0]), .o(q[0:0]));
           assign d = od[4:1];
           assign od = kernel_parent(clock_reset, q);
           function [4:0] kernel_parent(input reg [1:0] arg_0, input reg [0:0] arg_2);
                 reg [0:0] or0;
                 reg [4:0] or1;
                 reg [1:0] or2;
                 localparam ol0 = 4'b0011;
                 begin
                    or2 = arg_0;
                    or0 = arg_2;
                    or1 = {ol0, or0};
                    kernel_parent = or1;
                 end
           endfunction
        endmodule
    "#]];
    let synth = test_parse_quote::<ModuleList>(
        "module uut (input wire [1 : 0] clock_reset , output wire [0 : 0] o) ; wire [4 : 0] od ;
         wire [3 : 0] d ; wire [0 : 0] q ; assign o = od [0 : 0] ; uut_anyer c0 (. clock_reset (clock_reset) , 
         . i (d [3 : 0]) , . o (q [0 : 0])) ; assign d = od [4 : 1] ; assign od = kernel_parent (clock_reset , q) ;
         function [4 : 0] kernel_parent (input reg [1 : 0] arg_0 , input reg [0 : 0] arg_2) ; reg [0 : 0] or0 ; 
         reg [4 : 0] or1 ; reg [1 : 0] or2 ; localparam ol0 = 4 'b0011 ; begin or2 = arg_0 ; or0 = arg_2 ; 
         or1 = { ol0 , or0 } ; kernel_parent = or1 ; end endfunction endmodule",
    )?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_assignment_with_index() -> miette::Result<()> {
    let expect = expect_test::expect![[r#"
        module foo(input wire [7:0] a, input wire [7:0] b, output wire [7:0] c);
           c = a[3] + b[5:2];
           c <= a[3] + b[5:2];
           c[a] <= b;
           c[a[3:0]] <= b;
           c[3:0] <= a;
        endmodule"#]];
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
    let expect = expect_test::expect![[r#"
        module foo(input wire [2:0] clock_reset, input wire [7:0] i, output wire [7:0] o);
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
        endmodule"#]];
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
    let expect = expect_test::expect![[r#"
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
              {r15, r14, r13, r12, r11, r10, r9, r8} = {1'b0, 1'b0, 1'b0, 1'b0, 1'b0, 1'b1, 1'b1, 1'b0} + {r7, r6, r5, r4, r3, r2, r1, r0};
              out = {r15, r14, r13, r12, r11, r10, r9, r8};
           end
        endmodule"#]];
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
    let expect = expect_test::expect![[r#"
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
              {r16} = $signed({r7, r6, r5, r4, r3, r2, r1, r0}) >= $signed({r15, r14, r13, r12, r11, r10, r9, r8});
              out = {r16};
           end
        endmodule"#]];
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

const KITCHEN_SINK: &str = r#"
        module foo(input wire[2:0] clock_reset, input wire[7:0] i, output wire signed [7:0] o, inout wire baz);
           wire [0:0] clock;
           wire [0:0] reset;
           reg [3:0] a, b, c;
           reg [7:0] memory[15:0];
           wire foo;
           assign clock = clock_reset[0];
           assign wire = clock_reset[1];
           assign o = {i, i};
           assign o = {3 {i}};
           a[b +: 1] = clock;
           a[1:0] = reset;
           if (c <= 3) begin
              {a, b} = c;
           end
           localparam cost = 42;
           localparam bar = 16'd16;
           localparam pie = "apple";
           obj obj(.clk(clock), .reset(reset), .i(i), .o(o));
           initial begin
               o = 8'b0;
               # 10;
               o = add(8'b0, 8'b1) + !c;
               o = (a > b) ? a : b[o -: 4];
               $display("o = 2");
               o = (a > b) && (b < 6);
               o = (a == b) || (b == 3);
               o = a >> 3;
               o = a >>> 3;
               o = a === 3;
               o = a !== 4;
               o = a | b;
               o = a ^ b;
               o = a != 3;
               o = a << 2;
               o = a >= b;
               o = a % b;
               o = a * b - c & d;
               o = -b;
               o = ~b;
               o = &b;
               o = |b;
               o = ^b;
               o = +b;
           end
           always @(posedge clock, negedge reset, foo, *) begin
               if (reset) begin
                  o <= 8'b0;
                end else begin
                   o <= i;
                end
           end
           case (rega)
            16'd0: result = 10'b0111111111;
            16'd1: result = 10'b1011111111;
            16'd2: result = 10'b1101111111;
            16'd3: result = 10'b1110111111;
            START: result = 10'b1110111111;
            16'd4: result = 10'b1111011111;
            16'd5: result = 10'b1111101111;
            16'd6: result = 10'b1111110111;
            16'd7: result = 10'b1111111011;
            16'd8: result = 10'b1111111101;
            16'd9: result = 10'b1111111110;
            default: result = 10'bx;
          endcase
          function [3:0] add(input wire[3:0] a, input wire[3:0] b);
            begin
              add = a + b;
            end
          endfunction
        endmodule
"#;

#[test]
fn test_parse_kitchen_sink() -> miette::Result<()> {
    let expect = expect_test::expect![[r#"
        module foo(input wire [2:0] clock_reset, input wire [7:0] i, output wire signed [7:0] o, inout wire  baz);
           wire [0:0] clock;
           wire [0:0] reset;
           reg [3:0] a, b, c;
           reg [7:0] memory[15:0];
           wire  foo;
           assign clock = clock_reset[0];
           assign wire = clock_reset[1];
           assign o = {i, i};
           assign o = {3{i}};
           a[b+:1] = clock;
           a[1:0] = reset;
           if (c <= 3) begin
              {a, b} = c;
           end
           localparam cost = 42;
           localparam bar = 16'd16;
           localparam pie = "apple";
           obj obj(.clk(clock), .reset(reset), .i(i), .o(o));
           initial begin
              o = 8'b0;
              #10;
              o = add(8'b0, 8'b1) + !c;
              o = (a > b) ? a : b[o-:4];
              $display("o = 2");
              o = (a > b) && (b < 6);
              o = (a == b) || (b == 3);
              o = a >> 3;
              o = a >>> 3;
              o = a === 3;
              o = a !== 4;
              o = a | b;
              o = a ^ b;
              o = a != 3;
              o = a << 2;
              o = a >= b;
              o = a % b;
              o = a * b - c & d;
              o = -b;
              o = ~b;
              o = &b;
              o = |b;
              o = ^b;
              o = +b;
           end
           always @(posedge clock, negedge reset, foo, *) begin
              if (reset) begin
                 o <= 8'b0;
              end else begin
                 o <= i;
              end
           end
           case (rega)
              16'd0 : result = 10'b0111111111;
              16'd1 : result = 10'b1011111111;
              16'd2 : result = 10'b1101111111;
              16'd3 : result = 10'b1110111111;
              START : result = 10'b1110111111;
              16'd4 : result = 10'b1111011111;
              16'd5 : result = 10'b1111101111;
              16'd6 : result = 10'b1111110111;
              16'd7 : result = 10'b1111111011;
              16'd8 : result = 10'b1111111101;
              16'd9 : result = 10'b1111111110;
              default : result = 10'bx;
           endcase
           function [3:0] add(input wire [3:0] a, input wire [3:0] b);
                 begin
                    add = a + b;
                 end
           endfunction
        endmodule
    "#]];
    let synth = test_parse_quote::<ModuleList>(KITCHEN_SINK)?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_pretty_kitchen_sink() -> miette::Result<()> {
    let expect = expect_test::expect![[r#"
        module foo(input wire [2:0] clock_reset, input wire [7:0] i, output wire signed [7:0] o, inout wire  baz);
           wire [0:0] clock;
           wire [0:0] reset;
           reg [3:0] a, b, c;
           reg [7:0] memory[15:0];
           wire  foo;
           assign clock = clock_reset[0];
           assign wire = clock_reset[1];
           assign o = {i, i};
           assign o = {3{i}};
           a[b+:1] = clock;
           a[1:0] = reset;
           if (c <= 3) begin
              {a, b} = c;
           end
           localparam cost = 42;
           localparam bar = 16'd16;
           localparam pie = "apple";
           obj obj(.clk(clock), .reset(reset), .i(i), .o(o));
           initial begin
              o = 8'b0;
              #10;
              o = add(8'b0, 8'b1) + !c;
              o = (a > b) ? a : b[o-:4];
              $display("o = 2");
              o = (a > b) && (b < 6);
              o = (a == b) || (b == 3);
              o = a >> 3;
              o = a >>> 3;
              o = a === 3;
              o = a !== 4;
              o = a | b;
              o = a ^ b;
              o = a != 3;
              o = a << 2;
              o = a >= b;
              o = a % b;
              o = a * b - c & d;
              o = -b;
              o = ~b;
              o = &b;
              o = |b;
              o = ^b;
              o = +b;
           end
           always @(posedge clock, negedge reset, foo, *) begin
              if (reset) begin
                 o <= 8'b0;
              end else begin
                 o <= i;
              end
           end
           case (rega)
              16'd0 : result = 10'b0111111111;
              16'd1 : result = 10'b1011111111;
              16'd2 : result = 10'b1101111111;
              16'd3 : result = 10'b1110111111;
              START : result = 10'b1110111111;
              16'd4 : result = 10'b1111011111;
              16'd5 : result = 10'b1111101111;
              16'd6 : result = 10'b1111110111;
              16'd7 : result = 10'b1111111011;
              16'd8 : result = 10'b1111111101;
              16'd9 : result = 10'b1111111110;
              default : result = 10'bx;
           endcase
           function [3:0] add(input wire [3:0] a, input wire [3:0] b);
                 begin
                    add = a + b;
                 end
           endfunction
        endmodule"#]];
    let synth = syn::parse_str::<ModuleDef>(KITCHEN_SINK).unwrap();
    expect.assert_eq(&synth.pretty());
    Ok(())
}

#[test]
fn test_pretty_kitchen_idempotence() -> miette::Result<()> {
    let synth = test_parse::<ModuleDef>(KITCHEN_SINK)?;
    let pretty = synth.pretty();
    let synth2 = test_parse::<ModuleDef>(&pretty)?;
    let pretty2 = synth2.pretty();
    assert_eq!(pretty, pretty2);
    Ok(())
}
