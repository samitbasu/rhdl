module top(input wire [1:0] clock_reset, input wire [3:0] i, output wire [1:0] o);
   wire [11:0] od;
   wire [9:0] d;
   wire [7:0] q;
   assign o = od[1:0];
   top_master c0(.clock_reset(clock_reset), .i(d[9:5]), .o(q[7:3]));
   top_sub c1(.clock_reset(clock_reset), .i(d[4:0]), .o(q[2:0]));
   assign d = od[11:2];
   assign od = kernel_kernel(clock_reset, i, q);
   function [11:0] kernel_kernel(input reg [1:0] arg_0, input reg [3:0] arg_1, input reg [7:0] arg_2);
         reg [4:0] or0;
         reg [7:0] or1;
         reg [3:0] or2;
         reg [0:0] or3;
         reg [2:0] or4;
         reg [1:0] or5;
         reg [0:0] or6;
         reg [2:0] or7;
         reg [3:0] or8;
         reg [2:0] or9;
         // d
         reg [9:0] or10;
         // d
         reg [9:0] or11;
         reg [4:0] or12;
         reg [0:0] or13;
         // d
         reg [9:0] or14;
         reg [2:0] or15;
         reg [0:0] or16;
         // d
         reg [9:0] or17;
         // d
         reg [9:0] or18;
         reg [3:0] or19;
         reg [2:0] or20;
         reg [1:0] or21;
         reg [0:0] or22;
         reg [1:0] or23;
         reg [0:0] or24;
         // o
         reg [1:0] or25;
         reg [11:0] or26;
         localparam ol0 = 1'b1;
         localparam ol1 = 10'bXXXXXX0000;
         localparam ol2 = 1'b1;
         localparam ol3 = 2'b00;
         begin
            or23 = arg_0;
            or19 = arg_1;
            or1 = arg_2;
            or0 = or1[7:3];
            or2 = or0[4:1];
            or3 = or2[3:3];
            or4 = or2[2:0];
            or5 = or4[1:0];
            or6 = or4[2:2];
            or7 = {or5, or6};
            or9 = or7[2:0];
            or8 = {ol0, or9};
            or10 = ol1;
            or10[3:0] = or8;
            case (or3)
               1'b1 : or11 = or10;
               default : or11 = ol1;
            endcase
            or12 = or1[7:3];
            or13 = or12[0:0];
            or14 = or11;
            or14[4:4] = or13;
            or15 = or1[2:0];
            or16 = or15[0:0];
            or17 = or14;
            or17[9:9] = or16;
            or18 = or17;
            or18[8:5] = or19;
            or20 = or1[2:0];
            or21 = or20[2:1];
            or22 = or23[1:1];
            or24 = |or22;
            or25 = or24 ? ol3 : or21;
            or26 = {or18, or25};
            kernel_kernel = or26;
         end
   endfunction
endmodule
module top_master(input wire [1:0] clock_reset, input wire [4:0] i, output wire [4:0] o);
   wire [4:0] od;
   ;
   ;
   assign o = od[4:0];
   assign od = kernel_master_kernel(clock_reset, i);
   function [4:0] kernel_master_kernel(input reg [1:0] arg_0, input reg [4:0] arg_1);
         reg [3:0] or0;
         reg [4:0] or1;
         reg [0:0] or2;
         reg [2:0] or3;
         reg [0:0] or4;
         reg [1:0] or5;
         reg [2:0] or6;
         reg [3:0] or7;
         reg [2:0] or8;
         // o
         reg [4:0] or9;
         // o
         reg [4:0] or10;
         reg [0:0] or11;
         reg [1:0] or12;
         reg [0:0] or13;
         // o
         reg [4:0] or14;
         // o
         reg [4:0] or15;
         // o
         reg [4:0] or16;
         localparam ol0 = 1'b1;
         localparam ol1 = 5'b00001;
         localparam ol2 = 1'b1;
         localparam ol3 = 4'b0000;
         localparam ol4 = 1'b0;
         begin
            or12 = arg_0;
            or1 = arg_1;
            or0 = or1[3:0];
            or2 = or0[3:3];
            or3 = or0[2:0];
            or4 = or3[0:0];
            or5 = or3[2:1];
            or6 = {or4, or5};
            or8 = or6[2:0];
            or7 = {ol0, or8};
            or9 = ol1;
            or9[4:1] = or7;
            case (or2)
               1'b1 : or10 = or9;
               default : or10 = ol1;
            endcase
            or11 = or12[1:1];
            or13 = |or11;
            or14 = or10;
            or14[4:1] = ol3;
            or15 = or14;
            or15[0:0] = ol4;
            or16 = or13 ? or15 : or10;
            kernel_master_kernel = or16;
         end
   endfunction
endmodule
module top_sub(input wire [1:0] clock_reset, input wire [4:0] i, output wire [2:0] o);
   wire [4:0] od;
   wire [1:0] d;
   wire [1:0] q;
   assign o = od[2:0];
   top_sub_data c0(.clock_reset(clock_reset), .i(d[1:0]), .o(q[1:0]));
   assign d = od[4:3];
   assign od = kernel_bottom_kernel(clock_reset, i, q);
   function [4:0] kernel_bottom_kernel(input reg [1:0] arg_0, input reg [4:0] arg_1, input reg [1:0] arg_2);
         reg [1:0] or0;
         // d
         reg [1:0] or1;
         // o
         reg [2:0] or2;
         reg [3:0] or3;
         reg [4:0] or4;
         reg [0:0] or5;
         reg [2:0] or6;
         reg [0:0] or7;
         reg [1:0] or8;
         // d
         reg [1:0] or9;
         // o
         reg [2:0] or10;
         // d
         reg [1:0] or11;
         // o
         reg [2:0] or12;
         reg [0:0] or13;
         reg [1:0] or14;
         reg [0:0] or15;
         // o
         reg [2:0] or16;
         // o
         reg [2:0] or17;
         reg [4:0] or18;
         localparam ol0 = 2'bXX;
         localparam ol1 = 3'bXX0;
         localparam ol2 = 1'b1;
         localparam ol3 = 1'b0;
         begin
            or14 = arg_0;
            or4 = arg_1;
            or0 = arg_2;
            or1 = ol0;
            or1[1:0] = or0;
            or2 = ol1;
            or2[2:1] = or0;
            or3 = or4[3:0];
            or5 = or3[3:3];
            or6 = or3[2:0];
            or7 = or6[0:0];
            or8 = or6[2:1];
            or9 = or1;
            or9[1:0] = or8;
            or10 = or2;
            or10[0:0] = or7;
            case (or5)
               1'b1 : or11 = or9;
               default : or11 = or1;
            endcase
            case (or5)
               1'b1 : or12 = or10;
               default : or12 = or2;
            endcase
            or13 = or14[1:1];
            or15 = |or13;
            or16 = or12;
            or16[0:0] = ol3;
            or17 = or15 ? or16 : or12;
            or18 = {or11, or17};
            kernel_bottom_kernel = or18;
         end
   endfunction
endmodule
module top_sub_data(input wire [1:0] clock_reset, input wire [1:0] i, output reg [1:0] o);
   wire  clock;
   wire  reset;
   assign clock = clock_reset[0];
   assign reset = clock_reset[1];
   initial begin
      o = 2'b00;
   end
   always @(posedge clock) begin
      if (reset) begin
         o <= 2'b00;
      end else begin
         o <= i;
      end
   end
endmodule
