module testbench();
   reg [1:0] clock_reset;
   reg [0:0] i;
   wire [2:0] o;
   reg [2:0] rust_out;
   uut t(.clock_reset(clock_reset), .i(i), .o(o));
   initial begin
      #0;
      clock_reset = 2'b10;
      i = 1'b0;
      rust_out = 3'b000;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b Test 1 at time 0", rust_out, o);
         $finish;
      end
      #49;
      clock_reset = 2'b11;
      i = 1'b0;
      rust_out = 3'b000;
      #1;
      clock_reset = 2'b01;
      i = 1'b1;
      rust_out = 3'b000;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b Test 3 at time 51", rust_out, o);
         $finish;
      end
      #48;
      clock_reset = 2'b00;
      i = 1'b1;
      rust_out = 3'b000;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b Test 4 at time 100", rust_out, o);
         $finish;
      end
      #49;
      clock_reset = 2'b01;
      i = 1'b1;
      rust_out = 3'b001;
      #1;
      clock_reset = 2'b01;
      i = 1'b1;
      rust_out = 3'b001;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b Test 6 at time 151", rust_out, o);
         $finish;
      end
      #48;
      clock_reset = 2'b00;
      i = 1'b1;
      rust_out = 3'b001;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b Test 7 at time 200", rust_out, o);
         $finish;
      end
      #49;
      clock_reset = 2'b01;
      i = 1'b1;
      rust_out = 3'b010;
      #1;
      clock_reset = 2'b01;
      i = 1'b0;
      rust_out = 3'b010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b Test 9 at time 251", rust_out, o);
         $finish;
      end
      #48;
      clock_reset = 2'b00;
      i = 1'b0;
      rust_out = 3'b010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b Test 10 at time 300", rust_out, o);
         $finish;
      end
      #49;
      clock_reset = 2'b01;
      i = 1'b0;
      rust_out = 3'b010;
      #1;
      clock_reset = 2'b01;
      i = 1'b0;
      rust_out = 3'b010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b Test 12 at time 351", rust_out, o);
         $finish;
      end
      #48;
      clock_reset = 2'b00;
      i = 1'b0;
      rust_out = 3'b010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b Test 13 at time 400", rust_out, o);
         $finish;
      end
      #49;
      clock_reset = 2'b01;
      i = 1'b0;
      rust_out = 3'b010;
      #1;
      clock_reset = 2'b01;
      i = 1'b1;
      rust_out = 3'b010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b Test 15 at time 451", rust_out, o);
         $finish;
      end
      #48;
      clock_reset = 2'b00;
      i = 1'b1;
      rust_out = 3'b010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b Test 16 at time 500", rust_out, o);
         $finish;
      end
      #49;
      clock_reset = 2'b01;
      i = 1'b1;
      rust_out = 3'b011;
      #1;
      clock_reset = 2'b01;
      i = 1'b1;
      rust_out = 3'b011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b Test 18 at time 551", rust_out, o);
         $finish;
      end
      #48;
      clock_reset = 2'b00;
      i = 1'b1;
      rust_out = 3'b011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b Test 19 at time 600", rust_out, o);
         $finish;
      end
      #49;
      clock_reset = 2'b01;
      i = 1'b1;
      rust_out = 3'b100;
      #1;
      clock_reset = 2'b01;
      i = 1'b1;
      rust_out = 3'b100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b Test 21 at time 651", rust_out, o);
         $finish;
      end
      #48;
      clock_reset = 2'b00;
      i = 1'b1;
      rust_out = 3'b100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b Test 22 at time 700", rust_out, o);
         $finish;
      end
      #49;
      clock_reset = 2'b00;
      i = 1'b1;
      rust_out = 3'b100;
      $display("TESTBENCH OK");
      $finish;
   end
endmodule
module uut(input wire [1:0] clock_reset, input wire [0:0] i, output wire [2:0] o);
   wire [5:0] od;
   wire [2:0] d;
   wire [2:0] q;
   assign o = od[2:0];
   uut_count c0(.clock_reset(clock_reset), .i(d[2:0]), .o(q[2:0]));
   assign d = od[5:3];
   assign od = kernel_counter(clock_reset, i, q);
   function [5:0] kernel_counter(input reg [1:0] arg_0, input reg [0:0] arg_1, input reg [2:0] arg_2);
         reg [2:0] r0;
         reg [2:0] r1;
         reg [2:0] r2;
         reg [0:0] r3;
         reg [0:0] r4;
         reg [1:0] r5;
         reg [0:0] r6;
         reg [2:0] r7;
         reg [2:0] r8;
         reg [5:0] r9;
         localparam l0 = 3'b001;
         localparam l1 = 3'b000;
         localparam l2 = 3'b000;
         begin
            r5 = arg_0;
            r3 = arg_1;
            r0 = arg_2;
            r1 = r0 + l0;
            r2 = r3 ? r1 : r0;
            r4 = r5[1:1];
            r6 = |r4;
            r7 = r6 ? l1 : r2;
            r8 = l2;
            r8[2:0] = r7;
            r9 = {r8, r0};
            kernel_counter = r9;
         end
   endfunction
endmodule
module uut_count(input wire [1:0] clock_reset, input wire [2:0] i, output reg [2:0] o);
   wire  clock;
   wire  reset;
   assign clock = clock_reset[0];
   assign reset = clock_reset[1];
   initial begin
      o = 3'b000;
   end
   always @(posedge clock) begin
      if (reset) begin
         o <= 3'b000;
      end else begin
         o <= i;
      end
   end
endmodule
