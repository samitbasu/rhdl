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
        self.contents.push_str(&format!("{}\n", text));
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

    pub fn comma_separated<T: Pretty>(&mut self, items: impl IntoIterator<Item = T>) {
        let mut iter = items.into_iter();
        if let Some(first) = iter.next() {
            first.pretty_print(self);
            for item in iter {
                self.write(",");
                item.pretty_print(self);
            }
        }
    }

    pub fn semi_line_separated<T: Pretty>(&mut self, items: impl IntoIterator<Item = T>) {
        let iter = items.into_iter();
        for item in iter {
            item.pretty_print(self);
            self.write(";");
            self.newline();
        }
    }
}

pub trait Pretty {
    fn pretty_print(&self, formatter: &mut Formatter);
}

impl<T> Pretty for &T
where
    T: Pretty,
{
    fn pretty_print(&self, formatter: &mut Formatter) {
        (*self).pretty_print(formatter);
    }
}

#[cfg(test)]
mod tests {

    use crate::cst;

    use super::*;

    #[test]
    fn test_formatter() {
        let mut formatter = Formatter::new();
        formatter.push();
        formatter.write("Hello");
        formatter.pop();
        formatter.write("World");
        assert_eq!(formatter.contents, "   Hello\nWorld\n");
    }

    #[test]
    fn test_pretty_printing() {
        let expect = expect_test::expect_file!["../expect/pretty_dff_definition.expect"];
        let synth = syn::parse_str::<cst::ModuleList>(
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
        )
        .unwrap();
        let mut formatter = Formatter::new();
        synth.pretty_print(&mut formatter);
        expect.assert_eq(&formatter.finish());
    }
}
