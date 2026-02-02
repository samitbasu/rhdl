module testbench();
   reg [1:0] i;
   wire [0:0] o;
   reg [0:0] rust_out;
   dut t(.arg_0(i), .out(o));
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
module dut(input wire [1:0] arg_0, output reg [0:0] out);
   reg  r4;
   reg  r5;
   reg  r7;
   always @(*) begin
      r4 = arg_0[0];
      r5 = arg_0[1];
      r7 = r4 ^ r5;
      out = {r7};
   end
endmodule
