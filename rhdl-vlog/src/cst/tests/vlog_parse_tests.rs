use crate::cst::tests::common::test_parse_quote;
use crate::cst::{
    Declaration, Direction, Expr, HDLKind, LitVerilog, ModuleDef, Port, SignedWidth, Stmt,
    WidthSpec,
};

#[test]
fn test_to_tokens_signed_width() -> miette::Result<()> {
    let expect = expect_test::expect!["rhdl :: vlog :: SignedWidth :: Signed (0 ..= 4)"];
    let synth = test_parse_quote::<SignedWidth>("signed [4:0]")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_hdl() -> miette::Result<()> {
    let expect = expect_test::expect!["rhdl :: vlog :: HDLKind :: Reg"];
    let synth = test_parse_quote::<HDLKind>("reg")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_direction() -> miette::Result<()> {
    let expect = expect_test::expect!["rhdl :: vlog :: Direction :: Inout"];
    let synth = test_parse_quote::<Direction>("inout")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_signed_width() -> miette::Result<()> {
    let expect = expect_test::expect!["rhdl :: vlog :: SignedWidth :: Signed (0 ..= 4)"];
    let synth = test_parse_quote::<SignedWidth>("signed [4:0]")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_declaration() -> miette::Result<()> {
    let expect = expect_test::expect![
        "rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Wire , signed_width : rhdl :: vlog :: SignedWidth :: Signed (0 ..= 4) , name : stringify ! (baz) . into () }"
    ];
    let synth = test_parse_quote::<Declaration>("wire signed [4:0] baz")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_literal() -> miette::Result<()> {
    let expect = expect_test::expect![
        "rhdl :: vlog :: LitVerilog { width : 41 , lifetime : stringify ! (sd2332) . into () , }"
    ];
    let synth = test_parse_quote::<LitVerilog>("41'sd2332")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_width_spec() -> miette::Result<()> {
    let expect = expect_test::expect!["0 ..= 4"];
    let synth = test_parse_quote::<WidthSpec>("[4:0]")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_port() -> miette::Result<()> {
    let expect = expect_test::expect!["rhdl :: vlog :: Port { direction : rhdl :: vlog :: Direction :: Input , decl : rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Signed (0 ..= 3) , name : stringify ! (nibble) . into () } , }"];
    let synth = test_parse_quote::<Port>("input reg signed [3:0] nibble")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_expr() -> miette::Result<()> {
    let expect = expect_test::expect![
        "rhdl :: vlog :: Expr :: Binary (rhdl :: vlog :: ExprBinary { lhs : rhdl :: vlog :: Expr :: Binary (rhdl :: vlog :: ExprBinary { lhs : rhdl :: vlog :: Expr :: Ident (d) , op : rhdl :: vlog :: BinaryOp :: Plus , rhs : rhdl :: vlog :: Expr :: Literal (3) }) , op : rhdl :: vlog :: BinaryOp :: Plus , rhs : rhdl :: vlog :: Expr :: Unary (rhdl :: vlog :: ExprUnary { op : rhdl :: vlog :: UnaryOp :: Not , arg : Box :: new (rhdl :: vlog :: Expr :: Paren (Box :: new (rhdl :: vlog :: Expr :: Binary (rhdl :: vlog :: ExprBinary { lhs : rhdl :: vlog :: Expr :: Binary (rhdl :: vlog :: ExprBinary { lhs : rhdl :: vlog :: Expr :: Unary (rhdl :: vlog :: ExprUnary { op : rhdl :: vlog :: UnaryOp :: Xor , arg : Box :: new (rhdl :: vlog :: Expr :: Literal (4)) , }) , op : rhdl :: vlog :: BinaryOp :: Plus , rhs : rhdl :: vlog :: Expr :: Binary (rhdl :: vlog :: ExprBinary { lhs : rhdl :: vlog :: Expr :: Binary (rhdl :: vlog :: ExprBinary { lhs : rhdl :: vlog :: Expr :: Binary (rhdl :: vlog :: ExprBinary { lhs : rhdl :: vlog :: Expr :: Literal (4) , op : rhdl :: vlog :: BinaryOp :: Mul , rhs : rhdl :: vlog :: Expr :: Ident (c) }) , op : rhdl :: vlog :: BinaryOp :: Mul , rhs : rhdl :: vlog :: Expr :: Literal (6) }) , op : rhdl :: vlog :: BinaryOp :: Mod , rhs : rhdl :: vlog :: Expr :: Literal (8) }) }) , op : rhdl :: vlog :: BinaryOp :: And , rhs : rhdl :: vlog :: Expr :: Literal (5) })))) , }) })"
    ];
    let synth = test_parse_quote::<Expr>("d+3+~(^4+4*c*6%8&5)")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_ternary() -> miette::Result<()> {
    let expect = expect_test::expect![
        "rhdl :: vlog :: Expr :: Ternary (rhdl :: vlog :: ExprTernary { lhs : rhdl :: vlog :: Expr :: Binary (rhdl :: vlog :: ExprBinary { lhs : rhdl :: vlog :: Expr :: Ident (a) , op : rhdl :: vlog :: BinaryOp :: Gt , rhs : rhdl :: vlog :: Expr :: Literal (3) }) , mhs : rhdl :: vlog :: Expr :: Literal (1) , rhs : rhdl :: vlog :: Expr :: Literal (7) , })"
    ];
    let synth = test_parse_quote::<Expr>("a > 3 ? 1 : 7")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_concat() -> miette::Result<()> {
    let expect = expect_test::expect![
        "rhdl :: vlog :: Expr :: Concat (vec ! [rhdl :: vlog :: Expr :: Ident (a) , rhdl :: vlog :: Expr :: Literal (3) , rhdl :: vlog :: Expr :: Literal (1) ,])"
    ];
    let synth = test_parse_quote::<Expr>("{a, 3, 1}")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_index() -> miette::Result<()> {
    let expect = expect_test::expect![
        "rhdl :: vlog :: Expr :: Binary (rhdl :: vlog :: ExprBinary { lhs : rhdl :: vlog :: Expr :: Binary (rhdl :: vlog :: ExprBinary { lhs : rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (a) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (3)) , lsb : . map (Box :: new) , } , }) , op : rhdl :: vlog :: BinaryOp :: Plus , rhs : rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (b) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (5)) , lsb : rhdl :: vlog :: Expr :: Literal (2) . map (Box :: new) , } , }) }) , op : rhdl :: vlog :: BinaryOp :: Minus , rhs : rhdl :: vlog :: Expr :: Replica (rhdl :: vlog :: ExprReplica { count : 4 , concatenation : vec ! [rhdl :: vlog :: Expr :: Ident (w) ,] , }) })"
    ];
    let synth = test_parse_quote::<Expr>("a[3] + b[5:2] - {4 {w}}")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_higher_order() -> miette::Result<()> {
    let expect = expect_test::expect![
        "rhdl :: vlog :: Expr :: DynIndex (rhdl :: vlog :: ExprDynIndex { target : stringify ! (h) . into () , base : Box :: new (rhdl :: vlog :: Expr :: Ident (a)) , width : Box :: new (rhdl :: vlog :: Expr :: Literal (3)) , })"
    ];
    let synth = test_parse_quote::<Expr>("h[a +: 3]")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_signed() -> miette::Result<()> {
    let expect = expect_test::expect![
        "rhdl :: vlog :: Expr :: Function (rhdl :: vlog :: ExprFunction { name : _signed , args : vec ! [rhdl :: vlog :: Expr :: Ident (a) ,] })"
    ];
    let synth = test_parse_quote::<Expr>("$signed(a)")?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_stmt_if() -> miette::Result<()> {
    let expect = expect_test::expect![
        "rhdl :: vlog :: Stmt :: Block (rhdl :: vlog :: Stmt :: Block (vec ! [rhdl :: vlog :: Stmt :: If (rhdl :: vlog :: If { condition : Box :: new (rhdl :: vlog :: Expr :: Binary (rhdl :: vlog :: ExprBinary { lhs : rhdl :: vlog :: Expr :: Ident (a) , op : rhdl :: vlog :: BinaryOp :: Gt , rhs : rhdl :: vlog :: Expr :: Literal (3) })) , true_stmt : Box :: new (rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (b) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Literal (4)) , })) , else_branch : rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (c) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Ident (b)) , }) . map (Box :: new) , }) ,]))"
    ];
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
    let expect = expect_test::expect![
        "rhdl :: vlog :: Stmt :: Case (rhdl :: vlog :: Case { discriminant : Box :: new (rhdl :: vlog :: Expr :: Ident (rega)) , lines : vec ! [rhdl :: vlog :: CaseLine { item : rhdl :: vlog :: CaseItem :: Literal (rhdl :: vlog :: LitVerilog { width : 16 , lifetime : stringify ! (d0) . into () , }) , stmt : Box :: new (rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (result) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 10 , lifetime : stringify ! (b0111111111) . into () , })) , })) , } , rhdl :: vlog :: CaseLine { item : rhdl :: vlog :: CaseItem :: Literal (rhdl :: vlog :: LitVerilog { width : 16 , lifetime : stringify ! (d1) . into () , }) , stmt : Box :: new (rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (result) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 10 , lifetime : stringify ! (b1011111111) . into () , })) , })) , } , rhdl :: vlog :: CaseLine { item : rhdl :: vlog :: CaseItem :: Literal (rhdl :: vlog :: LitVerilog { width : 16 , lifetime : stringify ! (d2) . into () , }) , stmt : Box :: new (rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (result) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 10 , lifetime : stringify ! (b1101111111) . into () , })) , })) , } , rhdl :: vlog :: CaseLine { item : rhdl :: vlog :: CaseItem :: Literal (rhdl :: vlog :: LitVerilog { width : 16 , lifetime : stringify ! (d3) . into () , }) , stmt : Box :: new (rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (result) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 10 , lifetime : stringify ! (b1110111111) . into () , })) , })) , } , rhdl :: vlog :: CaseLine { item : rhdl :: vlog :: CaseItem :: Literal (rhdl :: vlog :: LitVerilog { width : 16 , lifetime : stringify ! (d4) . into () , }) , stmt : Box :: new (rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (result) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 10 , lifetime : stringify ! (b1111011111) . into () , })) , })) , } , rhdl :: vlog :: CaseLine { item : rhdl :: vlog :: CaseItem :: Literal (rhdl :: vlog :: LitVerilog { width : 16 , lifetime : stringify ! (d5) . into () , }) , stmt : Box :: new (rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (result) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 10 , lifetime : stringify ! (b1111101111) . into () , })) , })) , } , rhdl :: vlog :: CaseLine { item : rhdl :: vlog :: CaseItem :: Literal (rhdl :: vlog :: LitVerilog { width : 16 , lifetime : stringify ! (d6) . into () , }) , stmt : Box :: new (rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (result) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 10 , lifetime : stringify ! (b1111110111) . into () , })) , })) , } , rhdl :: vlog :: CaseLine { item : rhdl :: vlog :: CaseItem :: Literal (rhdl :: vlog :: LitVerilog { width : 16 , lifetime : stringify ! (d7) . into () , }) , stmt : Box :: new (rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (result) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 10 , lifetime : stringify ! (b1111111011) . into () , })) , })) , } , rhdl :: vlog :: CaseLine { item : rhdl :: vlog :: CaseItem :: Literal (rhdl :: vlog :: LitVerilog { width : 16 , lifetime : stringify ! (d8) . into () , }) , stmt : Box :: new (rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (result) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 10 , lifetime : stringify ! (b1111111101) . into () , })) , })) , } , rhdl :: vlog :: CaseLine { item : rhdl :: vlog :: CaseItem :: Literal (rhdl :: vlog :: LitVerilog { width : 16 , lifetime : stringify ! (d9) . into () , }) , stmt : Box :: new (rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (result) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 10 , lifetime : stringify ! (b1111111110) . into () , })) , })) , } , rhdl :: vlog :: CaseLine { item : rhdl :: vlog :: CaseItem :: Wild , stmt : Box :: new (rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (result) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 10 , lifetime : stringify ! (bx) . into () , })) , })) , }] , })"
    ];
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
default result = 10'bx;
endcase
        ",
    )?;
    expect.assert_eq(&synth.to_string());
    Ok(())
}

#[test]
fn test_quote_parse_module_def() -> miette::Result<()> {
    let expect = expect_test::expect![
        "rhdl :: vlog :: ModuleDef { name : stringify ! (foo) . into () , args : vec ! [rhdl :: vlog :: Port { direction : rhdl :: vlog :: Direction :: Input , decl : rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Wire , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 2) , name : stringify ! (clock_reset) . into () } , } , rhdl :: vlog :: Port { direction : rhdl :: vlog :: Direction :: Input , decl : rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Wire , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 7) , name : stringify ! (i) . into () } , } , rhdl :: vlog :: Port { direction : rhdl :: vlog :: Direction :: Output , decl : rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Wire , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 7) , name : stringify ! (o) . into () } , } ,] , items : vec ! [] , }"
    ];
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
fn test_quote_parse_dff_definition() -> miette::Result<()> {
    let expect = expect_test::expect![
        "rhdl :: vlog :: ModuleDef { name : stringify ! (foo) . into () , args : vec ! [rhdl :: vlog :: Port { direction : rhdl :: vlog :: Direction :: Input , decl : rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Wire , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 2) , name : stringify ! (clock_reset) . into () } , } , rhdl :: vlog :: Port { direction : rhdl :: vlog :: Direction :: Input , decl : rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Wire , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 7) , name : stringify ! (i) . into () } , } , rhdl :: vlog :: Port { direction : rhdl :: vlog :: Direction :: Output , decl : rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Wire , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 7) , name : stringify ! (o) . into () } , } ,] , items : vec ! [rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Wire , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (clock) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Wire , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (reset) . into () }) , rhdl :: vlog :: Item :: Statement (rhdl :: vlog :: Stmt :: ContinuousAssign (rhdl :: vlog :: Assign { target : stringify ! (clock) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (clock_reset) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (0)) , lsb : . map (Box :: new) , } , })) , })) , rhdl :: vlog :: Item :: Statement (rhdl :: vlog :: Stmt :: ContinuousAssign (rhdl :: vlog :: Assign { target : stringify ! (wire) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (clock_reset) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (1)) , lsb : . map (Box :: new) , } , })) , })) , rhdl :: vlog :: Item :: Statement (rhdl :: vlog :: Stmt :: Always (rhdl :: vlog :: Always { sensitivity : vec ! [rhdl :: vlog :: Sensitivity :: PosEdge (stringify ! (clock) . into ())] , body : Box :: new (rhdl :: vlog :: Stmt :: Block (rhdl :: vlog :: Stmt :: Block (vec ! [rhdl :: vlog :: Stmt :: If (rhdl :: vlog :: If { condition : Box :: new (rhdl :: vlog :: Expr :: Ident (reset)) , true_stmt : Box :: new (rhdl :: vlog :: Stmt :: Block (rhdl :: vlog :: Stmt :: Block (vec ! [rhdl :: vlog :: Stmt :: NonblockAssign (rhdl :: vlog :: Assign { target : stringify ! (o) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 8 , lifetime : stringify ! (b0) . into () , })) , }) ,]))) , else_branch : rhdl :: vlog :: Stmt :: Block (rhdl :: vlog :: Stmt :: Block (vec ! [rhdl :: vlog :: Stmt :: NonblockAssign (rhdl :: vlog :: Assign { target : stringify ! (o) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Ident (i)) , }) ,])) . map (Box :: new) , }) ,]))) , })) ,] , }"
    ];
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
    let expect = expect_test::expect![
        "rhdl :: vlog :: ModuleDef { name : stringify ! (dut) . into () , args : vec ! [rhdl :: vlog :: Port { direction : rhdl :: vlog :: Direction :: Input , decl : rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Wire , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 7) , name : stringify ! (arg_0) . into () } , } , rhdl :: vlog :: Port { direction : rhdl :: vlog :: Direction :: Output , decl : rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 7) , name : stringify ! (out) . into () } , } ,] , items : vec ! [rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r0) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r1) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r2) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r3) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r4) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r5) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r6) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r7) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r8) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r9) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r10) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r11) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r12) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r13) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r14) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r15) . into () }) , rhdl :: vlog :: Item :: Statement (rhdl :: vlog :: Stmt :: Always (rhdl :: vlog :: Always { sensitivity : vec ! [rhdl :: vlog :: Sensitivity :: Star] , body : Box :: new (rhdl :: vlog :: Stmt :: Block (rhdl :: vlog :: Stmt :: Block (vec ! [rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r0) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_0) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (0)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r1) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_0) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (1)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r2) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_0) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (2)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r3) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_0) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (3)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r4) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_0) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (4)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r5) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_0) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (5)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r6) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_0) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (6)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r7) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_0) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (7)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: ConcatAssign (rhdl :: vlog :: ConcatAssign { target : vec ! [rhdl :: vlog :: Expr :: Ident (r15) , rhdl :: vlog :: Expr :: Ident (r14) , rhdl :: vlog :: Expr :: Ident (r13) , rhdl :: vlog :: Expr :: Ident (r12) , rhdl :: vlog :: Expr :: Ident (r11) , rhdl :: vlog :: Expr :: Ident (r10) , rhdl :: vlog :: Expr :: Ident (r9) , rhdl :: vlog :: Expr :: Ident (r8) ,] rhs : Box :: new (rhdl :: vlog :: Expr :: Binary (rhdl :: vlog :: ExprBinary { lhs : rhdl :: vlog :: Expr :: Concat (vec ! [rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 1 , lifetime : stringify ! (b0) . into () , }) , rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 1 , lifetime : stringify ! (b0) . into () , }) , rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 1 , lifetime : stringify ! (b0) . into () , }) , rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 1 , lifetime : stringify ! (b0) . into () , }) , rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 1 , lifetime : stringify ! (b0) . into () , }) , rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 1 , lifetime : stringify ! (b1) . into () , }) , rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 1 , lifetime : stringify ! (b1) . into () , }) , rhdl :: vlog :: Expr :: Constant (rhdl :: vlog :: LitVerilog { width : 1 , lifetime : stringify ! (b0) . into () , }) ,]) , op : rhdl :: vlog :: BinaryOp :: Plus , rhs : rhdl :: vlog :: Expr :: Concat (vec ! [rhdl :: vlog :: Expr :: Ident (r7) , rhdl :: vlog :: Expr :: Ident (r6) , rhdl :: vlog :: Expr :: Ident (r5) , rhdl :: vlog :: Expr :: Ident (r4) , rhdl :: vlog :: Expr :: Ident (r3) , rhdl :: vlog :: Expr :: Ident (r2) , rhdl :: vlog :: Expr :: Ident (r1) , rhdl :: vlog :: Expr :: Ident (r0) ,]) })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (out) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Concat (vec ! [rhdl :: vlog :: Expr :: Ident (r15) , rhdl :: vlog :: Expr :: Ident (r14) , rhdl :: vlog :: Expr :: Ident (r13) , rhdl :: vlog :: Expr :: Ident (r12) , rhdl :: vlog :: Expr :: Ident (r11) , rhdl :: vlog :: Expr :: Ident (r10) , rhdl :: vlog :: Expr :: Ident (r9) , rhdl :: vlog :: Expr :: Ident (r8) ,])) , }) ,]))) , })) ,] , }"
    ];
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
    let expect = expect_test::expect![
        "rhdl :: vlog :: ModuleDef { name : stringify ! (dut) . into () , args : vec ! [rhdl :: vlog :: Port { direction : rhdl :: vlog :: Direction :: Input , decl : rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Wire , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 7) , name : stringify ! (arg_0) . into () } , } , rhdl :: vlog :: Port { direction : rhdl :: vlog :: Direction :: Input , decl : rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Wire , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 7) , name : stringify ! (arg_1) . into () } , } , rhdl :: vlog :: Port { direction : rhdl :: vlog :: Direction :: Output , decl : rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (out) . into () } , } ,] , items : vec ! [rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r0) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r1) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r2) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r3) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r4) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r5) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r6) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r7) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r8) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r9) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r10) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r11) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r12) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r13) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r14) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r15) . into () }) , rhdl :: vlog :: Item :: Declaration (rhdl :: vlog :: Declaration { kind : rhdl :: vlog :: HDLKind :: Reg , signed_width : rhdl :: vlog :: SignedWidth :: Unsigned (0 ..= 0) , name : stringify ! (r16) . into () }) , rhdl :: vlog :: Item :: Statement (rhdl :: vlog :: Stmt :: Always (rhdl :: vlog :: Always { sensitivity : vec ! [rhdl :: vlog :: Sensitivity :: Star] , body : Box :: new (rhdl :: vlog :: Stmt :: Block (rhdl :: vlog :: Stmt :: Block (vec ! [rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r0) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_0) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (0)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r1) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_0) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (1)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r2) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_0) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (2)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r3) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_0) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (3)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r4) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_0) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (4)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r5) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_0) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (5)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r6) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_0) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (6)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r7) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_0) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (7)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r8) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_1) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (0)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r9) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_1) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (1)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r10) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_1) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (2)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r11) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_1) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (3)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r12) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_1) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (4)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r13) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_1) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (5)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r14) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_1) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (6)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (r15) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Index (rhdl :: vlog :: ExprIndex { target : stringify ! (arg_1) . into () , address : rhdl :: vlog :: ExprIndexAddress { msb : Box :: new (rhdl :: vlog :: Expr :: Literal (7)) , lsb : . map (Box :: new) , } , })) , }) , rhdl :: vlog :: Stmt :: ConcatAssign (rhdl :: vlog :: ConcatAssign { target : vec ! [rhdl :: vlog :: Expr :: Ident (r16) ,] rhs : Box :: new (rhdl :: vlog :: Expr :: Binary (rhdl :: vlog :: ExprBinary { lhs : rhdl :: vlog :: Expr :: Function (rhdl :: vlog :: ExprFunction { name : _signed , args : vec ! [rhdl :: vlog :: Expr :: Concat (vec ! [rhdl :: vlog :: Expr :: Ident (r7) , rhdl :: vlog :: Expr :: Ident (r6) , rhdl :: vlog :: Expr :: Ident (r5) , rhdl :: vlog :: Expr :: Ident (r4) , rhdl :: vlog :: Expr :: Ident (r3) , rhdl :: vlog :: Expr :: Ident (r2) , rhdl :: vlog :: Expr :: Ident (r1) , rhdl :: vlog :: Expr :: Ident (r0) ,]) ,] }) , op : rhdl :: vlog :: BinaryOp :: Ge , rhs : rhdl :: vlog :: Expr :: Function (rhdl :: vlog :: ExprFunction { name : _signed , args : vec ! [rhdl :: vlog :: Expr :: Concat (vec ! [rhdl :: vlog :: Expr :: Ident (r15) , rhdl :: vlog :: Expr :: Ident (r14) , rhdl :: vlog :: Expr :: Ident (r13) , rhdl :: vlog :: Expr :: Ident (r12) , rhdl :: vlog :: Expr :: Ident (r11) , rhdl :: vlog :: Expr :: Ident (r10) , rhdl :: vlog :: Expr :: Ident (r9) , rhdl :: vlog :: Expr :: Ident (r8) ,]) ,] }) })) , }) , rhdl :: vlog :: Stmt :: Assign (rhdl :: vlog :: Assign { target : stringify ! (out) . into () , rhs : Box :: new (rhdl :: vlog :: Expr :: Concat (vec ! [rhdl :: vlog :: Expr :: Ident (r16) ,])) , }) ,]))) , })) ,] , }"
    ];
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
