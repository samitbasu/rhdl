// Testbench for synchronous module
    module testbench();
        reg [1:0] clock_reset;
        reg [4:0] i;
        wire [4:0] o;
        reg [4:0] rust_out;
        dut t (.arg_0(clock_reset),.arg_1(i),.out(o));
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
    //
    module dut(input wire [1:0] arg_0, input wire [4:0] arg_1, output reg [4:0] out);
        reg [0:0] r0;
        reg [0:0] r1;
        reg [0:0] r21;
        reg [0:0] r22;
        reg [0:0] r23;
        reg [0:0] r24;
        reg [0:0] r25;
        reg [0:0] r45;
        reg [0:0] r70;
        reg [0:0] r71;
        reg [0:0] r72;
        reg [0:0] r73;
        reg [0:0] r80;
        reg [0:0] r81;
        reg [0:0] r82;
        reg [0:0] r83;
        reg [0:0] r84;
        reg [0:0] r85;
        reg [0:0] r86;
        reg [0:0] r87;
        reg [0:0] r88;
        wire [0:0] r40;
        wire [0:0] r41;
        wire [0:0] r42;
        wire [0:0] r43;
        wire [0:0] r44;
        uut_input bb_0 (.o({ r44, r43, r42, r41, r40 }),.clock_reset({ r1, r0 }),.i({ r25, r24, r23, r22, r21 }));
        always @(*) begin
            r0 = arg_0[0];
            r1 = arg_0[1];
            r21 = arg_1[0];
            r22 = arg_1[1];
            r23 = arg_1[2];
            r24 = arg_1[3];
            r25 = arg_1[4];
            // !(t & bits(1)).any()
            //
            // (o, d, )
            //
            r83 = (r44) ? (r43) : (1'bX);
            r82 = (r44) ? (r42) : (1'bX);
            r81 = (r44) ? (r41) : (1'bX);
            r80 = (r44) ? (r40) : (1'bX);
            r45 = ~(r80);
            r88 = (r44) ? (r45) : (1'b0);
            r73 = (r45) ? (r43) : (1'b0);
            r87 = (r44) ? (r73) : (1'b0);
            r72 = (r45) ? (r42) : (1'b0);
            r86 = (r44) ? (r72) : (1'b0);
            r71 = (r45) ? (r41) : (1'b0);
            r85 = (r44) ? (r71) : (1'b0);
            r70 = (r45) ? (r40) : (1'b0);
            r84 = (r44) ? (r70) : (1'b0);
            // o = Some(data);
            //
            // if q.func {
            //    o = Some(data);
            // }
            //
            //
            // d.func = data;
            //
            // if let Some(data, )#true = q.input{
            //    d.func = data;
            //    if q.func {
            //       o = Some(data);
            //    }
            //
            // }
            //
            //
            // d.func = T::dont_care();
            //
            // let o = None();
            //
            // d.input = i;
            //
            // let d = D::<T>::dont_care();
            //
            out = { r88, r87, r86, r85, r84 };
        end
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
