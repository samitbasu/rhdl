module top(input wire [1:0] clock_reset, input wire [0:0] i, output wire [3:0] o);
   wire [7:0] od;
   wire [3:0] d;
   wire [3:0] q;
   assign o = od[3:0];
   top_count c0(.clock_reset(clock_reset), .i(d[3:0]), .o(q[3:0]));
   assign d = od[7:4];
   assign od = kernel_counter(clock_reset, i, q);
   function [7:0] kernel_counter(input reg [1:0] arg_0, input reg [0:0] arg_1, input reg [3:0] arg_2);
         reg [3:0] r0;
         reg [3:0] r1;
         reg [3:0] r2;
         reg [0:0] r3;
         reg [0:0] r4;
         reg [1:0] r5;
         reg [0:0] r6;
         reg [3:0] r7;
         reg [3:0] r8;
         reg [7:0] r9;
         localparam l0 = 4'b0001;
         localparam l1 = 4'b0000;
         localparam l2 = 4'b0000;
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
            r8[3:0] = r7;
            r9 = {r8, r0};
            kernel_counter = r9;
         end
   endfunction
endmodule
module top_count(input wire [1:0] clock_reset, input wire [3:0] i, output reg [3:0] o);
   wire  clock;
   wire  reset;
   assign clock = clock_reset[0];
   assign reset = clock_reset[1];
   initial begin
      o = 4'b0000;
   end
   always @(posedge clock) begin
      if (reset) begin
         o <= 4'b0000;
      end else begin
         o <= i;
      end
   end
endmodule
