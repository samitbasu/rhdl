module testbench();
   reg [1:0] i;
   wire [0:0] o;
   reg [0:0] rust_out;
   uut t(.i(i), .o(o));
   initial begin
      #0;
      i = 2'b00;
      rust_out = 1'b0;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 1 at time 100", rust_out, o);
         $finish;
      end
      #99;
      i = 2'b10;
      rust_out = 1'b1;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 2 at time 200", rust_out, o);
         $finish;
      end
      #99;
      i = 2'b01;
      rust_out = 1'b1;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 3 at time 300", rust_out, o);
         $finish;
      end
      #99;
      i = 2'b11;
      rust_out = 1'b0;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 4 at time 400", rust_out, o);
         $finish;
      end
      #99;
      i = 2'b00;
      rust_out = 1'b0;
      $display("TESTBENCH OK");
      $finish;
   end
endmodule
module uut(input wire [1:0] i, output wire [0:0] o);
   wire [0:0] od;
   assign o = od[0:0];
   assign od = kernel_xor_gate(i);
   function [0:0] kernel_xor_gate(input reg [1:0] arg_0);
         reg [1:0] r0;
         reg [0:0] r1;
         reg [0:0] r2;
         reg [0:0] r3;
         begin
            r0 = arg_0;
            r1 = r0[0:0];
            r2 = r0[1:1];
            r3 = r1 ^ r2;
            kernel_xor_gate = r3;
         end
   endfunction
endmodule
