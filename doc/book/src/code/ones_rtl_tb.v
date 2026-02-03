module testbench();
   reg [7:0] i;
   wire [3:0] o;
   reg [3:0] rust_out;
   uut t(.i(i), .o(o));
   initial begin
      #0;
      i = 8'b00000000;
      rust_out = 4'b0000;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 1 at time 100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00000001;
      rust_out = 4'b0001;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 2 at time 200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00000010;
      rust_out = 4'b0001;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 3 at time 300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00000011;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 4 at time 400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00000100;
      rust_out = 4'b0001;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 5 at time 500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00000101;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 6 at time 600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00000110;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 7 at time 700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00000111;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 8 at time 800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00001000;
      rust_out = 4'b0001;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 9 at time 900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00001001;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 10 at time 1000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00001010;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 11 at time 1100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00001011;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 12 at time 1200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00001100;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 13 at time 1300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00001101;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 14 at time 1400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00001110;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 15 at time 1500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00001111;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 16 at time 1600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00010000;
      rust_out = 4'b0001;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 17 at time 1700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00010001;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 18 at time 1800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00010010;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 19 at time 1900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00010011;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 20 at time 2000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00010100;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 21 at time 2100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00010101;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 22 at time 2200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00010110;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 23 at time 2300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00010111;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 24 at time 2400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00011000;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 25 at time 2500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00011001;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 26 at time 2600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00011010;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 27 at time 2700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00011011;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 28 at time 2800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00011100;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 29 at time 2900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00011101;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 30 at time 3000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00011110;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 31 at time 3100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00011111;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 32 at time 3200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00100000;
      rust_out = 4'b0001;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 33 at time 3300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00100001;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 34 at time 3400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00100010;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 35 at time 3500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00100011;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 36 at time 3600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00100100;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 37 at time 3700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00100101;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 38 at time 3800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00100110;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 39 at time 3900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00100111;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 40 at time 4000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00101000;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 41 at time 4100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00101001;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 42 at time 4200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00101010;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 43 at time 4300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00101011;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 44 at time 4400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00101100;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 45 at time 4500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00101101;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 46 at time 4600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00101110;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 47 at time 4700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00101111;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 48 at time 4800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00110000;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 49 at time 4900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00110001;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 50 at time 5000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00110010;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 51 at time 5100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00110011;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 52 at time 5200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00110100;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 53 at time 5300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00110101;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 54 at time 5400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00110110;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 55 at time 5500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00110111;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 56 at time 5600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00111000;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 57 at time 5700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00111001;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 58 at time 5800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00111010;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 59 at time 5900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00111011;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 60 at time 6000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00111100;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 61 at time 6100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00111101;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 62 at time 6200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00111110;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 63 at time 6300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b00111111;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 64 at time 6400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01000000;
      rust_out = 4'b0001;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 65 at time 6500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01000001;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 66 at time 6600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01000010;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 67 at time 6700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01000011;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 68 at time 6800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01000100;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 69 at time 6900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01000101;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 70 at time 7000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01000110;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 71 at time 7100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01000111;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 72 at time 7200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01001000;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 73 at time 7300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01001001;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 74 at time 7400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01001010;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 75 at time 7500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01001011;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 76 at time 7600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01001100;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 77 at time 7700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01001101;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 78 at time 7800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01001110;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 79 at time 7900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01001111;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 80 at time 8000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01010000;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 81 at time 8100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01010001;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 82 at time 8200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01010010;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 83 at time 8300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01010011;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 84 at time 8400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01010100;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 85 at time 8500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01010101;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 86 at time 8600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01010110;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 87 at time 8700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01010111;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 88 at time 8800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01011000;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 89 at time 8900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01011001;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 90 at time 9000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01011010;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 91 at time 9100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01011011;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 92 at time 9200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01011100;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 93 at time 9300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01011101;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 94 at time 9400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01011110;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 95 at time 9500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01011111;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 96 at time 9600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01100000;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 97 at time 9700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01100001;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 98 at time 9800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01100010;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 99 at time 9900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01100011;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 100 at time 10000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01100100;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 101 at time 10100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01100101;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 102 at time 10200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01100110;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 103 at time 10300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01100111;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 104 at time 10400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01101000;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 105 at time 10500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01101001;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 106 at time 10600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01101010;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 107 at time 10700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01101011;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 108 at time 10800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01101100;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 109 at time 10900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01101101;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 110 at time 11000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01101110;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 111 at time 11100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01101111;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 112 at time 11200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01110000;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 113 at time 11300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01110001;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 114 at time 11400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01110010;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 115 at time 11500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01110011;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 116 at time 11600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01110100;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 117 at time 11700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01110101;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 118 at time 11800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01110110;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 119 at time 11900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01110111;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 120 at time 12000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01111000;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 121 at time 12100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01111001;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 122 at time 12200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01111010;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 123 at time 12300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01111011;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 124 at time 12400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01111100;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 125 at time 12500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01111101;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 126 at time 12600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01111110;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 127 at time 12700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b01111111;
      rust_out = 4'b0111;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 128 at time 12800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10000000;
      rust_out = 4'b0001;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 129 at time 12900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10000001;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 130 at time 13000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10000010;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 131 at time 13100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10000011;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 132 at time 13200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10000100;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 133 at time 13300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10000101;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 134 at time 13400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10000110;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 135 at time 13500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10000111;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 136 at time 13600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10001000;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 137 at time 13700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10001001;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 138 at time 13800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10001010;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 139 at time 13900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10001011;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 140 at time 14000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10001100;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 141 at time 14100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10001101;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 142 at time 14200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10001110;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 143 at time 14300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10001111;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 144 at time 14400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10010000;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 145 at time 14500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10010001;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 146 at time 14600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10010010;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 147 at time 14700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10010011;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 148 at time 14800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10010100;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 149 at time 14900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10010101;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 150 at time 15000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10010110;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 151 at time 15100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10010111;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 152 at time 15200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10011000;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 153 at time 15300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10011001;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 154 at time 15400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10011010;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 155 at time 15500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10011011;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 156 at time 15600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10011100;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 157 at time 15700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10011101;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 158 at time 15800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10011110;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 159 at time 15900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10011111;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 160 at time 16000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10100000;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 161 at time 16100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10100001;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 162 at time 16200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10100010;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 163 at time 16300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10100011;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 164 at time 16400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10100100;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 165 at time 16500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10100101;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 166 at time 16600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10100110;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 167 at time 16700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10100111;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 168 at time 16800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10101000;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 169 at time 16900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10101001;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 170 at time 17000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10101010;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 171 at time 17100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10101011;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 172 at time 17200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10101100;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 173 at time 17300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10101101;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 174 at time 17400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10101110;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 175 at time 17500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10101111;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 176 at time 17600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10110000;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 177 at time 17700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10110001;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 178 at time 17800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10110010;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 179 at time 17900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10110011;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 180 at time 18000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10110100;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 181 at time 18100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10110101;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 182 at time 18200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10110110;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 183 at time 18300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10110111;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 184 at time 18400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10111000;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 185 at time 18500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10111001;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 186 at time 18600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10111010;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 187 at time 18700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10111011;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 188 at time 18800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10111100;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 189 at time 18900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10111101;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 190 at time 19000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10111110;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 191 at time 19100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b10111111;
      rust_out = 4'b0111;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 192 at time 19200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11000000;
      rust_out = 4'b0010;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 193 at time 19300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11000001;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 194 at time 19400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11000010;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 195 at time 19500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11000011;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 196 at time 19600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11000100;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 197 at time 19700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11000101;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 198 at time 19800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11000110;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 199 at time 19900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11000111;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 200 at time 20000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11001000;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 201 at time 20100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11001001;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 202 at time 20200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11001010;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 203 at time 20300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11001011;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 204 at time 20400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11001100;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 205 at time 20500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11001101;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 206 at time 20600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11001110;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 207 at time 20700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11001111;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 208 at time 20800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11010000;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 209 at time 20900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11010001;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 210 at time 21000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11010010;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 211 at time 21100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11010011;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 212 at time 21200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11010100;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 213 at time 21300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11010101;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 214 at time 21400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11010110;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 215 at time 21500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11010111;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 216 at time 21600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11011000;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 217 at time 21700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11011001;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 218 at time 21800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11011010;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 219 at time 21900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11011011;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 220 at time 22000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11011100;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 221 at time 22100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11011101;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 222 at time 22200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11011110;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 223 at time 22300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11011111;
      rust_out = 4'b0111;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 224 at time 22400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11100000;
      rust_out = 4'b0011;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 225 at time 22500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11100001;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 226 at time 22600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11100010;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 227 at time 22700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11100011;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 228 at time 22800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11100100;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 229 at time 22900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11100101;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 230 at time 23000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11100110;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 231 at time 23100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11100111;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 232 at time 23200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11101000;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 233 at time 23300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11101001;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 234 at time 23400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11101010;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 235 at time 23500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11101011;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 236 at time 23600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11101100;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 237 at time 23700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11101101;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 238 at time 23800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11101110;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 239 at time 23900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11101111;
      rust_out = 4'b0111;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 240 at time 24000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11110000;
      rust_out = 4'b0100;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 241 at time 24100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11110001;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 242 at time 24200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11110010;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 243 at time 24300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11110011;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 244 at time 24400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11110100;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 245 at time 24500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11110101;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 246 at time 24600", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11110110;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 247 at time 24700", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11110111;
      rust_out = 4'b0111;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 248 at time 24800", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11111000;
      rust_out = 4'b0101;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 249 at time 24900", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11111001;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 250 at time 25000", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11111010;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 251 at time 25100", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11111011;
      rust_out = 4'b0111;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 252 at time 25200", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11111100;
      rust_out = 4'b0110;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 253 at time 25300", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11111101;
      rust_out = 4'b0111;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 254 at time 25400", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11111110;
      rust_out = 4'b0111;
      #1;
      if (o !== rust_out) begin
         $display("TESTBENCH FAILED: Expected %b, got %b -- Test 255 at time 25500", rust_out, o);
         $finish;
      end
      #99;
      i = 8'b11111111;
      rust_out = 4'b1000;
      $display("TESTBENCH OK");
      $finish;
   end
