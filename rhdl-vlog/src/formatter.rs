#[derive(Default)]
pub struct Formatter {
    indent_level: usize,
    contents: String,
    start_of_line: bool,
}

const TAB: &str = "   ";

impl Formatter {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            contents: String::new(),
            start_of_line: true,
        }
    }

    pub fn push(&mut self) {
        self.indent_level += 1;
    }

    pub fn pop(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(1);
    }

    pub fn write(&mut self, text: &str) {
        if self.start_of_line {
            self.contents.push_str(&TAB.repeat(self.indent_level));
            self.start_of_line = false;
        }
        self.contents.push_str(&format!("{}", text));
        self.start_of_line = text.ends_with('\n');
    }

    pub fn newline(&mut self) {
        self.contents.push_str("\n");
        self.start_of_line = true;
    }

    pub fn finish(self) -> String {
        self.contents
    }

    pub fn scoped(&mut self, f: impl FnOnce(&mut Self)) {
        self.push();
        f(self);
        self.pop();
    }

    pub fn parenthesized(&mut self, f: impl FnOnce(&mut Self)) {
        self.write("(");
        f(self);
        self.write(")");
    }

    pub fn braced(&mut self, f: impl FnOnce(&mut Self)) {
        self.write("{");
        f(self);
        self.write("}");
    }

    pub fn bracketed(&mut self, f: impl FnOnce(&mut Self)) {
        self.write("[");
        f(self);
        self.write("]");
    }

    pub fn comma_separated<T: Pretty>(&mut self, items: impl IntoIterator<Item = T>) {
        let mut iter = items.into_iter();
        if let Some(first) = iter.next() {
            first.pretty_print(self);
            for item in iter {
                self.write(", ");
                item.pretty_print(self);
            }
        }
    }

    pub fn lines<T: Pretty>(&mut self, items: impl IntoIterator<Item = T>) {
        let iter = items.into_iter();
        for item in iter {
            item.pretty_print(self);
            if !(self.contents.ends_with("end")
                || self.contents.ends_with("endcase")
                || self.contents.ends_with("endfunction"))
            {
                self.write(";");
            }
            self.newline();
        }
    }
}

pub trait Pretty {
    fn pretty_print(&self, formatter: &mut Formatter);
    fn pretty(&self) -> String {
        let mut formatter = Formatter::new();
        self.pretty_print(&mut formatter);
        formatter.finish()
    }
}

impl<T> Pretty for &T
where
    T: Pretty,
{
    fn pretty_print(&self, formatter: &mut Formatter) {
        (*self).pretty_print(formatter);
    }
}

impl<T> Pretty for Box<T>
where
    T: Pretty,
{
    fn pretty_print(&self, formatter: &mut Formatter) {
        (**self).pretty_print(formatter);
    }
}

impl<T> Pretty for Option<T>
where
    T: Pretty,
{
    fn pretty_print(&self, formatter: &mut Formatter) {
        if let Some(value) = self {
            value.pretty_print(formatter);
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::cst;

    use super::*;

    #[test]
    fn test_pretty_printing() {
        let expect = expect_test::expect_file!["cst/expect/pretty_dff_definition.expect"];
        let synth = syn::parse_str::<cst::ModuleList>(
            "
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
           {a, b} = c;
           localparam cost = 42;
           localparam bar = 16'd16;
           localparam pie = \"apple\";
           obj obj(.clk(clock), .reset(reset), .i(i), .o(o));
           initial begin
               o = 8'b0;
               # 10;
               o = add(8'b0, 8'b1) + !c;
               o = (a > b) ? a : b[o -: 4];
               $display(\"o = 2\");
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
",
        )
        .unwrap();
        let pretty = synth.pretty();
        expect.assert_eq(&pretty);
    }
}
