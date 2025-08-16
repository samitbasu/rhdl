// Testbench for synchronous module
    module testbench();
        reg [1:0] clock_reset;
        reg [0:0] i;
        wire [15:0] o;
        reg [15:0] rust_out;
        uut t (.clock_reset(clock_reset),.i(i),.o(o));
        initial begin
            #0;
            clock_reset = 2'b10;
            i = 1'b0;
            rust_out = 16'b0100001100100001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 1 at time 0", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            i = 1'b0;
            rust_out = 16'b0100001100100001;
            #1;
            clock_reset = 2'b11;
            i = 1'b0;
            rust_out = 16'b0100001100100001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 3 at time 51", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b10;
            i = 1'b0;
            rust_out = 16'b0100001100100001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 4 at time 100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            i = 1'b0;
            rust_out = 16'b0100001100100001;
            #1;
            clock_reset = 2'b11;
            i = 1'b0;
            rust_out = 16'b0100001100100001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 6 at time 151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b10;
            i = 1'b0;
            rust_out = 16'b0100001100100001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 7 at time 200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            i = 1'b0;
            rust_out = 16'b0100001100100001;
            #1;
            clock_reset = 2'b11;
            i = 1'b0;
            rust_out = 16'b0100001100100001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 9 at time 251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b10;
            i = 1'b0;
            rust_out = 16'b0100001100100001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 10 at time 300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            i = 1'b0;
            rust_out = 16'b0100001100100001;
            #1;
            clock_reset = 2'b01;
            i = 1'b0;
            rust_out = 16'b0100001100100001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 12 at time 351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 1'b0;
            rust_out = 16'b0100001100100001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 13 at time 400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 1'b0;
            rust_out = 16'b0100001100100001;
            #1;
            clock_reset = 2'b01;
            i = 1'b1;
            rust_out = 16'b0101001100100001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 15 at time 451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 1'b1;
            rust_out = 16'b0101001100100001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 16 at time 500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 1'b1;
            rust_out = 16'b0101001100100001;
            $display("TESTBENCH OK", );
            $finish;
        end
    endmodule
    // synchronous circuit flow_graph::test_constant_propagates_through_splicing::parent::Parent
    module uut(input wire [1:0] clock_reset, input wire [0:0] i, output wire [15:0] o);
        wire [37:0] od;
        wire [21:0] d;
        wire [15:0] q;
        assign o = od[15:0];
        uut_splicer c0 (.clock_reset(clock_reset),.i(d[21:0]),.o(q[15:0]));
        assign od = kernel_parent(clock_reset, i, q);
        assign d = od[37:16];
        function [37:0] kernel_parent(input reg [1:0] arg_0, input reg [0:0] arg_1, input reg [15:0] arg_2);
            reg [15:0] or0;
            reg [15:0] or1;
            reg [0:0] or2;
            reg [37:0] or3;
            reg [1:0] or4;
            localparam ol0 = 16'b0100001100100001;
            localparam ol1 = 22'b0101010000110010000111;
            begin
                or4 = arg_0;
                or2 = arg_1;
                or0 = arg_2;
                // let d = D::dont_care();
                //
                // let index = bits(3);
                //
                // let orig = [bits(1), bits(2), bits(3), bits(4), ];
                //
                // d.splicer = (index, orig, bits(5), );
                //
                // let o = if i {
                //    q.splicer
                // }
                //  else {
                //    orig
                // }
                // ;
                //
                // q.splicer
                //
                // orig
                //
                or1 = (or2) ? (or0) : (ol0);
                // (o, d, )
                //
                or3 = { ol1, or1 };
                kernel_parent = or3;
            end
        endfunction
    endmodule
    // synchronous circuit flow_graph::splicer::U
    module uut_splicer(input wire [1:0] clock_reset, input wire [21:0] i, output wire [15:0] o);
        wire [15:0] od;
        assign o = od[15:0];
        assign od = kernel_splicer(clock_reset, i);
        function [15:0] kernel_splicer(input reg [1:0] arg_0, input reg [21:0] arg_1);
            reg [1:0] or0;
            reg [21:0] or1;
            reg [15:0] or2;
            reg [3:0] or3;
            reg [15:0] or4;  // arr
            reg [15:0] or5;
            reg [15:0] or6;
            reg [15:0] or7;
            reg [15:0] or8;
            reg [1:0] or9;
            localparam ol0 = 2'b00;
            localparam ol1 = 2'b01;
            localparam ol2 = 2'b10;
            localparam ol3 = 2'b11;
            begin
                or9 = arg_0;
                or1 = arg_1;
                // let (ndx, arr, val, ) = i;
                //
                or0 = or1[1:0];
                or2 = or1[17:2];
                or3 = or1[21:18];
                // arr[ndx] = val;
                //
                or5 = or2; or5[3:0] = or3;
                or6 = or2; or6[7:4] = or3;
                or7 = or2; or7[11:8] = or3;
                or8 = or2; or8[15:12] = or3;
                case (or0)
                    2'b00: or4 = or5;
                    2'b01: or4 = or6;
                    2'b10: or4 = or7;
                    2'b11: or4 = or8;
                endcase
                // (arr, (), )
                //
                kernel_splicer = or4;
            end
        endfunction
    endmodule