endmodule
module uut(input wire [7:0] i, output wire [3:0] o);
   wire [3:0] od;
   assign o = od[3:0];
   assign od = kernel_one_counter(i);
   function [3:0] kernel_one_counter(input reg [7:0] arg_0);
         reg [7:0] r0;
         reg [7:0] r1;
         reg [0:0] r2;
         // count
         reg [3:0] r3;
         reg [7:0] r4;
         reg [0:0] r5;
         reg [3:0] r6;
         // count
         reg [3:0] r7;
         reg [7:0] r8;
         reg [0:0] r9;
         reg [3:0] r10;
         // count
         reg [3:0] r11;
         reg [7:0] r12;
         reg [0:0] r13;
         reg [3:0] r14;
         // count
         reg [3:0] r15;
         reg [7:0] r16;
         reg [0:0] r17;
         reg [3:0] r18;
         // count
         reg [3:0] r19;
         reg [7:0] r20;
         reg [0:0] r21;
         reg [3:0] r22;
         // count
         reg [3:0] r23;
         reg [7:0] r24;
         reg [0:0] r25;
         reg [3:0] r26;
         // count
         reg [3:0] r27;
         reg [7:0] r28;
         reg [0:0] r29;
         reg [3:0] r30;
         // count
         reg [3:0] r31;
         localparam l0 = 8'b00000001;
         localparam l1 = 4'b0001;
         localparam l2 = 4'b0000;
         localparam l3 = 8'b00000010;
         localparam l4 = 4'b0001;
         localparam l5 = 8'b00000100;
         localparam l6 = 4'b0001;
         localparam l7 = 8'b00001000;
         localparam l8 = 4'b0001;
         localparam l9 = 8'b00010000;
         localparam l10 = 4'b0001;
         localparam l11 = 8'b00100000;
         localparam l12 = 4'b0001;
         localparam l13 = 8'b01000000;
         localparam l14 = 4'b0001;
         localparam l15 = 8'b10000000;
         localparam l16 = 4'b0001;
         begin
            r0 = arg_0;
            r1 = r0 & l0;
            r2 = |r1;
            r3 = r2 ? l1 : l2;
            r4 = r0 & l3;
            r5 = |r4;
            r6 = r3 + l4;
            r7 = r5 ? r6 : r3;
            r8 = r0 & l5;
            r9 = |r8;
            r10 = r7 + l6;
            r11 = r9 ? r10 : r7;
            r12 = r0 & l7;
            r13 = |r12;
            r14 = r11 + l8;
            r15 = r13 ? r14 : r11;
            r16 = r0 & l9;
            r17 = |r16;
            r18 = r15 + l10;
            r19 = r17 ? r18 : r15;
            r20 = r0 & l11;
            r21 = |r20;
            r22 = r19 + l12;
            r23 = r21 ? r22 : r19;
            r24 = r0 & l13;
            r25 = |r24;
            r26 = r23 + l14;
            r27 = r25 ? r26 : r23;
            r28 = r0 & l15;
            r29 = |r28;
            r30 = r27 + l16;
            r31 = r29 ? r30 : r27;
            kernel_one_counter = r31;
         end
   endfunction
endmodule
