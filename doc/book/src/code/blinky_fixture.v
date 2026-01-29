module top(input wire [0:0] sysclk_p, input wire [0:0] sysclk_n, output wire [7:0] led);
   wire [1:0] inner_input;
   wire [7:0] inner_output;
   IBUFDS #(.DIFF_TERM("FALSE"), .IBUF_LOW_PWR("TRUE"), .IOSTANDARD("LVDS_25")) ibufds_sysclk(.O(inner_input[0:0]), .I(sysclk_p), .IB(sysclk_n));
   assign inner_input[1:1] = 1'b0;
   wire [7:0] _drive_led;
   assign _drive_led = inner_output[7:0];
   assign led[0] = (_drive_led[0] == 1'b1) ? (1'b0) : (1'bz);
   assign led[1] = (_drive_led[1] == 1'b1) ? (1'b0) : (1'bz);
   assign led[2] = (_drive_led[2] == 1'b1) ? (1'b0) : (1'bz);
   assign led[3] = (_drive_led[3] == 1'b1) ? (1'b0) : (1'bz);
   assign led[4] = (_drive_led[4] == 1'b1) ? (1'b0) : (1'bz);
   assign led[5] = (_drive_led[5] == 1'b1) ? (1'b0) : (1'bz);
   assign led[6] = (_drive_led[6] == 1'b1) ? (1'b0) : (1'bz);
   assign led[7] = (_drive_led[7] == 1'b1) ? (1'b0) : (1'bz);
   inner inner_inst(.i(inner_input), .o(inner_output));
endmodule
module inner(input wire [1:0] i, output wire [7:0] o);
   inner_inner c(.clock_reset(i[1:0]), .o(o));
endmodule
module inner_inner(input wire [1:0] clock_reset, output wire [7:0] o);
   wire [8:0] od;
   wire [0:0] d;
   wire [31:0] q;
   assign o = od[7:0];
   inner_inner_counter c0(.clock_reset(clock_reset), .i(d[0:0]), .o(q[31:0]));
   assign d = od[8:8];
   assign od = kernel_blinker(clock_reset, q);
   function [8:0] kernel_blinker(input reg [1:0] arg_0, input reg [31:0] arg_2);
         reg [31:0] r0;
         reg [31:0] r1;
         reg [31:0] r2;
         reg [0:0] r3;
         reg [7:0] r4;
         reg [8:0] r5;
         reg [1:0] r6;
         reg [59:0] r7;
         localparam l0 = 32'b00000000000000000000000000000001;
         localparam l1 = 8'b10101010;
         localparam l2 = 8'b01010101;
         localparam l3 = 1'b1;
         begin
            r6 = arg_0;
            r0 = arg_2;
            r7 = {{28{1'b0}}, r0};
            r1 = r7[59:28];
            r2 = r1 & l0;
            r3 = |r2;
            r4 = r3 ? l1 : l2;
            r5 = {l3, r4};
            kernel_blinker = r5;
         end
   endfunction
endmodule
module inner_inner_counter(input wire [1:0] clock_reset, input wire [0:0] i, output wire [31:0] o);
   wire [63:0] od;
   wire [31:0] d;
   wire [31:0] q;
   assign o = od[31:0];
   inner_inner_counter_count c0(.clock_reset(clock_reset), .i(d[31:0]), .o(q[31:0]));
   assign d = od[63:32];
   assign od = kernel_counter(clock_reset, i, q);
   function [63:0] kernel_counter(input reg [1:0] arg_0, input reg [0:0] arg_1, input reg [31:0] arg_2);
         reg [31:0] r0;
         reg [31:0] r1;
         reg [31:0] r2;
         reg [0:0] r3;
         reg [0:0] r4;
         reg [1:0] r5;
         reg [0:0] r6;
         reg [31:0] r7;
         reg [31:0] r8;
         reg [63:0] r9;
         localparam l0 = 32'b00000000000000000000000000000001;
         localparam l1 = 32'b00000000000000000000000000000000;
         localparam l2 = 32'b00000000000000000000000000000000;
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
            r8[31:0] = r7;
            r9 = {r8, r0};
            kernel_counter = r9;
         end
   endfunction
endmodule
module inner_inner_counter_count(input wire [1:0] clock_reset, input wire [31:0] i, output reg [31:0] o);
   wire  clock;
   wire  reset;
   assign clock = clock_reset[0];
   assign reset = clock_reset[1];
   initial begin
      o = 32'b00000000000000000000000000000000;
   end
   always @(posedge clock) begin
      if (reset) begin
         o <= 32'b00000000000000000000000000000000;
      end else begin
         o <= i;
      end
   end
endmodule
