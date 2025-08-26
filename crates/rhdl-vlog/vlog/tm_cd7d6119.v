// Testbench for synchronous module
    module testbench();
        reg [1:0] clock_reset;
        reg [0:0] i;
        wire [3:0] o;
        reg [3:0] rust_out;
        uut t (.clock_reset(clock_reset),.i(i),.o(o));
        initial begin
            #0;
            clock_reset = 2'b10;
            i = 1'b0;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 1 at time 0", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            i = 1'b0;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b11;
            i = 1'b0;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 3 at time 51", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b10;
            i = 1'b0;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 4 at time 100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            i = 1'b0;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b11;
            i = 1'b0;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 6 at time 151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b10;
            i = 1'b0;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 7 at time 200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            i = 1'b0;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b11;
            i = 1'b0;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 9 at time 251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b10;
            i = 1'b0;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 10 at time 300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            i = 1'b0;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b01;
            i = 1'b0;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 12 at time 351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 1'b0;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 13 at time 400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 1'b0;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b01;
            i = 1'b1;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 15 at time 451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 1'b1;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 16 at time 500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 1'b1;
            rust_out = 4'b0100;
            $display("TESTBENCH OK", );
            $finish;
        end
    endmodule
    // synchronous circuit flow_graph::test_constant_propagates_through_indexing::parent::Parent
    module uut(input wire [1:0] clock_reset, input wire [0:0] i, output wire [3:0] o);
        wire [21:0] od;
        wire [17:0] d;
        wire [3:0] q;
        assign o = od[3:0];
        uut_indexor c0 (.clock_reset(clock_reset),.i(d[17:0]),.o(q[3:0]));
        assign od = kernel_parent(clock_reset, i, q);
        assign d = od[21:4];
        function [21:0] kernel_parent(input reg [1:0] arg_0, input reg [0:0] arg_1, input reg [3:0] arg_2);
            reg [3:0] or0;
            reg [3:0] or1;
            reg [0:0] or2;
            reg [21:0] or3;
            reg [1:0] or4;
            localparam ol0 = 4'b0011;
            localparam ol1 = 18'b010000110010000111;
            begin
                or4 = arg_0;
                or2 = arg_1;
                or0 = arg_2;
                // let d = D::dont_care();
                //
                // let index = bits(3);
                //
                // d.indexor = (index, [bits(1), bits(2), bits(3), bits(4), ], );
                //
                // let o = if i {
                //    q.indexor
                // }
                //  else {
                //    bits(3)
                // }
                // ;
                //
                // q.indexor
                //
                // bits(3)
                //
                or1 = (or2) ? (or0) : (ol0);
                // (o, d, )
                //
                or3 = { ol1, or1 };
                kernel_parent = or3;
            end
        endfunction
    endmodule
    // synchronous circuit flow_graph::indexor::U
    module uut_indexor(input wire [1:0] clock_reset, input wire [17:0] i, output wire [3:0] o);
        wire [3:0] od;
        assign o = od[3:0];
        assign od = kernel_indexor(clock_reset, i);
        function [3:0] kernel_indexor(input reg [1:0] arg_0, input reg [17:0] arg_1);
            reg [1:0] or0;
            reg [17:0] or1;
            reg [15:0] or2;
            reg [3:0] or3;
            reg [3:0] or4;
            reg [3:0] or5;
            reg [3:0] or6;
            reg [3:0] or7;
            reg [1:0] or8;
            localparam ol0 = 2'b00;
            localparam ol1 = 2'b01;
            localparam ol2 = 2'b10;
            localparam ol3 = 2'b11;
            begin
                or8 = arg_0;
                or1 = arg_1;
                // let (ndx, arr, ) = i;
                //
                or0 = or1[1:0];
                or2 = or1[17:2];
                // let out = arr[ndx];
                //
                or3 = or2[3:0];
                or4 = or2[7:4];
                or5 = or2[11:8];
                or6 = or2[15:12];
                case (or0)
                    2'b00: or7 = or3;
                    2'b01: or7 = or4;
                    2'b10: or7 = or5;
                    2'b11: or7 = or6;
                endcase
                // (out, (), )
                //
                kernel_indexor = or7;
            end
        endfunction
    endmodule
