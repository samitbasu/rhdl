module testbench();
   reg [1:0] i;
   wire [1:0] o;
   reg [1:0] rust_out;
   uut t(.i(i), .o(o));
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
module uut(input wire [1:0] i, output wire [1:0] o);
   wire [5:0] od;
   wire [3:0] d;
   wire [1:0] q;
   assign o = od[1:0];
   uut_xor c0(.i(d[1:0]), .o(q[0:0]));
   uut_and c1(.i(d[3:2]), .o(q[1:1]));
   assign d = od[5:2];
   assign od = kernel_half_adder_kernel(i, q);
   function [5:0] kernel_half_adder_kernel(input reg [1:0] arg_0, input reg [1:0] arg_1);
         reg [1:0] r0;
         reg [3:0] r1;
         reg [3:0] r2;
         reg [0:0] r3;
         reg [1:0] r4;
         reg [0:0] r5;
         reg [1:0] r6;
         reg [1:0] r7;
         reg [5:0] r8;
         localparam l0 = 4'b0000;
         localparam l1 = 2'b00;
         begin
            r0 = arg_0;
            r4 = arg_1;
            r1 = l0;
            r1[1:0] = r0;
            r2 = r1;
            r2[3:2] = r0;
            r3 = r4[0:0];
            r5 = r4[1:1];
            r6 = l1;
            r6[0:0] = r3;
            r7 = r6;
            r7[1:1] = r5;
            r8 = {r2, r7};
            kernel_half_adder_kernel = r8;
         end
   endfunction
endmodule
module uut_xor(input wire [1:0] i, output wire [0:0] o);
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
module uut_and(input wire [1:0] i, output wire [0:0] o);
   wire [0:0] od;
   assign o = od[0:0];
   assign od = kernel_and_gate(i);
   function [0:0] kernel_and_gate(input reg [1:0] arg_0);
         reg [1:0] r0;
         reg [0:0] r1;
         reg [0:0] r2;
         reg [0:0] r3;
         begin
            r0 = arg_0;
            r1 = r0[0:0];
            r2 = r0[1:1];
            r3 = r1 & r2;
            kernel_and_gate = r3;
         end
   endfunction
endmodule
