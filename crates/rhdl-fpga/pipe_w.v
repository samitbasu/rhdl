// Testbench for synchronous module
    module testbench();
        reg [1:0] clock_reset;
        reg [4:0] i;
        wire [4:0] o;
        reg [4:0] rust_out;
        uut t (.clock_reset(clock_reset),.i(i),.o(o));
        initial begin
            #0;
            clock_reset = 2'b10;
            i = 5'b00000;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 1 at time 0", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            i = 5'b00000;
            rust_out = 5'b00000;
            #1;
            clock_reset = 2'b01;
            i = 5'b10010;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 3 at time 51", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b10010;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 4 at time 100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b10010;
            rust_out = 5'b10010;
            #1;
            clock_reset = 2'b01;
            i = 5'b10101;
            rust_out = 5'b10010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 6 at time 151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b10101;
            rust_out = 5'b10010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 7 at time 200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b10101;
            rust_out = 5'b00000;
            #1;
            clock_reset = 2'b01;
            i = 5'b10110;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 9 at time 251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b10110;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 10 at time 300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b10110;
            rust_out = 5'b10110;
            #1;
            clock_reset = 2'b01;
            i = 5'b10010;
            rust_out = 5'b10110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 12 at time 351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b10010;
            rust_out = 5'b10110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 13 at time 400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b10010;
            rust_out = 5'b10010;
            #1;
            clock_reset = 2'b01;
            i = 5'b11010;
            rust_out = 5'b10010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 15 at time 451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b11010;
            rust_out = 5'b10010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 16 at time 500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b11010;
            rust_out = 5'b11010;
            #1;
            clock_reset = 2'b01;
            i = 5'b10001;
            rust_out = 5'b11010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 18 at time 551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b10001;
            rust_out = 5'b11010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 19 at time 600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b10001;
            rust_out = 5'b00000;
            #1;
            clock_reset = 2'b01;
            i = 5'b11100;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 21 at time 651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b11100;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 22 at time 700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b11100;
            rust_out = 5'b11100;
            #1;
            clock_reset = 2'b01;
            i = 5'b11100;
            rust_out = 5'b11100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 24 at time 751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b11100;
            rust_out = 5'b11100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 25 at time 800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b11100;
            rust_out = 5'b11100;
            #1;
            clock_reset = 2'b01;
            i = 5'b10101;
            rust_out = 5'b11100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 27 at time 851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b10101;
            rust_out = 5'b11100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 28 at time 900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b10101;
            rust_out = 5'b00000;
            #1;
            clock_reset = 2'b01;
            i = 5'b00000;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 30 at time 951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b00000;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 31 at time 1000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b00000;
            rust_out = 5'b00000;
            #1;
            clock_reset = 2'b01;
            i = 5'b11101;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 33 at time 1051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b11101;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 34 at time 1100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b11101;
            rust_out = 5'b00000;
            #1;
            clock_reset = 2'b01;
            i = 5'b00000;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 36 at time 1151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b00000;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 37 at time 1200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b00000;
            rust_out = 5'b00000;
            #1;
            clock_reset = 2'b01;
            i = 5'b10001;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 39 at time 1251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b10001;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 40 at time 1300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b10001;
            rust_out = 5'b00000;
            #1;
            clock_reset = 2'b01;
            i = 5'b11011;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 42 at time 1351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b11011;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 43 at time 1400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b11011;
            rust_out = 5'b00000;
            #1;
            clock_reset = 2'b01;
            i = 5'b00000;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 45 at time 1451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b00000;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 46 at time 1500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b00000;
            rust_out = 5'b00000;
            #1;
            clock_reset = 2'b01;
            i = 5'b10110;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 48 at time 1551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b10110;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 49 at time 1600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b10110;
            rust_out = 5'b10110;
            #1;
            clock_reset = 2'b01;
            i = 5'b00000;
            rust_out = 5'b10110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 51 at time 1651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b00000;
            rust_out = 5'b10110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 52 at time 1700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b00000;
            rust_out = 5'b00000;
            #1;
            clock_reset = 2'b01;
            i = 5'b00000;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 54 at time 1751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b00000;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 55 at time 1800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b00000;
            rust_out = 5'b00000;
            #1;
            clock_reset = 2'b01;
            i = 5'b10001;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 57 at time 1851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b10001;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 58 at time 1900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b10001;
            rust_out = 5'b00000;
            #1;
            clock_reset = 2'b01;
            i = 5'b00000;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 60 at time 1951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 5'b00000;
            rust_out = 5'b00000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 61 at time 2000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 5'b00000;
            rust_out = 5'b00000;
            $display("TESTBENCH OK", );
            $finish;
        end
    endmodule
    // synchronous circuit rhdl_fpga::pipe::filter::Filter<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U4>>
    module uut(input wire [1:0] clock_reset, input wire [4:0] i, output wire [4:0] o);
        wire [13:0] od;
        wire [8:0] d;
        wire [5:0] q;
        assign o = od[4:0];
        uut_func c0 (.clock_reset(clock_reset),.i(d[8:5]),.o(q[5]));
        uut_input c1 (.clock_reset(clock_reset),.i(d[4:0]),.o(q[4:0]));
        assign od = kernel_kernel(clock_reset, i, q);
        assign d = od[13:5];
        function [13:0] kernel_kernel(input reg [1:0] arg_0, input reg [4:0] arg_1, input reg [5:0] arg_2);
            reg [8:0] r0;
            reg [4:0] r1;
            reg [8:0] r4;
            reg [4:0] r5;
            reg [5:0] r6;
            reg [0:0] r7;
            reg [3:0] r8;
            reg [8:0] r10;
            reg [0:0] r11;
            reg [4:0] r12;
            reg [3:0] r13;
            reg [4:0] r14;
            reg [8:0] r15;
            reg [4:0] r16;
            reg [13:0] r17;
            reg [1:0] r18;
            localparam l0 = 9'bxxxxxxxxx;
            localparam l3 = 4'bxxxx;
            localparam l4 = 1'b1;
            localparam l5 = 1'b1;
            localparam l7 = 5'b00000;
            begin
                r18 = arg_0;
                r1 = arg_1;
                r6 = arg_2;
                // let d = D::<T>::dont_care();
                //
                // d.input = i;
                //
                r0 = l0; r0[4:0] = r1;
                // let o = None();
                //
                // d.func = T::dont_care();
                //
                r4 = r0; r4[8:5] = l3;
                // if let Some(data, )#true = q.input{
                //    d.func = data;
                //    if q.func {
                //       o = Some(data);
                //    }
                //
                // }
                //
                //
                r5 = r6[4:0];
                r7 = r5[4];
                r8 = r5[3:0];
                // d.func = data;
                //
                r10 = r4; r10[8:5] = r8;
                // if q.func {
                //    o = Some(data);
                // }
                //
                //
                r11 = r6[5];
                // o = Some(data);
                //
                r13 = r8[3:0];
                r12 = { l4, r13 };
                r14 = (r11) ? (r12) : (l7);
                case (r7)
                    1'b1: r15 = r10;
                    default: r15 = r4;
                endcase
                case (r7)
                    1'b1: r16 = r14;
                    default: r16 = l7;
                endcase
                // (o, d, )
                //
                r17 = { r15, r16 };
                kernel_kernel = r17;
            end
        endfunction
    endmodule
    // synchronous circuit rhdl::rhdl_core::circuit::func::Func<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U4>, bool>
    module uut_func(input wire [1:0] clock_reset, input wire [3:0] i, output wire [0:0] o);
        assign o = kernel_keep_even(clock_reset, i);
        function [0:0] kernel_keep_even(input reg [1:0] arg_0, input reg [3:0] arg_1);
            reg [3:0] r0;
            reg [3:0] r1;
            reg [0:0] r2;
            reg [0:0] r3;
            reg [1:0] r4;
            localparam l0 = 4'b0001;
            begin
                r4 = arg_0;
                r1 = arg_1;
                // !(t & bits(1)).any()
                //
                r0 = r1 & l0;
                r2 = |(r0);
                r3 = ~(r2);
                kernel_keep_even = r3;
            end
        endfunction
    endmodule
    //
    module uut_input(input wire [1:0] clock_reset, input wire [4:0] i, output reg [4:0] o);
        wire [0:0] clock;
        wire [0:0] reset;
        initial begin
            o = 5'b00000;
        end
        assign clock = clock_reset[0];
        assign reset = clock_reset[1];
        always @(posedge clock) begin
            if (reset)
            begin
                o <= 5'b00000;
            end else begin
                o <= i;
            end
        end
    endmodule
