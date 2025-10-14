use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Serialize;
use syn::{Result, parse::Parse};

fn parse_and_encode<T: Parse + Serialize>(
    typename: &str,
    input: TokenStream,
) -> Result<TokenStream> {
    let parsed = syn::parse::<T>(input.into())?;
    let json_data = serde_json::to_string(&parsed).map_err(|err| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("Serialization error: {}", err),
        )
    })?;
    let typename = format_ident!("{}", typename);
    Ok(quote! {
        {
            const AST_JSON: &'static str = #json_data;
            serde_json::from_str::<rhdl_vlog::cst::#typename>(AST_JSON).expect("Failed to deserialize AST")
        }
    })
}

pub fn modules(input: TokenStream) -> Result<TokenStream> {
    parse_and_encode::<rhdl_vlog::ModuleList>("ModuleList", input)
}

pub fn module(input: TokenStream) -> Result<TokenStream> {
    parse_and_encode::<rhdl_vlog::ModuleDef>("ModuleDef", input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect_file;

    #[test]
    fn test_modules_proc_macro() {
        let kitchen_sink = quote! {
        module foo(input wire[2:0] clock_reset, input wire[7:0] i, output wire signed [7:0] o, inout wire baz);
           wire [0:0] clock;
           wire [0:0] reset;
           reg [3:0] a, b, c;
           wire foo;
           assign clock = clock_reset[0];
           assign wire = clock_reset[1];
           assign o = {i, i};
           assign o = {3 {i}};
           a[b +: 1] = clock;
           a[1:0] = reset;
           {a, b} = c;
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
        };
        let sink = syn::parse2::<rhdl_vlog::ModuleList>(kitchen_sink).unwrap();
        let expect = expect_file!["expect/vlog_modules.expect"];
        expect.assert_eq(&sink.to_string());
    }
}
