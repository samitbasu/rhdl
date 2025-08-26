// Testbench for synchronous module
    module testbench();
        reg [1:0] clock_reset;
        wire [3:0] o;
        reg [3:0] rust_out;
        uut t (.clock_reset(clock_reset),.o(o));
        initial begin
            #0;
            clock_reset = 2'b10;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 1 at time 0", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b11;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 3 at time 51", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b10;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 4 at time 100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b11;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 6 at time 151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b10;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 7 at time 200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b11;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 9 at time 251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b10;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 10 at time 300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b01;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 12 at time 351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 13 at time 400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            rust_out = 4'b0111;
            $display("TESTBENCH OK", );
            $finish;
        end
    endmodule
    // synchronous circuit flow_graph::test_constant_propagates_through_adder::parent::Parent
    module uut(input wire [1:0] clock_reset, output wire [3:0] o);
        wire [11:0] od;
        wire [7:0] d;
        wire [3:0] q;
        assign o = od[3:0];
        uut_adder c0 (.clock_reset(clock_reset),.i(d[7:0]),.o(q[3:0]));
        assign od = kernel_parent(clock_reset, q);
        assign d = od[11:4];
        function [11:0] kernel_parent(input reg [1:0] arg_0, input reg [3:0] arg_2);
            reg [3:0] or0;
            reg [11:0] or1;
            reg [1:0] or2;
            localparam ol0 = 8'b01000011;
            begin
                or2 = arg_0;
                or0 = arg_2;
                // let (a, b, ) = (bits(3), bits(4), );
                //
                // let d = D::dont_care();
                //
                // d.adder = (a, b, );
                //
                // let o = q.adder;
                //
                // (o, d, )
                //
                or1 = { ol0, or0 };
                kernel_parent = or1;
            end
        endfunction
    endmodule
    // synchronous circuit flow_graph::adder::U
    module uut_adder(input wire [1:0] clock_reset, input wire [7:0] i, output wire [3:0] o);
        wire [3:0] od;
        assign o = od[3:0];
        assign od = kernel_adder(clock_reset, i);
        function [3:0] kernel_adder(input reg [1:0] arg_0, input reg [7:0] arg_1);
            reg [3:0] or0;
            reg [7:0] or1;
            reg [3:0] or2;
            reg [3:0] or3;
            reg [1:0] or4;
            begin
                or4 = arg_0;
                or1 = arg_1;
                // let (a, b, ) = i;
                //
                or0 = or1[3:0];
                or2 = or1[7:4];
                // let sum = a + b;
                //
                or3 = or0 + or2;
                // (sum, (), )
                //
                kernel_adder = or3;
            end
        endfunction
    endmodule
