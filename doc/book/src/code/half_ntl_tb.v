module testbench();
   reg [1:0] i;
   wire [1:0] o;
   reg [1:0] rust_out;
   dut t(.arg_0(i), .out(o));
   initial begin
      #0;
      i = 2'b00;
      rust_out = 2'b00;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 1 at time 100", rust_out, o);
         $finish;
      end
      #99;
      i = 2'b10;
      rust_out = 2'b01;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 2 at time 200", rust_out, o);
         $finish;
      end
      #99;
      i = 2'b01;
      rust_out = 2'b01;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 3 at time 300", rust_out, o);
         $finish;
      end
      #99;
      i = 2'b11;
      rust_out = 2'b10;
      $display("TESTBENCH OK");
      $finish;
   end
endmodule
module dut(input wire [1:0] arg_0, output reg [1:0] out);
   reg  r8;
   reg  r9;
   reg  r15;
   reg  r16;
   always @(*) begin
      r8 = arg_0[0];
      r9 = arg_0[1];
      r15 = r8 ^ r9;
      r16 = r8 & r9;
      out = {r16, r15};
   end
endmodule
