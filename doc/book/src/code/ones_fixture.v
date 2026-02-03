module ones_top(input wire [7:0] dips, output wire [3:0] leds);
   wire [7:0] inner_input;
   wire [3:0] inner_output;
   assign inner_input[7:0] = dips;
   assign leds = inner_output[3:0];
   inner inner_inst(.i(inner_input), .o(inner_output));
endmodule
module inner(input wire [7:0] i, output wire [3:0] o);
   wire [3:0] od;
   assign o = od[3:0];
   assign od = kernel_one_counter(i);
   function [3:0] kernel_one_counter(input reg [7:0] arg_0);
         reg [7:0] r0;
         reg [7:0] r1;
         reg [0:0] r2;
         // count
         reg [3:0] r3;
         reg [7:0] r4;
         reg [0:0] r5;
         reg [3:0] r6;
         // count
         reg [3:0] r7;
         reg [7:0] r8;
         reg [0:0] r9;
         reg [3:0] r10;
         // count
         reg [3:0] r11;
         reg [7:0] r12;
         reg [0:0] r13;
         reg [3:0] r14;
         // count
         reg [3:0] r15;
         reg [7:0] r16;
         reg [0:0] r17;
         reg [3:0] r18;
         // count
         reg [3:0] r19;
         reg [7:0] r20;
         reg [0:0] r21;
         reg [3:0] r22;
         // count
         reg [3:0] r23;
         reg [7:0] r24;
         reg [0:0] r25;
         reg [3:0] r26;
         // count
         reg [3:0] r27;
         reg [7:0] r28;
         reg [0:0] r29;
         reg [3:0] r30;
         // count
         reg [3:0] r31;
         localparam l0 = 8'b00000001;
         localparam l1 = 4'b0001;
         localparam l2 = 4'b0000;
         localparam l3 = 8'b00000010;
         localparam l4 = 4'b0001;
         localparam l5 = 8'b00000100;
         localparam l6 = 4'b0001;
         localparam l7 = 8'b00001000;
         localparam l8 = 4'b0001;
         localparam l9 = 8'b00010000;
         localparam l10 = 4'b0001;
         localparam l11 = 8'b00100000;
         localparam l12 = 4'b0001;
         localparam l13 = 8'b01000000;
         localparam l14 = 4'b0001;
         localparam l15 = 8'b10000000;
         localparam l16 = 4'b0001;
         begin
            r0 = arg_0;
            r1 = r0 & l0;
            r2 = |r1;
            r3 = r2 ? l1 : l2;
            r4 = r0 & l3;
            r5 = |r4;
            r6 = r3 + l4;
            r7 = r5 ? r6 : r3;
            r8 = r0 & l5;
            r9 = |r8;
            r10 = r7 + l6;
            r11 = r9 ? r10 : r7;
            r12 = r0 & l7;
            r13 = |r12;
            r14 = r11 + l8;
            r15 = r13 ? r14 : r11;
            r16 = r0 & l9;
            r17 = |r16;
            r18 = r15 + l10;
            r19 = r17 ? r18 : r15;
            r20 = r0 & l11;
            r21 = |r20;
            r22 = r19 + l12;
            r23 = r21 ? r22 : r19;
            r24 = r0 & l13;
            r25 = |r24;
            r26 = r23 + l14;
            r27 = r25 ? r26 : r23;
            r28 = r0 & l15;
            r29 = |r28;
            r30 = r27 + l16;
            r31 = r29 ? r30 : r27;
            kernel_one_counter = r31;
         end
   endfunction
endmodule
