// Testbench for synchronous module
    module testbench();
        reg [1:0] clock_reset;
        wire [0:0] o;
        reg [0:0] rust_out;
        dut t (.arg_0(clock_reset),.out(o));
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
    //
    module dut(input wire [1:0] arg_0, output reg [0:0] out);
        reg [0:0] r0;
        reg [0:0] r1;
        initial begin
            r0 = arg_0[0];
            r1 = arg_0[1];
            // let d = D::dont_care();
            //
            // d.anyer = bits(3);
            //
            // let o = q.anyer;
            //
            // (o, d, )
            //
            // (i.any(), (), )
            //
            out = { 1'b1 };
        end
    endmodule
