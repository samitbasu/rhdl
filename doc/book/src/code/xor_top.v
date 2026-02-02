module xor_top(input wire [0:0] a, input wire [0:0] b, output wire [0:0] y);
   wire [1:0] inner_input;
   wire [0:0] inner_output;
   assign inner_input[0:0] = a;
   assign inner_input[1:1] = b;
   assign y = inner_output[0:0];
   inner inner_inst(.i(inner_input), .o(inner_output));
endmodule
module inner(input wire [1:0] i, output wire [0:0] o);
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
