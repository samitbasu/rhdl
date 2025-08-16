// Testbench for synchronous module
    module testbench();
        reg [1:0] clock_reset;
        reg [0:0] i;
        wire [15:0] o;
        reg [15:0] rust_out;
        dut t (.arg_0(clock_reset),.arg_1(i),.out(o));
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
    //
    module dut(input wire [1:0] arg_0, input wire [0:0] arg_1, output reg [15:0] out);
        reg [0:0] r0;
        reg [0:0] r1;
        reg [0:0] r2;
        always @(*) begin
            r0 = arg_0[0];
            r1 = arg_0[1];
            r2 = arg_1[0];
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
            // (o, d, )
            //
            // let (ndx, arr, val, ) = i;
            //
            // arr[ndx] = val;
            //
            // (arr, (), )
            //
            out = { 1'b0, 1'b1, 1'b0, r2, 1'b0, 1'b0, 1'b1, 1'b1, 1'b0, 1'b0, 1'b1, 1'b0, 1'b0, 1'b0, 1'b0, 1'b1 };
        end
    endmodule
