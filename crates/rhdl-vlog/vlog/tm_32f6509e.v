// Testbench for synchronous module
    module testbench();
        reg [1:0] clock_reset;
        reg [7:0] i;
        wire [3:0] o;
        reg [3:0] rust_out;
        uut t (.clock_reset(clock_reset),.i(i),.o(o));
        initial begin
            #0;
            clock_reset = 2'b10;
            i = 8'b00000000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 1 at time 0", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            i = 8'b00000000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b11;
            i = 8'b00000000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 3 at time 51", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b10;
            i = 8'b00000000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 4 at time 100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            i = 8'b00000000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b11;
            i = 8'b00000000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 6 at time 151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b10;
            i = 8'b00000000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 7 at time 200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            i = 8'b00000000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b11;
            i = 8'b00000000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 9 at time 251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b10;
            i = 8'b00000000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 10 at time 300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b11;
            i = 8'b00000000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b01;
            i = 8'b00000000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 12 at time 351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00000000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 13 at time 400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00000000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b01;
            i = 8'b00010000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 15 at time 451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00010000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 16 at time 500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00010000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b01;
            i = 8'b00100000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 18 at time 551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00100000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 19 at time 600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00100000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b01;
            i = 8'b00110000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 21 at time 651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00110000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 22 at time 700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00110000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b01;
            i = 8'b01000000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 24 at time 751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01000000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 25 at time 800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01000000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b01;
            i = 8'b01010000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 27 at time 851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01010000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 28 at time 900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01010000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b01;
            i = 8'b01100000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 30 at time 951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01100000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 31 at time 1000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01100000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b01;
            i = 8'b01110000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 33 at time 1051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01110000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 34 at time 1100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01110000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b01;
            i = 8'b10000000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 36 at time 1151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10000000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 37 at time 1200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10000000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b01;
            i = 8'b10010000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 39 at time 1251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10010000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 40 at time 1300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10010000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b01;
            i = 8'b10100000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 42 at time 1351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10100000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 43 at time 1400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10100000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b01;
            i = 8'b10110000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 45 at time 1451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10110000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 46 at time 1500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10110000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b01;
            i = 8'b11000000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 48 at time 1551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11000000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 49 at time 1600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11000000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b01;
            i = 8'b11010000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 51 at time 1651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11010000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 52 at time 1700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11010000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b01;
            i = 8'b11100000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 54 at time 1751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11100000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 55 at time 1800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11100000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b01;
            i = 8'b11110000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 57 at time 1851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11110000;
            rust_out = 4'b0000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 58 at time 1900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11110000;
            rust_out = 4'b0000;
            #1;
            clock_reset = 2'b01;
            i = 8'b00000001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 60 at time 1951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00000001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 61 at time 2000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00000001;
            rust_out = 4'b0001;
            #1;
            clock_reset = 2'b01;
            i = 8'b00010001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 63 at time 2051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00010001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 64 at time 2100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00010001;
            rust_out = 4'b0001;
            #1;
            clock_reset = 2'b01;
            i = 8'b00100001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 66 at time 2151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00100001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 67 at time 2200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00100001;
            rust_out = 4'b0001;
            #1;
            clock_reset = 2'b01;
            i = 8'b00110001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 69 at time 2251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00110001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 70 at time 2300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00110001;
            rust_out = 4'b0001;
            #1;
            clock_reset = 2'b01;
            i = 8'b01000001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 72 at time 2351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01000001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 73 at time 2400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01000001;
            rust_out = 4'b0001;
            #1;
            clock_reset = 2'b01;
            i = 8'b01010001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 75 at time 2451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01010001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 76 at time 2500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01010001;
            rust_out = 4'b0001;
            #1;
            clock_reset = 2'b01;
            i = 8'b01100001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 78 at time 2551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01100001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 79 at time 2600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01100001;
            rust_out = 4'b0001;
            #1;
            clock_reset = 2'b01;
            i = 8'b01110001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 81 at time 2651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01110001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 82 at time 2700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01110001;
            rust_out = 4'b0001;
            #1;
            clock_reset = 2'b01;
            i = 8'b10000001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 84 at time 2751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10000001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 85 at time 2800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10000001;
            rust_out = 4'b0001;
            #1;
            clock_reset = 2'b01;
            i = 8'b10010001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 87 at time 2851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10010001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 88 at time 2900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10010001;
            rust_out = 4'b0001;
            #1;
            clock_reset = 2'b01;
            i = 8'b10100001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 90 at time 2951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10100001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 91 at time 3000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10100001;
            rust_out = 4'b0001;
            #1;
            clock_reset = 2'b01;
            i = 8'b10110001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 93 at time 3051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10110001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 94 at time 3100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10110001;
            rust_out = 4'b0001;
            #1;
            clock_reset = 2'b01;
            i = 8'b11000001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 96 at time 3151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11000001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 97 at time 3200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11000001;
            rust_out = 4'b0001;
            #1;
            clock_reset = 2'b01;
            i = 8'b11010001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 99 at time 3251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11010001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 100 at time 3300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11010001;
            rust_out = 4'b0001;
            #1;
            clock_reset = 2'b01;
            i = 8'b11100001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 102 at time 3351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11100001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 103 at time 3400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11100001;
            rust_out = 4'b0001;
            #1;
            clock_reset = 2'b01;
            i = 8'b11110001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 105 at time 3451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11110001;
            rust_out = 4'b0001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 106 at time 3500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11110001;
            rust_out = 4'b0001;
            #1;
            clock_reset = 2'b01;
            i = 8'b00000010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 108 at time 3551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00000010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 109 at time 3600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00000010;
            rust_out = 4'b0010;
            #1;
            clock_reset = 2'b01;
            i = 8'b00010010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 111 at time 3651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00010010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 112 at time 3700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00010010;
            rust_out = 4'b0010;
            #1;
            clock_reset = 2'b01;
            i = 8'b00100010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 114 at time 3751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00100010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 115 at time 3800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00100010;
            rust_out = 4'b0010;
            #1;
            clock_reset = 2'b01;
            i = 8'b00110010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 117 at time 3851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00110010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 118 at time 3900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00110010;
            rust_out = 4'b0010;
            #1;
            clock_reset = 2'b01;
            i = 8'b01000010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 120 at time 3951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01000010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 121 at time 4000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01000010;
            rust_out = 4'b0010;
            #1;
            clock_reset = 2'b01;
            i = 8'b01010010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 123 at time 4051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01010010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 124 at time 4100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01010010;
            rust_out = 4'b0010;
            #1;
            clock_reset = 2'b01;
            i = 8'b01100010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 126 at time 4151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01100010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 127 at time 4200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01100010;
            rust_out = 4'b0010;
            #1;
            clock_reset = 2'b01;
            i = 8'b01110010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 129 at time 4251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01110010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 130 at time 4300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01110010;
            rust_out = 4'b0010;
            #1;
            clock_reset = 2'b01;
            i = 8'b10000010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 132 at time 4351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10000010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 133 at time 4400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10000010;
            rust_out = 4'b0010;
            #1;
            clock_reset = 2'b01;
            i = 8'b10010010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 135 at time 4451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10010010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 136 at time 4500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10010010;
            rust_out = 4'b0010;
            #1;
            clock_reset = 2'b01;
            i = 8'b10100010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 138 at time 4551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10100010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 139 at time 4600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10100010;
            rust_out = 4'b0010;
            #1;
            clock_reset = 2'b01;
            i = 8'b10110010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 141 at time 4651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10110010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 142 at time 4700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10110010;
            rust_out = 4'b0010;
            #1;
            clock_reset = 2'b01;
            i = 8'b11000010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 144 at time 4751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11000010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 145 at time 4800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11000010;
            rust_out = 4'b0010;
            #1;
            clock_reset = 2'b01;
            i = 8'b11010010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 147 at time 4851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11010010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 148 at time 4900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11010010;
            rust_out = 4'b0010;
            #1;
            clock_reset = 2'b01;
            i = 8'b11100010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 150 at time 4951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11100010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 151 at time 5000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11100010;
            rust_out = 4'b0010;
            #1;
            clock_reset = 2'b01;
            i = 8'b11110010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 153 at time 5051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11110010;
            rust_out = 4'b0010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 154 at time 5100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11110010;
            rust_out = 4'b0010;
            #1;
            clock_reset = 2'b01;
            i = 8'b00000011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 156 at time 5151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00000011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 157 at time 5200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00000011;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b01;
            i = 8'b00010011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 159 at time 5251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00010011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 160 at time 5300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00010011;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b01;
            i = 8'b00100011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 162 at time 5351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00100011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 163 at time 5400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00100011;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b01;
            i = 8'b00110011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 165 at time 5451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00110011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 166 at time 5500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00110011;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b01;
            i = 8'b01000011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 168 at time 5551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01000011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 169 at time 5600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01000011;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b01;
            i = 8'b01010011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 171 at time 5651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01010011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 172 at time 5700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01010011;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b01;
            i = 8'b01100011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 174 at time 5751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01100011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 175 at time 5800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01100011;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b01;
            i = 8'b01110011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 177 at time 5851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01110011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 178 at time 5900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01110011;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b01;
            i = 8'b10000011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 180 at time 5951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10000011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 181 at time 6000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10000011;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b01;
            i = 8'b10010011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 183 at time 6051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10010011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 184 at time 6100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10010011;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b01;
            i = 8'b10100011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 186 at time 6151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10100011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 187 at time 6200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10100011;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b01;
            i = 8'b10110011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 189 at time 6251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10110011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 190 at time 6300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10110011;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b01;
            i = 8'b11000011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 192 at time 6351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11000011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 193 at time 6400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11000011;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b01;
            i = 8'b11010011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 195 at time 6451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11010011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 196 at time 6500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11010011;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b01;
            i = 8'b11100011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 198 at time 6551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11100011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 199 at time 6600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11100011;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b01;
            i = 8'b11110011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 201 at time 6651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11110011;
            rust_out = 4'b0011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 202 at time 6700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11110011;
            rust_out = 4'b0011;
            #1;
            clock_reset = 2'b01;
            i = 8'b00000100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 204 at time 6751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00000100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 205 at time 6800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00000100;
            rust_out = 4'b0100;
            #1;
            clock_reset = 2'b01;
            i = 8'b00010100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 207 at time 6851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00010100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 208 at time 6900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00010100;
            rust_out = 4'b0100;
            #1;
            clock_reset = 2'b01;
            i = 8'b00100100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 210 at time 6951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00100100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 211 at time 7000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00100100;
            rust_out = 4'b0100;
            #1;
            clock_reset = 2'b01;
            i = 8'b00110100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 213 at time 7051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00110100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 214 at time 7100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00110100;
            rust_out = 4'b0100;
            #1;
            clock_reset = 2'b01;
            i = 8'b01000100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 216 at time 7151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01000100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 217 at time 7200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01000100;
            rust_out = 4'b0100;
            #1;
            clock_reset = 2'b01;
            i = 8'b01010100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 219 at time 7251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01010100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 220 at time 7300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01010100;
            rust_out = 4'b0100;
            #1;
            clock_reset = 2'b01;
            i = 8'b01100100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 222 at time 7351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01100100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 223 at time 7400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01100100;
            rust_out = 4'b0100;
            #1;
            clock_reset = 2'b01;
            i = 8'b01110100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 225 at time 7451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01110100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 226 at time 7500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01110100;
            rust_out = 4'b0100;
            #1;
            clock_reset = 2'b01;
            i = 8'b10000100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 228 at time 7551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10000100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 229 at time 7600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10000100;
            rust_out = 4'b0100;
            #1;
            clock_reset = 2'b01;
            i = 8'b10010100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 231 at time 7651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10010100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 232 at time 7700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10010100;
            rust_out = 4'b0100;
            #1;
            clock_reset = 2'b01;
            i = 8'b10100100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 234 at time 7751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10100100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 235 at time 7800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10100100;
            rust_out = 4'b0100;
            #1;
            clock_reset = 2'b01;
            i = 8'b10110100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 237 at time 7851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10110100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 238 at time 7900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10110100;
            rust_out = 4'b0100;
            #1;
            clock_reset = 2'b01;
            i = 8'b11000100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 240 at time 7951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11000100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 241 at time 8000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11000100;
            rust_out = 4'b0100;
            #1;
            clock_reset = 2'b01;
            i = 8'b11010100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 243 at time 8051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11010100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 244 at time 8100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11010100;
            rust_out = 4'b0100;
            #1;
            clock_reset = 2'b01;
            i = 8'b11100100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 246 at time 8151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11100100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 247 at time 8200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11100100;
            rust_out = 4'b0100;
            #1;
            clock_reset = 2'b01;
            i = 8'b11110100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 249 at time 8251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11110100;
            rust_out = 4'b0100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 250 at time 8300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11110100;
            rust_out = 4'b0100;
            #1;
            clock_reset = 2'b01;
            i = 8'b00000101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 252 at time 8351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00000101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 253 at time 8400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00000101;
            rust_out = 4'b0101;
            #1;
            clock_reset = 2'b01;
            i = 8'b00010101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 255 at time 8451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00010101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 256 at time 8500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00010101;
            rust_out = 4'b0101;
            #1;
            clock_reset = 2'b01;
            i = 8'b00100101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 258 at time 8551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00100101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 259 at time 8600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00100101;
            rust_out = 4'b0101;
            #1;
            clock_reset = 2'b01;
            i = 8'b00110101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 261 at time 8651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00110101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 262 at time 8700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00110101;
            rust_out = 4'b0101;
            #1;
            clock_reset = 2'b01;
            i = 8'b01000101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 264 at time 8751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01000101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 265 at time 8800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01000101;
            rust_out = 4'b0101;
            #1;
            clock_reset = 2'b01;
            i = 8'b01010101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 267 at time 8851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01010101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 268 at time 8900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01010101;
            rust_out = 4'b0101;
            #1;
            clock_reset = 2'b01;
            i = 8'b01100101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 270 at time 8951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01100101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 271 at time 9000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01100101;
            rust_out = 4'b0101;
            #1;
            clock_reset = 2'b01;
            i = 8'b01110101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 273 at time 9051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01110101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 274 at time 9100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01110101;
            rust_out = 4'b0101;
            #1;
            clock_reset = 2'b01;
            i = 8'b10000101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 276 at time 9151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10000101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 277 at time 9200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10000101;
            rust_out = 4'b0101;
            #1;
            clock_reset = 2'b01;
            i = 8'b10010101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 279 at time 9251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10010101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 280 at time 9300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10010101;
            rust_out = 4'b0101;
            #1;
            clock_reset = 2'b01;
            i = 8'b10100101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 282 at time 9351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10100101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 283 at time 9400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10100101;
            rust_out = 4'b0101;
            #1;
            clock_reset = 2'b01;
            i = 8'b10110101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 285 at time 9451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10110101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 286 at time 9500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10110101;
            rust_out = 4'b0101;
            #1;
            clock_reset = 2'b01;
            i = 8'b11000101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 288 at time 9551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11000101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 289 at time 9600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11000101;
            rust_out = 4'b0101;
            #1;
            clock_reset = 2'b01;
            i = 8'b11010101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 291 at time 9651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11010101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 292 at time 9700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11010101;
            rust_out = 4'b0101;
            #1;
            clock_reset = 2'b01;
            i = 8'b11100101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 294 at time 9751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11100101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 295 at time 9800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11100101;
            rust_out = 4'b0101;
            #1;
            clock_reset = 2'b01;
            i = 8'b11110101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 297 at time 9851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11110101;
            rust_out = 4'b0101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 298 at time 9900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11110101;
            rust_out = 4'b0101;
            #1;
            clock_reset = 2'b01;
            i = 8'b00000110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 300 at time 9951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00000110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 301 at time 10000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00000110;
            rust_out = 4'b0110;
            #1;
            clock_reset = 2'b01;
            i = 8'b00010110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 303 at time 10051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00010110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 304 at time 10100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00010110;
            rust_out = 4'b0110;
            #1;
            clock_reset = 2'b01;
            i = 8'b00100110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 306 at time 10151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00100110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 307 at time 10200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00100110;
            rust_out = 4'b0110;
            #1;
            clock_reset = 2'b01;
            i = 8'b00110110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 309 at time 10251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00110110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 310 at time 10300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00110110;
            rust_out = 4'b0110;
            #1;
            clock_reset = 2'b01;
            i = 8'b01000110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 312 at time 10351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01000110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 313 at time 10400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01000110;
            rust_out = 4'b0110;
            #1;
            clock_reset = 2'b01;
            i = 8'b01010110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 315 at time 10451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01010110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 316 at time 10500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01010110;
            rust_out = 4'b0110;
            #1;
            clock_reset = 2'b01;
            i = 8'b01100110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 318 at time 10551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01100110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 319 at time 10600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01100110;
            rust_out = 4'b0110;
            #1;
            clock_reset = 2'b01;
            i = 8'b01110110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 321 at time 10651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01110110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 322 at time 10700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01110110;
            rust_out = 4'b0110;
            #1;
            clock_reset = 2'b01;
            i = 8'b10000110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 324 at time 10751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10000110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 325 at time 10800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10000110;
            rust_out = 4'b0110;
            #1;
            clock_reset = 2'b01;
            i = 8'b10010110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 327 at time 10851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10010110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 328 at time 10900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10010110;
            rust_out = 4'b0110;
            #1;
            clock_reset = 2'b01;
            i = 8'b10100110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 330 at time 10951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10100110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 331 at time 11000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10100110;
            rust_out = 4'b0110;
            #1;
            clock_reset = 2'b01;
            i = 8'b10110110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 333 at time 11051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10110110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 334 at time 11100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10110110;
            rust_out = 4'b0110;
            #1;
            clock_reset = 2'b01;
            i = 8'b11000110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 336 at time 11151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11000110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 337 at time 11200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11000110;
            rust_out = 4'b0110;
            #1;
            clock_reset = 2'b01;
            i = 8'b11010110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 339 at time 11251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11010110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 340 at time 11300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11010110;
            rust_out = 4'b0110;
            #1;
            clock_reset = 2'b01;
            i = 8'b11100110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 342 at time 11351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11100110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 343 at time 11400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11100110;
            rust_out = 4'b0110;
            #1;
            clock_reset = 2'b01;
            i = 8'b11110110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 345 at time 11451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11110110;
            rust_out = 4'b0110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 346 at time 11500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11110110;
            rust_out = 4'b0110;
            #1;
            clock_reset = 2'b01;
            i = 8'b00000111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 348 at time 11551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00000111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 349 at time 11600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00000111;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b01;
            i = 8'b00010111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 351 at time 11651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00010111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 352 at time 11700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00010111;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b01;
            i = 8'b00100111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 354 at time 11751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00100111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 355 at time 11800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00100111;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b01;
            i = 8'b00110111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 357 at time 11851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00110111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 358 at time 11900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00110111;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b01;
            i = 8'b01000111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 360 at time 11951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01000111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 361 at time 12000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01000111;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b01;
            i = 8'b01010111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 363 at time 12051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01010111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 364 at time 12100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01010111;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b01;
            i = 8'b01100111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 366 at time 12151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01100111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 367 at time 12200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01100111;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b01;
            i = 8'b01110111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 369 at time 12251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01110111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 370 at time 12300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01110111;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b01;
            i = 8'b10000111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 372 at time 12351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10000111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 373 at time 12400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10000111;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b01;
            i = 8'b10010111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 375 at time 12451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10010111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 376 at time 12500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10010111;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b01;
            i = 8'b10100111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 378 at time 12551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10100111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 379 at time 12600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10100111;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b01;
            i = 8'b10110111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 381 at time 12651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10110111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 382 at time 12700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10110111;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b01;
            i = 8'b11000111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 384 at time 12751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11000111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 385 at time 12800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11000111;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b01;
            i = 8'b11010111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 387 at time 12851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11010111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 388 at time 12900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11010111;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b01;
            i = 8'b11100111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 390 at time 12951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11100111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 391 at time 13000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11100111;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b01;
            i = 8'b11110111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 393 at time 13051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11110111;
            rust_out = 4'b0111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 394 at time 13100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11110111;
            rust_out = 4'b0111;
            #1;
            clock_reset = 2'b01;
            i = 8'b00001000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 396 at time 13151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00001000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 397 at time 13200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00001000;
            rust_out = 4'b1000;
            #1;
            clock_reset = 2'b01;
            i = 8'b00011000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 399 at time 13251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00011000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 400 at time 13300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00011000;
            rust_out = 4'b1000;
            #1;
            clock_reset = 2'b01;
            i = 8'b00101000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 402 at time 13351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00101000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 403 at time 13400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00101000;
            rust_out = 4'b1000;
            #1;
            clock_reset = 2'b01;
            i = 8'b00111000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 405 at time 13451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00111000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 406 at time 13500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00111000;
            rust_out = 4'b1000;
            #1;
            clock_reset = 2'b01;
            i = 8'b01001000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 408 at time 13551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01001000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 409 at time 13600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01001000;
            rust_out = 4'b1000;
            #1;
            clock_reset = 2'b01;
            i = 8'b01011000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 411 at time 13651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01011000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 412 at time 13700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01011000;
            rust_out = 4'b1000;
            #1;
            clock_reset = 2'b01;
            i = 8'b01101000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 414 at time 13751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01101000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 415 at time 13800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01101000;
            rust_out = 4'b1000;
            #1;
            clock_reset = 2'b01;
            i = 8'b01111000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 417 at time 13851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01111000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 418 at time 13900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01111000;
            rust_out = 4'b1000;
            #1;
            clock_reset = 2'b01;
            i = 8'b10001000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 420 at time 13951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10001000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 421 at time 14000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10001000;
            rust_out = 4'b1000;
            #1;
            clock_reset = 2'b01;
            i = 8'b10011000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 423 at time 14051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10011000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 424 at time 14100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10011000;
            rust_out = 4'b1000;
            #1;
            clock_reset = 2'b01;
            i = 8'b10101000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 426 at time 14151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10101000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 427 at time 14200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10101000;
            rust_out = 4'b1000;
            #1;
            clock_reset = 2'b01;
            i = 8'b10111000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 429 at time 14251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10111000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 430 at time 14300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10111000;
            rust_out = 4'b1000;
            #1;
            clock_reset = 2'b01;
            i = 8'b11001000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 432 at time 14351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11001000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 433 at time 14400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11001000;
            rust_out = 4'b1000;
            #1;
            clock_reset = 2'b01;
            i = 8'b11011000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 435 at time 14451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11011000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 436 at time 14500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11011000;
            rust_out = 4'b1000;
            #1;
            clock_reset = 2'b01;
            i = 8'b11101000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 438 at time 14551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11101000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 439 at time 14600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11101000;
            rust_out = 4'b1000;
            #1;
            clock_reset = 2'b01;
            i = 8'b11111000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 441 at time 14651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11111000;
            rust_out = 4'b1000;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 442 at time 14700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11111000;
            rust_out = 4'b1000;
            #1;
            clock_reset = 2'b01;
            i = 8'b00001001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 444 at time 14751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00001001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 445 at time 14800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00001001;
            rust_out = 4'b1001;
            #1;
            clock_reset = 2'b01;
            i = 8'b00011001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 447 at time 14851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00011001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 448 at time 14900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00011001;
            rust_out = 4'b1001;
            #1;
            clock_reset = 2'b01;
            i = 8'b00101001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 450 at time 14951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00101001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 451 at time 15000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00101001;
            rust_out = 4'b1001;
            #1;
            clock_reset = 2'b01;
            i = 8'b00111001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 453 at time 15051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00111001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 454 at time 15100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00111001;
            rust_out = 4'b1001;
            #1;
            clock_reset = 2'b01;
            i = 8'b01001001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 456 at time 15151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01001001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 457 at time 15200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01001001;
            rust_out = 4'b1001;
            #1;
            clock_reset = 2'b01;
            i = 8'b01011001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 459 at time 15251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01011001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 460 at time 15300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01011001;
            rust_out = 4'b1001;
            #1;
            clock_reset = 2'b01;
            i = 8'b01101001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 462 at time 15351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01101001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 463 at time 15400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01101001;
            rust_out = 4'b1001;
            #1;
            clock_reset = 2'b01;
            i = 8'b01111001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 465 at time 15451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01111001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 466 at time 15500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01111001;
            rust_out = 4'b1001;
            #1;
            clock_reset = 2'b01;
            i = 8'b10001001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 468 at time 15551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10001001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 469 at time 15600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10001001;
            rust_out = 4'b1001;
            #1;
            clock_reset = 2'b01;
            i = 8'b10011001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 471 at time 15651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10011001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 472 at time 15700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10011001;
            rust_out = 4'b1001;
            #1;
            clock_reset = 2'b01;
            i = 8'b10101001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 474 at time 15751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10101001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 475 at time 15800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10101001;
            rust_out = 4'b1001;
            #1;
            clock_reset = 2'b01;
            i = 8'b10111001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 477 at time 15851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10111001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 478 at time 15900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10111001;
            rust_out = 4'b1001;
            #1;
            clock_reset = 2'b01;
            i = 8'b11001001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 480 at time 15951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11001001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 481 at time 16000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11001001;
            rust_out = 4'b1001;
            #1;
            clock_reset = 2'b01;
            i = 8'b11011001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 483 at time 16051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11011001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 484 at time 16100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11011001;
            rust_out = 4'b1001;
            #1;
            clock_reset = 2'b01;
            i = 8'b11101001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 486 at time 16151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11101001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 487 at time 16200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11101001;
            rust_out = 4'b1001;
            #1;
            clock_reset = 2'b01;
            i = 8'b11111001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 489 at time 16251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11111001;
            rust_out = 4'b1001;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 490 at time 16300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11111001;
            rust_out = 4'b1001;
            #1;
            clock_reset = 2'b01;
            i = 8'b00001010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 492 at time 16351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00001010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 493 at time 16400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00001010;
            rust_out = 4'b1010;
            #1;
            clock_reset = 2'b01;
            i = 8'b00011010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 495 at time 16451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00011010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 496 at time 16500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00011010;
            rust_out = 4'b1010;
            #1;
            clock_reset = 2'b01;
            i = 8'b00101010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 498 at time 16551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00101010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 499 at time 16600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00101010;
            rust_out = 4'b1010;
            #1;
            clock_reset = 2'b01;
            i = 8'b00111010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 501 at time 16651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00111010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 502 at time 16700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00111010;
            rust_out = 4'b1010;
            #1;
            clock_reset = 2'b01;
            i = 8'b01001010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 504 at time 16751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01001010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 505 at time 16800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01001010;
            rust_out = 4'b1010;
            #1;
            clock_reset = 2'b01;
            i = 8'b01011010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 507 at time 16851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01011010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 508 at time 16900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01011010;
            rust_out = 4'b1010;
            #1;
            clock_reset = 2'b01;
            i = 8'b01101010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 510 at time 16951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01101010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 511 at time 17000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01101010;
            rust_out = 4'b1010;
            #1;
            clock_reset = 2'b01;
            i = 8'b01111010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 513 at time 17051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01111010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 514 at time 17100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01111010;
            rust_out = 4'b1010;
            #1;
            clock_reset = 2'b01;
            i = 8'b10001010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 516 at time 17151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10001010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 517 at time 17200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10001010;
            rust_out = 4'b1010;
            #1;
            clock_reset = 2'b01;
            i = 8'b10011010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 519 at time 17251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10011010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 520 at time 17300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10011010;
            rust_out = 4'b1010;
            #1;
            clock_reset = 2'b01;
            i = 8'b10101010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 522 at time 17351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10101010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 523 at time 17400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10101010;
            rust_out = 4'b1010;
            #1;
            clock_reset = 2'b01;
            i = 8'b10111010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 525 at time 17451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10111010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 526 at time 17500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10111010;
            rust_out = 4'b1010;
            #1;
            clock_reset = 2'b01;
            i = 8'b11001010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 528 at time 17551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11001010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 529 at time 17600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11001010;
            rust_out = 4'b1010;
            #1;
            clock_reset = 2'b01;
            i = 8'b11011010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 531 at time 17651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11011010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 532 at time 17700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11011010;
            rust_out = 4'b1010;
            #1;
            clock_reset = 2'b01;
            i = 8'b11101010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 534 at time 17751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11101010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 535 at time 17800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11101010;
            rust_out = 4'b1010;
            #1;
            clock_reset = 2'b01;
            i = 8'b11111010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 537 at time 17851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11111010;
            rust_out = 4'b1010;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 538 at time 17900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11111010;
            rust_out = 4'b1010;
            #1;
            clock_reset = 2'b01;
            i = 8'b00001011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 540 at time 17951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00001011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 541 at time 18000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00001011;
            rust_out = 4'b1011;
            #1;
            clock_reset = 2'b01;
            i = 8'b00011011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 543 at time 18051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00011011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 544 at time 18100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00011011;
            rust_out = 4'b1011;
            #1;
            clock_reset = 2'b01;
            i = 8'b00101011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 546 at time 18151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00101011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 547 at time 18200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00101011;
            rust_out = 4'b1011;
            #1;
            clock_reset = 2'b01;
            i = 8'b00111011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 549 at time 18251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00111011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 550 at time 18300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00111011;
            rust_out = 4'b1011;
            #1;
            clock_reset = 2'b01;
            i = 8'b01001011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 552 at time 18351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01001011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 553 at time 18400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01001011;
            rust_out = 4'b1011;
            #1;
            clock_reset = 2'b01;
            i = 8'b01011011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 555 at time 18451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01011011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 556 at time 18500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01011011;
            rust_out = 4'b1011;
            #1;
            clock_reset = 2'b01;
            i = 8'b01101011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 558 at time 18551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01101011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 559 at time 18600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01101011;
            rust_out = 4'b1011;
            #1;
            clock_reset = 2'b01;
            i = 8'b01111011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 561 at time 18651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01111011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 562 at time 18700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01111011;
            rust_out = 4'b1011;
            #1;
            clock_reset = 2'b01;
            i = 8'b10001011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 564 at time 18751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10001011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 565 at time 18800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10001011;
            rust_out = 4'b1011;
            #1;
            clock_reset = 2'b01;
            i = 8'b10011011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 567 at time 18851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10011011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 568 at time 18900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10011011;
            rust_out = 4'b1011;
            #1;
            clock_reset = 2'b01;
            i = 8'b10101011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 570 at time 18951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10101011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 571 at time 19000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10101011;
            rust_out = 4'b1011;
            #1;
            clock_reset = 2'b01;
            i = 8'b10111011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 573 at time 19051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10111011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 574 at time 19100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10111011;
            rust_out = 4'b1011;
            #1;
            clock_reset = 2'b01;
            i = 8'b11001011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 576 at time 19151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11001011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 577 at time 19200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11001011;
            rust_out = 4'b1011;
            #1;
            clock_reset = 2'b01;
            i = 8'b11011011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 579 at time 19251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11011011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 580 at time 19300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11011011;
            rust_out = 4'b1011;
            #1;
            clock_reset = 2'b01;
            i = 8'b11101011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 582 at time 19351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11101011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 583 at time 19400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11101011;
            rust_out = 4'b1011;
            #1;
            clock_reset = 2'b01;
            i = 8'b11111011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 585 at time 19451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11111011;
            rust_out = 4'b1011;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 586 at time 19500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11111011;
            rust_out = 4'b1011;
            #1;
            clock_reset = 2'b01;
            i = 8'b00001100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 588 at time 19551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00001100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 589 at time 19600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00001100;
            rust_out = 4'b1100;
            #1;
            clock_reset = 2'b01;
            i = 8'b00011100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 591 at time 19651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00011100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 592 at time 19700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00011100;
            rust_out = 4'b1100;
            #1;
            clock_reset = 2'b01;
            i = 8'b00101100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 594 at time 19751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00101100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 595 at time 19800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00101100;
            rust_out = 4'b1100;
            #1;
            clock_reset = 2'b01;
            i = 8'b00111100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 597 at time 19851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00111100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 598 at time 19900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00111100;
            rust_out = 4'b1100;
            #1;
            clock_reset = 2'b01;
            i = 8'b01001100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 600 at time 19951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01001100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 601 at time 20000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01001100;
            rust_out = 4'b1100;
            #1;
            clock_reset = 2'b01;
            i = 8'b01011100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 603 at time 20051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01011100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 604 at time 20100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01011100;
            rust_out = 4'b1100;
            #1;
            clock_reset = 2'b01;
            i = 8'b01101100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 606 at time 20151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01101100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 607 at time 20200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01101100;
            rust_out = 4'b1100;
            #1;
            clock_reset = 2'b01;
            i = 8'b01111100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 609 at time 20251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01111100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 610 at time 20300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01111100;
            rust_out = 4'b1100;
            #1;
            clock_reset = 2'b01;
            i = 8'b10001100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 612 at time 20351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10001100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 613 at time 20400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10001100;
            rust_out = 4'b1100;
            #1;
            clock_reset = 2'b01;
            i = 8'b10011100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 615 at time 20451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10011100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 616 at time 20500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10011100;
            rust_out = 4'b1100;
            #1;
            clock_reset = 2'b01;
            i = 8'b10101100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 618 at time 20551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10101100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 619 at time 20600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10101100;
            rust_out = 4'b1100;
            #1;
            clock_reset = 2'b01;
            i = 8'b10111100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 621 at time 20651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10111100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 622 at time 20700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10111100;
            rust_out = 4'b1100;
            #1;
            clock_reset = 2'b01;
            i = 8'b11001100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 624 at time 20751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11001100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 625 at time 20800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11001100;
            rust_out = 4'b1100;
            #1;
            clock_reset = 2'b01;
            i = 8'b11011100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 627 at time 20851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11011100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 628 at time 20900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11011100;
            rust_out = 4'b1100;
            #1;
            clock_reset = 2'b01;
            i = 8'b11101100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 630 at time 20951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11101100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 631 at time 21000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11101100;
            rust_out = 4'b1100;
            #1;
            clock_reset = 2'b01;
            i = 8'b11111100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 633 at time 21051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11111100;
            rust_out = 4'b1100;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 634 at time 21100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11111100;
            rust_out = 4'b1100;
            #1;
            clock_reset = 2'b01;
            i = 8'b00001101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 636 at time 21151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00001101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 637 at time 21200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00001101;
            rust_out = 4'b1101;
            #1;
            clock_reset = 2'b01;
            i = 8'b00011101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 639 at time 21251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00011101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 640 at time 21300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00011101;
            rust_out = 4'b1101;
            #1;
            clock_reset = 2'b01;
            i = 8'b00101101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 642 at time 21351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00101101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 643 at time 21400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00101101;
            rust_out = 4'b1101;
            #1;
            clock_reset = 2'b01;
            i = 8'b00111101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 645 at time 21451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00111101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 646 at time 21500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00111101;
            rust_out = 4'b1101;
            #1;
            clock_reset = 2'b01;
            i = 8'b01001101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 648 at time 21551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01001101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 649 at time 21600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01001101;
            rust_out = 4'b1101;
            #1;
            clock_reset = 2'b01;
            i = 8'b01011101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 651 at time 21651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01011101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 652 at time 21700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01011101;
            rust_out = 4'b1101;
            #1;
            clock_reset = 2'b01;
            i = 8'b01101101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 654 at time 21751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01101101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 655 at time 21800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01101101;
            rust_out = 4'b1101;
            #1;
            clock_reset = 2'b01;
            i = 8'b01111101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 657 at time 21851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01111101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 658 at time 21900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01111101;
            rust_out = 4'b1101;
            #1;
            clock_reset = 2'b01;
            i = 8'b10001101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 660 at time 21951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10001101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 661 at time 22000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10001101;
            rust_out = 4'b1101;
            #1;
            clock_reset = 2'b01;
            i = 8'b10011101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 663 at time 22051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10011101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 664 at time 22100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10011101;
            rust_out = 4'b1101;
            #1;
            clock_reset = 2'b01;
            i = 8'b10101101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 666 at time 22151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10101101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 667 at time 22200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10101101;
            rust_out = 4'b1101;
            #1;
            clock_reset = 2'b01;
            i = 8'b10111101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 669 at time 22251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10111101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 670 at time 22300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10111101;
            rust_out = 4'b1101;
            #1;
            clock_reset = 2'b01;
            i = 8'b11001101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 672 at time 22351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11001101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 673 at time 22400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11001101;
            rust_out = 4'b1101;
            #1;
            clock_reset = 2'b01;
            i = 8'b11011101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 675 at time 22451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11011101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 676 at time 22500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11011101;
            rust_out = 4'b1101;
            #1;
            clock_reset = 2'b01;
            i = 8'b11101101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 678 at time 22551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11101101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 679 at time 22600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11101101;
            rust_out = 4'b1101;
            #1;
            clock_reset = 2'b01;
            i = 8'b11111101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 681 at time 22651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11111101;
            rust_out = 4'b1101;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 682 at time 22700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11111101;
            rust_out = 4'b1101;
            #1;
            clock_reset = 2'b01;
            i = 8'b00001110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 684 at time 22751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00001110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 685 at time 22800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00001110;
            rust_out = 4'b1110;
            #1;
            clock_reset = 2'b01;
            i = 8'b00011110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 687 at time 22851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00011110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 688 at time 22900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00011110;
            rust_out = 4'b1110;
            #1;
            clock_reset = 2'b01;
            i = 8'b00101110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 690 at time 22951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00101110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 691 at time 23000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00101110;
            rust_out = 4'b1110;
            #1;
            clock_reset = 2'b01;
            i = 8'b00111110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 693 at time 23051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00111110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 694 at time 23100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00111110;
            rust_out = 4'b1110;
            #1;
            clock_reset = 2'b01;
            i = 8'b01001110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 696 at time 23151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01001110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 697 at time 23200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01001110;
            rust_out = 4'b1110;
            #1;
            clock_reset = 2'b01;
            i = 8'b01011110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 699 at time 23251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01011110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 700 at time 23300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01011110;
            rust_out = 4'b1110;
            #1;
            clock_reset = 2'b01;
            i = 8'b01101110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 702 at time 23351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01101110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 703 at time 23400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01101110;
            rust_out = 4'b1110;
            #1;
            clock_reset = 2'b01;
            i = 8'b01111110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 705 at time 23451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01111110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 706 at time 23500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01111110;
            rust_out = 4'b1110;
            #1;
            clock_reset = 2'b01;
            i = 8'b10001110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 708 at time 23551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10001110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 709 at time 23600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10001110;
            rust_out = 4'b1110;
            #1;
            clock_reset = 2'b01;
            i = 8'b10011110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 711 at time 23651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10011110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 712 at time 23700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10011110;
            rust_out = 4'b1110;
            #1;
            clock_reset = 2'b01;
            i = 8'b10101110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 714 at time 23751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10101110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 715 at time 23800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10101110;
            rust_out = 4'b1110;
            #1;
            clock_reset = 2'b01;
            i = 8'b10111110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 717 at time 23851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10111110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 718 at time 23900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10111110;
            rust_out = 4'b1110;
            #1;
            clock_reset = 2'b01;
            i = 8'b11001110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 720 at time 23951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11001110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 721 at time 24000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11001110;
            rust_out = 4'b1110;
            #1;
            clock_reset = 2'b01;
            i = 8'b11011110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 723 at time 24051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11011110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 724 at time 24100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11011110;
            rust_out = 4'b1110;
            #1;
            clock_reset = 2'b01;
            i = 8'b11101110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 726 at time 24151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11101110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 727 at time 24200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11101110;
            rust_out = 4'b1110;
            #1;
            clock_reset = 2'b01;
            i = 8'b11111110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 729 at time 24251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11111110;
            rust_out = 4'b1110;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 730 at time 24300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11111110;
            rust_out = 4'b1110;
            #1;
            clock_reset = 2'b01;
            i = 8'b00001111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 732 at time 24351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00001111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 733 at time 24400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00001111;
            rust_out = 4'b1111;
            #1;
            clock_reset = 2'b01;
            i = 8'b00011111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 735 at time 24451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00011111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 736 at time 24500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00011111;
            rust_out = 4'b1111;
            #1;
            clock_reset = 2'b01;
            i = 8'b00101111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 738 at time 24551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00101111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 739 at time 24600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00101111;
            rust_out = 4'b1111;
            #1;
            clock_reset = 2'b01;
            i = 8'b00111111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 741 at time 24651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b00111111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 742 at time 24700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b00111111;
            rust_out = 4'b1111;
            #1;
            clock_reset = 2'b01;
            i = 8'b01001111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 744 at time 24751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01001111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 745 at time 24800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01001111;
            rust_out = 4'b1111;
            #1;
            clock_reset = 2'b01;
            i = 8'b01011111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 747 at time 24851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01011111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 748 at time 24900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01011111;
            rust_out = 4'b1111;
            #1;
            clock_reset = 2'b01;
            i = 8'b01101111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 750 at time 24951", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01101111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 751 at time 25000", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01101111;
            rust_out = 4'b1111;
            #1;
            clock_reset = 2'b01;
            i = 8'b01111111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 753 at time 25051", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b01111111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 754 at time 25100", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b01111111;
            rust_out = 4'b1111;
            #1;
            clock_reset = 2'b01;
            i = 8'b10001111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 756 at time 25151", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10001111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 757 at time 25200", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10001111;
            rust_out = 4'b1111;
            #1;
            clock_reset = 2'b01;
            i = 8'b10011111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 759 at time 25251", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10011111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 760 at time 25300", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10011111;
            rust_out = 4'b1111;
            #1;
            clock_reset = 2'b01;
            i = 8'b10101111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 762 at time 25351", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10101111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 763 at time 25400", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10101111;
            rust_out = 4'b1111;
            #1;
            clock_reset = 2'b01;
            i = 8'b10111111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 765 at time 25451", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b10111111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 766 at time 25500", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b10111111;
            rust_out = 4'b1111;
            #1;
            clock_reset = 2'b01;
            i = 8'b11001111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 768 at time 25551", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11001111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 769 at time 25600", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11001111;
            rust_out = 4'b1111;
            #1;
            clock_reset = 2'b01;
            i = 8'b11011111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 771 at time 25651", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11011111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 772 at time 25700", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11011111;
            rust_out = 4'b1111;
            #1;
            clock_reset = 2'b01;
            i = 8'b11101111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 774 at time 25751", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11101111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 775 at time 25800", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11101111;
            rust_out = 4'b1111;
            #1;
            clock_reset = 2'b01;
            i = 8'b11111111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 777 at time 25851", o, rust_out);
                $finish;
            end
            #48;
            clock_reset = 2'b00;
            i = 8'b11111111;
            rust_out = 4'b1111;
            #1;
            if (o !== rust_out) begin
                $display("ASSERTION FAILED 0x%0h !== 0x%0h CASE Test 778 at time 25900", o, rust_out);
                $finish;
            end
            #49;
            clock_reset = 2'b01;
            i = 8'b11111111;
            rust_out = 4'b1111;
            $display("TESTBENCH OK", );
            $finish;
        end
    endmodule
    // synchronous circuit flow_graph::test_constant_propogation_through_selector_inline::parent::Parent
    module uut(input wire [1:0] clock_reset, input wire [7:0] i, output wire [3:0] o);
        wire [12:0] od;
        wire [8:0] d;
        wire [3:0] q;
        assign o = od[3:0];
        uut_selector c0 (.clock_reset(clock_reset),.i(d[8:0]),.o(q[3:0]));
        assign od = kernel_parent(clock_reset, i, q);
        assign d = od[12:4];
        function [12:0] kernel_parent(input reg [1:0] arg_0, input reg [7:0] arg_1, input reg [3:0] arg_2);
            reg [3:0] or0;
            reg [7:0] or1;
            reg [3:0] or2;
            reg [8:0] or3;
            reg [8:0] or4;  // d
            reg [3:0] or5;
            reg [12:0] or6;
            reg [1:0] or7;
            localparam ol0 = 1'b1;
            localparam ol1 = 9'bxxxxxxxxx;
            begin
                or7 = arg_0;
                or1 = arg_1;
                or5 = arg_2;
                // let (a, b, ) = i;
                //
                or0 = or1[3:0];
                or2 = or1[7:4];
                // let d = D::dont_care();
                //
                // d.selector = (true, a, b, );
                //
                or3 = { or2, or0, ol0 };
                or4 = ol1; or4[8:0] = or3;
                // let o = q.selector;
                //
                // (o, d, )
                //
                or6 = { or4, or5 };
                kernel_parent = or6;
            end
        endfunction
    endmodule
    // synchronous circuit flow_graph::selector::U
    module uut_selector(input wire [1:0] clock_reset, input wire [8:0] i, output wire [3:0] o);
        wire [3:0] od;
        assign o = od[3:0];
        assign od = kernel_selector(clock_reset, i);
        function [3:0] kernel_selector(input reg [1:0] arg_0, input reg [8:0] arg_1);
            reg [0:0] or0;
            reg [8:0] or1;
            reg [3:0] or2;
            reg [3:0] or3;
            reg [3:0] or4;
            reg [1:0] or5;
            begin
                or5 = arg_0;
                or1 = arg_1;
                // let (sel, a, b, ) = i;
                //
                or0 = or1[0];
                or2 = or1[4:1];
                or3 = or1[8:5];
                // let out = if sel {
                //    a
                // }
                //  else {
                //    b
                // }
                // ;
                //
                // a
                //
                // b
                //
                or4 = (or0) ? (or2) : (or3);
                // (out, (), )
                //
                kernel_selector = or4;
            end
        endfunction
    endmodule
