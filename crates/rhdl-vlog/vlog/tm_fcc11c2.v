// Testbench for synchronous module
    module testbench();
        reg [1:0] clock_reset;
        wire [0:0] o;
        reg [0:0] rust_out;
        uut t (.clock_reset(clock_reset),.o(o));
        initial begin
            #0;
            clock_reset = 2'b10;
            rust_out = 1'b1;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 1 at time 0", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            rust_out = 1'b1;
            #1;
            clock_reset = 2'b11;
            rust_out = 1'b1;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 3 at time 51", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b10;
            rust_out = 1'b1;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 4 at time 100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            rust_out = 1'b1;
            #1;
            clock_reset = 2'b11;
            rust_out = 1'b1;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 6 at time 151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b10;
            rust_out = 1'b1;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 7 at time 200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            rust_out = 1'b1;
            #1;
            clock_reset = 2'b11;
            rust_out = 1'b1;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 9 at time 251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b10;
            rust_out = 1'b1;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 10 at time 300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            rust_out = 1'b1;
            #1;
            clock_reset = 2'b01;
            rust_out = 1'b1;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 12 at time 351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            rust_out = 1'b1;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 13 at time 400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            rust_out = 1'b1;
            $display("TESTBENCH OK", );
            $finish;
        end
    endmodule
    // synchronous circuit flow_graph::test_constant_propagates_through_unary::parent::Parent
    module uut(input wire [1:0] clock_reset, output wire [0:0] o);
        wire [4:0] od;
        wire [3:0] d;
        wire [0:0] q;
        assign o = od[0];
        uut_anyer c0 (.clock_reset(clock_reset),.i(d[3:0]),.o(q[0]));
        assign od = kernel_parent(clock_reset, q);
        assign d = od[4:1];
        function [4:0] kernel_parent(input reg [1:0] arg_0, input reg [0:0] arg_2);
            reg [0:0] or0;
            reg [4:0] or1;
            reg [1:0] or2;
            localparam ol0 = 4'b0011;
            begin
                or2 = arg_0;
                or0 = arg_2;
                // let d = D::dont_care();
                //
                // d.anyer = bits(3);
                //
                // let o = q.anyer;
                //
                // (o, d, )
                //
                or1 = { ol0, or0 };
                kernel_parent = or1;
            end
        endfunction
    endmodule
    // synchronous circuit flow_graph::anyer::U
    module uut_anyer(input wire [1:0] clock_reset, input wire [3:0] i, output wire [0:0] o);
        wire [0:0] od;
        assign o = od[0];
        assign od = kernel_anyer(clock_reset, i);
        function [0:0] kernel_anyer(input reg [1:0] arg_0, input reg [3:0] arg_1);
            reg [0:0] or0;
            reg [3:0] or1;
            reg [1:0] or2;
            begin
                or2 = arg_0;
                or1 = arg_1;
                // (i.any(), (), )
                //
                or0 = |(or1);
                kernel_anyer = or0;
            end
        endfunction
    endmodule
