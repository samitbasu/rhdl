// synchronous circuit rhdl_fpga::axi4lite::core::controller::blocking::BlockReadWriteController
module top(input wire [1:0] clock_reset, input wire [111:0] i, output wire [140:0] o);
    wire [394:0] od;
    wire [253:0] d;
    wire [252:0] q;
    assign o = od[140:0];
    top_inbuf c0 (.clock_reset(clock_reset),.i(d[70:0]),.o(q[70:0]));
    top_outbuf c1 (.clock_reset(clock_reset),.i(d[251:216]),.o(q[250:215]));
    top_read_controller c2 (.clock_reset(clock_reset),.i(d[215:146]),.o(q[214:146]));
    top_state c3 (.clock_reset(clock_reset),.i(d[253:252]),.o(q[252:251]));
    top_write_controller c4 (.clock_reset(clock_reset),.i(d[145:71]),.o(q[145:71]));
    assign od = kernel_kernel(clock_reset, i, q);
    assign d = od[394:141];
    function [394:0] kernel_kernel(input reg [1:0] arg_0, input reg [111:0] arg_1, input reg [252:0] arg_2);
        reg [1:0] or0;
        reg [252:0] or1;
        reg [253:0] or2;  // d
        reg [69:0] or3;
        reg [111:0] or4;
        reg [253:0] or5;  // d
        reg [35:0] or6;
        reg [0:0] or7;
        reg [253:0] or8;  // d
        reg [253:0] or9;  // d
        reg [253:0] or10;  // d
        reg [1:0] or11;
        reg [68:0] or12;
        reg [33:0] or13;
        reg [0:0] or14;
        reg [32:0] or15;
        reg [33:0] or16;
        reg [34:0] or17;
        reg [33:0] or18;
        reg [253:0] or19;  // d
        reg [253:0] or20;  // d
        reg [253:0] or21;  // d
        reg [0:0] or22;  // will_unload
        reg [253:0] or23;  // d
        reg [0:0] or24;  // will_unload
        reg [74:0] or25;
        reg [2:0] or26;
        reg [0:0] or27;
        reg [1:0] or28;
        reg [33:0] or29;
        reg [34:0] or30;
        reg [33:0] or31;
        reg [253:0] or32;  // d
        reg [253:0] or33;  // d
        reg [253:0] or34;  // d
        reg [0:0] or35;  // will_unload
        reg [253:0] or36;  // d
        reg [0:0] or37;  // will_unload
        reg [253:0] or38;  // d
        reg [0:0] or39;  // will_unload
        reg [74:0] or40;
        reg [0:0] or41;
        reg [68:0] or42;
        reg [0:0] or43;
        reg [0:0] or44;
        reg [1:0] or45;
        reg [0:0] or46;
        reg [0:0] or47;
        reg [0:0] or48;
        reg [70:0] or49;
        reg [69:0] or50;
        reg [0:0] or51;
        reg [0:0] or52;
        reg [0:0] or53;
        reg [4:0] or54;
        reg [253:0] or55;  // d
        reg [253:0] or56;  // d
        reg [35:0] or57;
        reg [253:0] or58;  // d
        reg [253:0] or59;  // d
        reg [253:0] or60;  // d
        reg [70:0] or61;
        reg [69:0] or62;
        reg [0:0] or63;
        reg [68:0] or64;
        reg [0:0] or65;
        reg [31:0] or66;
        reg [253:0] or67;  // d
        reg [32:0] or68;
        reg [31:0] or69;
        reg [253:0] or70;  // d
        reg [67:0] or71;
        reg [253:0] or72;  // d
        reg [68:0] or73;
        reg [67:0] or74;
        reg [253:0] or75;  // d
        reg [253:0] or76;  // d
        reg [253:0] or77;  // d
        reg [253:0] or78;  // d
        reg [0:0] or79;
        reg [253:0] or80;  // d
        reg [68:0] or81;
        reg [33:0] or82;
        reg [140:0] or83;  // o
        reg [70:0] or84;
        reg [0:0] or85;
        reg [140:0] or86;  // o
        reg [35:0] or87;
        reg [34:0] or88;
        reg [140:0] or89;  // o
        reg [74:0] or90;
        reg [70:0] or91;
        reg [140:0] or92;  // o
        reg [394:0] or93;
        reg [1:0] or94;
        localparam ol0 = 254'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol1 = 35'b00000000000000000000000000000000000;
        localparam ol2 = 34'b1000000000000000000000000000000000;
        localparam ol3 = 1'b1;
        localparam ol4 = 2'b00;
        localparam ol5 = 1'b1;
        localparam ol6 = 1'b0;
        localparam ol7 = 1'b1;
        localparam ol8 = 34'b0000000000000000000000000000000000;
        localparam ol9 = 1'b1;
        localparam ol10 = 2'b00;
        localparam ol11 = 1'b1;
        localparam ol12 = 1'b1;
        localparam ol13 = 2'b00;
        localparam ol14 = 2'b10;
        localparam ol15 = 2'b01;
        localparam ol16 = 2'b00;
        localparam ol17 = 1'b1;
        localparam ol18 = 1'b1;
        localparam ol19 = 1'b0;
        localparam ol20 = 1'b0;
        localparam ol21 = 69'b000000000000000000000000000000000000000000000000000000000000000000000;
        localparam ol22 = 33'b000000000000000000000000000000000;
        localparam ol23 = 2'b10;
        localparam ol24 = 1'b1;
        localparam ol25 = 2'b01;
        localparam ol26 = 1'b1;
        localparam ol27 = 1'b1;
        localparam ol28 = 1'b0;
        localparam ol29 = 1'b1;
        localparam ol30 = 141'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        begin
            or94 = arg_0;
            or4 = arg_1;
            or1 = arg_2;
            // let d = D::dont_care();
            //
            // let o = Out::dont_care();
            //
            // d.state = q.state;
            //
            or0 = or1[252:251];
            or2 = ol0; or2[253:252] = or0;
            // d.inbuf.data = i.request;
            //
            or3 = or4[69:0];
            or5 = or2; or5[69:0] = or3;
            // let will_unload = false;
            //
            // let can_accept = q.outbuf.ready.raw;
            //
            or6 = or1[250:215];
            or7 = or6[35];
            // d.read_controller.resp_ready.raw = can_accept;
            //
            or8 = or5; or8[215:215] = or7;
            // d.write_controller.resp_ready.raw = can_accept;
            //
            or9 = or8; or9[145:145] = or7;
            // d.outbuf.data = None();
            //
            or10 = or9; or10[250:216] = ol1;
            // match q.state {
            //    const State::Idle => {
            //    }
            //    ,
            //    const State::Reading => {
            //       if let Some(resp, )#true = q.read_controller.resp_data{
            //          d.outbuf.data = Some(BlockResponse::Read(resp));
            //          if can_accept {
            //             d.state = State :: Idle;
            //             will_unload = true;
            //          }
            //
            //       }
            //
            //    }
            //    ,
            //    const State::Writing => {
            //       if let Some(resp, )#true = q.write_controller.resp_data{
            //          d.outbuf.data = Some(BlockResponse::Write(resp));
            //          if can_accept {
            //             d.state = State :: Idle;
            //             will_unload = true;
            //          }
            //
            //       }
            //
            //    }
            //    ,
            // }
            //
            or11 = or1[252:251];
            // if let Some(resp, )#true = q.read_controller.resp_data{
            //    d.outbuf.data = Some(BlockResponse::Read(resp));
            //    if can_accept {
            //       d.state = State :: Idle;
            //       will_unload = true;
            //    }
            //
            // }
            //
            //
            or12 = or1[214:146];
            or13 = or12[68:35];
            or14 = or13[33];
            or15 = or13[32:0];
            // d.outbuf.data = Some(BlockResponse::Read(resp));
            //
            or16 = ol2; or16[32:0] = or15;
            or18 = or16[33:0];
            or17 = { ol3, or18 };
            or19 = or10; or19[250:216] = or17;
            // if can_accept {
            //    d.state = State :: Idle;
            //    will_unload = true;
            // }
            //
            //
            // d.state = State :: Idle;
            //
            or20 = or19; or20[253:252] = ol4;
            // will_unload = true;
            //
            or21 = (or7) ? (or20) : (or19);
            or22 = (or7) ? (ol5) : (ol6);
            case (or14)
                1'b1: or23 = or21;
                default: or23 = or10;
            endcase
            case (or14)
                1'b1: or24 = or22;
                default: or24 = ol6;
            endcase
            // if let Some(resp, )#true = q.write_controller.resp_data{
            //    d.outbuf.data = Some(BlockResponse::Write(resp));
            //    if can_accept {
            //       d.state = State :: Idle;
            //       will_unload = true;
            //    }
            //
            // }
            //
            //
            or25 = or1[145:71];
            or26 = or25[74:72];
            or27 = or26[2];
            or28 = or26[1:0];
            // d.outbuf.data = Some(BlockResponse::Write(resp));
            //
            or29 = ol8; or29[1:0] = or28;
            or31 = or29[33:0];
            or30 = { ol9, or31 };
            or32 = or10; or32[250:216] = or30;
            // if can_accept {
            //    d.state = State :: Idle;
            //    will_unload = true;
            // }
            //
            //
            // d.state = State :: Idle;
            //
            or33 = or32; or33[253:252] = ol10;
            // will_unload = true;
            //
            or34 = (or7) ? (or33) : (or32);
            or35 = (or7) ? (ol11) : (ol6);
            case (or27)
                1'b1: or36 = or34;
                default: or36 = or10;
            endcase
            case (or27)
                1'b1: or37 = or35;
                default: or37 = ol6;
            endcase
            case (or11)
                2'b00: or38 = or10;
                2'b10: or38 = or23;
                2'b01: or38 = or36;
            endcase
            case (or11)
                2'b00: or39 = ol6;
                2'b10: or39 = or24;
                2'b01: or39 = or37;
            endcase
            // let will_start = q.write_controller.req_ready.raw & q.read_controller.req_ready.raw & ((q.state == State :: Idle) | will_unload) & is_some(q.inbuf.data);
            //
            or40 = or1[145:71];
            or41 = or40[71];
            or42 = or1[214:146];
            or43 = or42[34];
            or44 = or41 & or43;
            or45 = or1[252:251];
            or46 = or45 == ol16;
            or47 = or46 | or39;
            or48 = or44 & or47;
            or49 = or1[70:0];
            or50 = or49[69:0];
            // match x {
            //    Some(_, )#true => true,
            //    _#false => false,
            // }
            //
            or51 = or50[69];
            case (or51)
                1'b1: or52 = ol18;
                1'b0: or52 = ol20;
            endcase
            or53 = or48 & or52;
            // d.write_controller.axi = i.write_axi;
            //
            or54 = or4[75:71];
            or55 = or38; or55[75:71] = or54;
            // d.write_controller.req_data = None();
            //
            or56 = or55; or56[144:76] = ol21;
            // d.read_controller.axi = i.read_axi;
            //
            or57 = or4[111:76];
            or58 = or56; or58[181:146] = or57;
            // d.read_controller.req_data = None();
            //
            or59 = or58; or59[214:182] = ol22;
            // d.inbuf.ready.raw = will_start;
            //
            or60 = or59; or60[70:70] = or53;
            // if will_start {
            //    if let Some(req, )#true = q.inbuf.data{
            //       match req {
            //          BlockRequest::Read(read, )#true => {
            //             d.state = State :: Reading;
            //             d.read_controller.req_data = Some(read);
            //          }
            //          ,
            //          BlockRequest::Write(write, )#false => {
            //             d.state = State :: Writing;
            //             d.write_controller.req_data = Some(write);
            //          }
            //          ,
            //       };
            //    }
            //
            // }
            //
            //
            // if let Some(req, )#true = q.inbuf.data{
            //    match req {
            //       BlockRequest::Read(read, )#true => {
            //          d.state = State :: Reading;
            //          d.read_controller.req_data = Some(read);
            //       }
            //       ,
            //       BlockRequest::Write(write, )#false => {
            //          d.state = State :: Writing;
            //          d.write_controller.req_data = Some(write);
            //       }
            //       ,
            //    };
            // }
            //
            //
            or61 = or1[70:0];
            or62 = or61[69:0];
            or63 = or62[69];
            or64 = or62[68:0];
            // match req {
            //    BlockRequest::Read(read, )#true => {
            //       d.state = State :: Reading;
            //       d.read_controller.req_data = Some(read);
            //    }
            //    ,
            //    BlockRequest::Write(write, )#false => {
            //       d.state = State :: Writing;
            //       d.write_controller.req_data = Some(write);
            //    }
            //    ,
            // };
            //
            or65 = or64[68];
            or66 = or64[31:0];
            // d.state = State :: Reading;
            //
            or67 = or60; or67[253:252] = ol23;
            // d.read_controller.req_data = Some(read);
            //
            or69 = or66[31:0];
            or68 = { ol24, or69 };
            or70 = or67; or70[214:182] = or68;
            or71 = or64[67:0];
            // d.state = State :: Writing;
            //
            or72 = or60; or72[253:252] = ol25;
            // d.write_controller.req_data = Some(write);
            //
            or74 = or71[67:0];
            or73 = { ol26, or74 };
            or75 = or72; or75[144:76] = or73;
            case (or65)
                1'b1: or76 = or70;
                1'b0: or76 = or75;
            endcase
            case (or63)
                1'b1: or77 = or76;
                default: or77 = or60;
            endcase
            or78 = (or53) ? (or77) : (or60);
            // d.outbuf.ready = i.resp_ready;
            //
            or79 = or4[70];
            or80 = or78; or80[251:251] = or79;
            // o.read_axi = q.read_controller.axi;
            //
            or81 = or1[214:146];
            or82 = or81[33:0];
            or83 = ol30; or83[140:107] = or82;
            // o.req_ready = q.inbuf.ready;
            //
            or84 = or1[70:0];
            or85 = or84[70];
            or86 = or83; or86[35:35] = or85;
            // o.response = q.outbuf.data;
            //
            or87 = or1[250:215];
            or88 = or87[34:0];
            or89 = or86; or89[34:0] = or88;
            // o.write_axi = q.write_controller.axi;
            //
            or90 = or1[145:71];
            or91 = or90[70:0];
            or92 = or89; or92[106:36] = or91;
            // (o, d, )
            //
            or93 = { or80, or92 };
            kernel_kernel = or93;
        end
    endfunction
endmodule
// synchronous circuit rhdl_fpga::stream::stream_buffer::StreamBuffer<rhdl_fpga::axi4lite::core::controller::blocking::BlockRequest>
module top_inbuf(input wire [1:0] clock_reset, input wire [70:0] i, output wire [70:0] o);
    wire [141:0] od;
    wire [70:0] d;
    wire [70:0] q;
    assign o = od[70:0];
    top_inbuf_inner c0 (.clock_reset(clock_reset),.i(d[70:0]),.o(q[70:0]));
    assign od = kernel_option_carloni_kernel(clock_reset, i, q);
    assign d = od[141:71];
    function [141:0] kernel_option_carloni_kernel(input reg [1:0] arg_0, input reg [70:0] arg_1, input reg [70:0] arg_2);
        reg [69:0] or0;
        reg [70:0] or1;
        reg [0:0] or2;
        reg [68:0] or3;
        reg [69:0] or4;
        reg [69:0] or5;
        reg [0:0] or6;
        reg [68:0] or7;
        reg [70:0] or8;  // d
        reg [0:0] or9;
        reg [70:0] or10;  // d
        reg [0:0] or11;
        reg [0:0] or12;
        reg [70:0] or13;  // d
        reg [70:0] or14;
        reg [0:0] or15;
        reg [0:0] or16;
        reg [0:0] or17;
        reg [0:0] or18;
        reg [70:0] or19;  // o
        reg [0:0] or20;
        reg [0:0] or21;
        reg [68:0] or22;
        reg [69:0] or23;
        reg [68:0] or24;
        reg [69:0] or25;
        reg [70:0] or26;  // o
        reg [141:0] or27;
        reg [1:0] or28;
        localparam ol0 = 1'b1;
        localparam ol1 = 1'b1;
        localparam ol2 = 1'b0;
        localparam ol3 = 70'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx0;
        localparam ol4 = 71'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol5 = 1'b0;
        localparam ol6 = 71'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol7 = 1'b1;
        localparam ol8 = 70'b0000000000000000000000000000000000000000000000000000000000000000000000;
        begin
            or28 = arg_0;
            or1 = arg_1;
            or14 = arg_2;
            // let d = D::<T>::dont_care();
            //
            // let (data_valid, data, ) = match i.data {
            //    Some(data, )#true => (true, data, ),
            //    _#false => (false, T::dont_care(), ),
            // };
            //
            or0 = or1[69:0];
            or2 = or0[69];
            or3 = or0[68:0];
            or4 = { or3, ol0 };
            case (or2)
                1'b1: or5 = or4;
                1'b0: or5 = ol3;
            endcase
            or6 = or5[0];
            or7 = or5[69:1];
            // d.inner.data_in = data;
            //
            or8 = ol4; or8[68:0] = or7;
            // d.inner.void_in = !data_valid;
            //
            or9 = ~(or6);
            or10 = or8; or10[69:69] = or9;
            // d.inner.stop_in = !i.ready.raw;
            //
            or11 = or1[70];
            or12 = ~(or11);
            or13 = or10; or13[70:70] = or12;
            // let o = Out::<T>::dont_care();
            //
            // o.ready = ready(!q.inner.stop_out);
            //
            or15 = or14[70];
            or16 = ~(or15);
            // Ready/* rhdl_fpga::stream::Ready<rhdl_fpga::axi4lite::core::controller::blocking::BlockRequest> */ {marker: PhantomData :: < T >, raw: raw,}
            //
            or17 = ol5;
            or18 = or17; or18[0:0] = or16;
            or19 = ol6; or19[70:70] = or18;
            // o.data = pack(!q.inner.void_out, q.inner.data_out);
            //
            or20 = or14[69];
            or21 = ~(or20);
            or22 = or14[68:0];
            // if valid {
            //    Some(data)
            // }
            //  else {
            //    None()
            // }
            //
            //
            // Some(data)
            //
            or24 = or22[68:0];
            or23 = { ol7, or24 };
            // None()
            //
            or25 = (or21) ? (or23) : (ol8);
            or26 = or19; or26[69:0] = or25;
            // (o, d, )
            //
            or27 = { or13, or26 };
            kernel_option_carloni_kernel = or27;
        end
    endfunction
endmodule
// synchronous circuit rhdl_fpga::lid::carloni::Carloni<rhdl_fpga::axi4lite::core::controller::blocking::BlockRequest>
module top_inbuf_inner(input wire [1:0] clock_reset, input wire [70:0] i, output wire [70:0] o);
    wire [210:0] od;
    wire [139:0] d;
    wire [139:0] q;
    assign o = od[70:0];
    top_inbuf_inner_aux_ff c0 (.clock_reset(clock_reset),.i(d[137:69]),.o(q[137:69]));
    top_inbuf_inner_main_ff c1 (.clock_reset(clock_reset),.i(d[68:0]),.o(q[68:0]));
    top_inbuf_inner_state_ff c2 (.clock_reset(clock_reset),.i(d[139]),.o(q[139]));
    top_inbuf_inner_void_ff c3 (.clock_reset(clock_reset),.i(d[138]),.o(q[138]));
    assign od = kernel_carloni_kernel(clock_reset, i, q);
    assign d = od[210:71];
    function [210:0] kernel_carloni_kernel(input reg [1:0] arg_0, input reg [70:0] arg_1, input reg [139:0] arg_2);
        reg [0:0] or0;
        reg [139:0] or1;
        reg [0:0] or2;
        reg [70:0] or3;
        reg [0:0] or4;
        reg [0:0] or5;
        reg [0:0] or6;
        reg [0:0] or7;
        reg [0:0] or8;
        reg [0:0] or9;
        reg [139:0] or10;  // d
        reg [0:0] or11;
        reg [0:0] or12;
        reg [0:0] or13;
        reg [0:0] or14;
        reg [0:0] or15;
        reg [0:0] or16;
        reg [0:0] or17;
        reg [139:0] or18;  // d
        reg [0:0] or19;  // aux_en
        reg [139:0] or20;  // d
        reg [0:0] or21;  // aux_en
        reg [139:0] or22;  // d
        reg [0:0] or23;  // main_en
        reg [0:0] or24;
        reg [139:0] or25;  // d
        reg [139:0] or26;  // d
        reg [0:0] or27;  // main_en
        reg [0:0] or28;  // sel
        reg [0:0] or29;  // stop_out
        reg [0:0] or30;  // aux_en
        reg [139:0] or31;  // d
        reg [0:0] or32;  // main_en
        reg [0:0] or33;  // sel
        reg [0:0] or34;  // stop_out
        reg [68:0] or35;
        reg [68:0] or36;
        reg [68:0] or37;
        reg [139:0] or38;  // d
        reg [68:0] or39;
        reg [68:0] or40;
        reg [68:0] or41;
        reg [68:0] or42;
        reg [68:0] or43;
        reg [139:0] or44;  // d
        reg [0:0] or45;
        reg [0:0] or46;
        reg [0:0] or47;
        reg [0:0] or48;
        reg [139:0] or49;  // d
        reg [68:0] or50;
        reg [70:0] or51;  // o
        reg [0:0] or52;
        reg [70:0] or53;  // o
        reg [70:0] or54;  // o
        reg [0:0] or55;
        reg [1:0] or56;
        reg [0:0] or57;
        reg [70:0] or58;  // o
        reg [70:0] or59;  // o
        reg [70:0] or60;  // o
        reg [210:0] or61;
        localparam ol0 = 140'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol1 = 1'b1;
        localparam ol2 = 1'b1;
        localparam ol3 = 1'b0;
        localparam ol4 = 1'b1;
        localparam ol5 = 1'b0;
        localparam ol6 = 1'b0;
        localparam ol7 = 1'b1;
        localparam ol8 = 1'b0;
        localparam ol9 = 1'b1;
        localparam ol10 = 1'b1;
        localparam ol11 = 1'b1;
        localparam ol12 = 1'b0;
        localparam ol13 = 1'b1;
        localparam ol14 = 1'b0;
        localparam ol15 = 1'b0;
        localparam ol16 = 71'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol17 = 1'b1;
        localparam ol18 = 1'b1;
        begin
            or56 = arg_0;
            or3 = arg_1;
            or1 = arg_2;
            // let d = D::<T>::dont_care();
            //
            // let o = Out::<T>::dont_care();
            //
            // let sel = false;
            //
            // let main_en = false;
            //
            // let aux_en = false;
            //
            // let stop_out = false;
            //
            // let void_out = q.void_ff;
            //
            or0 = or1[138];
            // let will_stall = i.stop_in & (!i.void_in & !void_out);
            //
            or2 = or3[70];
            or4 = or3[69];
            or5 = ~(or4);
            or6 = ~(or0);
            or7 = or5 & or6;
            or8 = or2 & or7;
            // d.state_ff = q.state_ff;
            //
            or9 = or1[139];
            or10 = ol0; or10[139:139] = or9;
            // match q.state_ff {
            //    const State::Run => {
            //       if !i.stop_in | (!i.void_in & void_out) {
            //          main_en = true;
            //       }
            //        else if will_stall {
            //          d.state_ff = State :: Stall;
            //          aux_en = true;
            //       }
            //
            //    }
            //    ,
            //    const State::Stall => {
            //       if i.stop_in {
            //          stop_out = true;
            //       }
            //        else {
            //          sel = true;
            //          main_en = true;
            //          stop_out = true;
            //          d.state_ff = State :: Run;
            //       }
            //
            //    }
            //    ,
            // }
            //
            or11 = or1[139];
            // if !i.stop_in | (!i.void_in & void_out) {
            //    main_en = true;
            // }
            //  else if will_stall {
            //    d.state_ff = State :: Stall;
            //    aux_en = true;
            // }
            //
            //
            or12 = or3[70];
            or13 = ~(or12);
            or14 = or3[69];
            or15 = ~(or14);
            or16 = or15 & or0;
            or17 = or13 | or16;
            // main_en = true;
            //
            // d.state_ff = State :: Stall;
            //
            or18 = or10; or18[139:139] = ol1;
            // aux_en = true;
            //
            or19 = (or8) ? (ol2) : (ol3);
            or20 = (or8) ? (or18) : (or10);
            or21 = (or17) ? (ol3) : (or19);
            or22 = (or17) ? (or10) : (or20);
            or23 = (or17) ? (ol4) : (ol5);
            // if i.stop_in {
            //    stop_out = true;
            // }
            //  else {
            //    sel = true;
            //    main_en = true;
            //    stop_out = true;
            //    d.state_ff = State :: Run;
            // }
            //
            //
            or24 = or3[70];
            // stop_out = true;
            //
            // sel = true;
            //
            // main_en = true;
            //
            // stop_out = true;
            //
            // d.state_ff = State :: Run;
            //
            or25 = or10; or25[139:139] = ol6;
            or26 = (or24) ? (or10) : (or25);
            or27 = (or24) ? (ol5) : (ol7);
            or28 = (or24) ? (ol8) : (ol9);
            or29 = (or24) ? (ol10) : (ol11);
            case (or11)
                1'b0: or30 = or21;
                1'b1: or30 = ol3;
            endcase
            case (or11)
                1'b0: or31 = or22;
                1'b1: or31 = or26;
            endcase
            case (or11)
                1'b0: or32 = or23;
                1'b1: or32 = or27;
            endcase
            case (or11)
                1'b0: or33 = ol8;
                1'b1: or33 = or28;
            endcase
            case (or11)
                1'b0: or34 = ol14;
                1'b1: or34 = or29;
            endcase
            // d.aux_ff = if aux_en {
            //    i.data_in
            // }
            //  else {
            //    q.aux_ff
            // }
            // ;
            //
            // i.data_in
            //
            or35 = or3[68:0];
            // q.aux_ff
            //
            or36 = or1[137:69];
            or37 = (or30) ? (or35) : (or36);
            or38 = or31; or38[137:69] = or37;
            // let d_mux = if sel {
            //    q.aux_ff
            // }
            //  else {
            //    i.data_in
            // }
            // ;
            //
            // q.aux_ff
            //
            or39 = or1[137:69];
            // i.data_in
            //
            or40 = or3[68:0];
            or41 = (or33) ? (or39) : (or40);
            // d.main_ff = if main_en {
            //    d_mux
            // }
            //  else {
            //    q.main_ff
            // }
            // ;
            //
            // d_mux
            //
            // q.main_ff
            //
            or42 = or1[68:0];
            or43 = (or32) ? (or41) : (or42);
            or44 = or38; or44[68:0] = or43;
            // let v_mux = if sel {
            //    false
            // }
            //  else {
            //    i.void_in
            // }
            // ;
            //
            // false
            //
            // i.void_in
            //
            or45 = or3[69];
            or46 = (or33) ? (ol15) : (or45);
            // d.void_ff = if main_en {
            //    v_mux
            // }
            //  else {
            //    q.void_ff
            // }
            // ;
            //
            // v_mux
            //
            // q.void_ff
            //
            or47 = or1[138];
            or48 = (or32) ? (or46) : (or47);
            or49 = or44; or49[138:138] = or48;
            // o.data_out = q.main_ff;
            //
            or50 = or1[68:0];
            or51 = ol16; or51[68:0] = or50;
            // o.void_out = q.void_ff;
            //
            or52 = or1[138];
            or53 = or51; or53[69:69] = or52;
            // o.stop_out = stop_out;
            //
            or54 = or53; or54[70:70] = or34;
            // if cr.reset.any() {
            //    o.void_out = true;
            //    o.stop_out = true;
            // }
            //
            //
            or55 = or56[1];
            or57 = |(or55);
            // o.void_out = true;
            //
            or58 = or54; or58[69:69] = ol17;
            // o.stop_out = true;
            //
            or59 = or58; or59[70:70] = ol18;
            or60 = (or57) ? (or59) : (or54);
            // (o, d, )
            //
            or61 = { or49, or60 };
            kernel_carloni_kernel = or61;
        end
    endfunction
endmodule
//
module top_inbuf_inner_aux_ff(input wire [1:0] clock_reset, input wire [68:0] i, output reg [68:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 69'b000000000000000000000000000000000000000000000000000000000000000000000;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 69'b000000000000000000000000000000000000000000000000000000000000000000000;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_inbuf_inner_main_ff(input wire [1:0] clock_reset, input wire [68:0] i, output reg [68:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 69'b000000000000000000000000000000000000000000000000000000000000000000000;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 69'b000000000000000000000000000000000000000000000000000000000000000000000;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_inbuf_inner_state_ff(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b0;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b0;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_inbuf_inner_void_ff(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b1;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b1;
        end else begin
            o <= i;
        end
    end
endmodule
// synchronous circuit rhdl_fpga::stream::stream_buffer::StreamBuffer<rhdl_fpga::axi4lite::core::controller::blocking::BlockResponse>
module top_outbuf(input wire [1:0] clock_reset, input wire [35:0] i, output wire [35:0] o);
    wire [71:0] od;
    wire [35:0] d;
    wire [35:0] q;
    assign o = od[35:0];
    top_outbuf_inner c0 (.clock_reset(clock_reset),.i(d[35:0]),.o(q[35:0]));
    assign od = kernel_option_carloni_kernel(clock_reset, i, q);
    assign d = od[71:36];
    function [71:0] kernel_option_carloni_kernel(input reg [1:0] arg_0, input reg [35:0] arg_1, input reg [35:0] arg_2);
        reg [34:0] or0;
        reg [35:0] or1;
        reg [0:0] or2;
        reg [33:0] or3;
        reg [34:0] or4;
        reg [34:0] or5;
        reg [0:0] or6;
        reg [33:0] or7;
        reg [35:0] or8;  // d
        reg [0:0] or9;
        reg [35:0] or10;  // d
        reg [0:0] or11;
        reg [0:0] or12;
        reg [35:0] or13;  // d
        reg [35:0] or14;
        reg [0:0] or15;
        reg [0:0] or16;
        reg [0:0] or17;
        reg [0:0] or18;
        reg [35:0] or19;  // o
        reg [0:0] or20;
        reg [0:0] or21;
        reg [33:0] or22;
        reg [34:0] or23;
        reg [33:0] or24;
        reg [34:0] or25;
        reg [35:0] or26;  // o
        reg [71:0] or27;
        reg [1:0] or28;
        localparam ol0 = 1'b1;
        localparam ol1 = 1'b1;
        localparam ol2 = 1'b0;
        localparam ol3 = 35'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx0;
        localparam ol4 = 36'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol5 = 1'b0;
        localparam ol6 = 36'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol7 = 1'b1;
        localparam ol8 = 35'b00000000000000000000000000000000000;
        begin
            or28 = arg_0;
            or1 = arg_1;
            or14 = arg_2;
            // let d = D::<T>::dont_care();
            //
            // let (data_valid, data, ) = match i.data {
            //    Some(data, )#true => (true, data, ),
            //    _#false => (false, T::dont_care(), ),
            // };
            //
            or0 = or1[34:0];
            or2 = or0[34];
            or3 = or0[33:0];
            or4 = { or3, ol0 };
            case (or2)
                1'b1: or5 = or4;
                1'b0: or5 = ol3;
            endcase
            or6 = or5[0];
            or7 = or5[34:1];
            // d.inner.data_in = data;
            //
            or8 = ol4; or8[33:0] = or7;
            // d.inner.void_in = !data_valid;
            //
            or9 = ~(or6);
            or10 = or8; or10[34:34] = or9;
            // d.inner.stop_in = !i.ready.raw;
            //
            or11 = or1[35];
            or12 = ~(or11);
            or13 = or10; or13[35:35] = or12;
            // let o = Out::<T>::dont_care();
            //
            // o.ready = ready(!q.inner.stop_out);
            //
            or15 = or14[35];
            or16 = ~(or15);
            // Ready/* rhdl_fpga::stream::Ready<rhdl_fpga::axi4lite::core::controller::blocking::BlockResponse> */ {marker: PhantomData :: < T >, raw: raw,}
            //
            or17 = ol5;
            or18 = or17; or18[0:0] = or16;
            or19 = ol6; or19[35:35] = or18;
            // o.data = pack(!q.inner.void_out, q.inner.data_out);
            //
            or20 = or14[34];
            or21 = ~(or20);
            or22 = or14[33:0];
            // if valid {
            //    Some(data)
            // }
            //  else {
            //    None()
            // }
            //
            //
            // Some(data)
            //
            or24 = or22[33:0];
            or23 = { ol7, or24 };
            // None()
            //
            or25 = (or21) ? (or23) : (ol8);
            or26 = or19; or26[34:0] = or25;
            // (o, d, )
            //
            or27 = { or13, or26 };
            kernel_option_carloni_kernel = or27;
        end
    endfunction
endmodule
// synchronous circuit rhdl_fpga::lid::carloni::Carloni<rhdl_fpga::axi4lite::core::controller::blocking::BlockResponse>
module top_outbuf_inner(input wire [1:0] clock_reset, input wire [35:0] i, output wire [35:0] o);
    wire [105:0] od;
    wire [69:0] d;
    wire [69:0] q;
    assign o = od[35:0];
    top_outbuf_inner_aux_ff c0 (.clock_reset(clock_reset),.i(d[67:34]),.o(q[67:34]));
    top_outbuf_inner_main_ff c1 (.clock_reset(clock_reset),.i(d[33:0]),.o(q[33:0]));
    top_outbuf_inner_state_ff c2 (.clock_reset(clock_reset),.i(d[69]),.o(q[69]));
    top_outbuf_inner_void_ff c3 (.clock_reset(clock_reset),.i(d[68]),.o(q[68]));
    assign od = kernel_carloni_kernel(clock_reset, i, q);
    assign d = od[105:36];
    function [105:0] kernel_carloni_kernel(input reg [1:0] arg_0, input reg [35:0] arg_1, input reg [69:0] arg_2);
        reg [0:0] or0;
        reg [69:0] or1;
        reg [0:0] or2;
        reg [35:0] or3;
        reg [0:0] or4;
        reg [0:0] or5;
        reg [0:0] or6;
        reg [0:0] or7;
        reg [0:0] or8;
        reg [0:0] or9;
        reg [69:0] or10;  // d
        reg [0:0] or11;
        reg [0:0] or12;
        reg [0:0] or13;
        reg [0:0] or14;
        reg [0:0] or15;
        reg [0:0] or16;
        reg [0:0] or17;
        reg [69:0] or18;  // d
        reg [0:0] or19;  // aux_en
        reg [69:0] or20;  // d
        reg [0:0] or21;  // aux_en
        reg [69:0] or22;  // d
        reg [0:0] or23;  // main_en
        reg [0:0] or24;
        reg [69:0] or25;  // d
        reg [69:0] or26;  // d
        reg [0:0] or27;  // main_en
        reg [0:0] or28;  // sel
        reg [0:0] or29;  // stop_out
        reg [0:0] or30;  // aux_en
        reg [69:0] or31;  // d
        reg [0:0] or32;  // main_en
        reg [0:0] or33;  // sel
        reg [0:0] or34;  // stop_out
        reg [33:0] or35;
        reg [33:0] or36;
        reg [33:0] or37;
        reg [69:0] or38;  // d
        reg [33:0] or39;
        reg [33:0] or40;
        reg [33:0] or41;
        reg [33:0] or42;
        reg [33:0] or43;
        reg [69:0] or44;  // d
        reg [0:0] or45;
        reg [0:0] or46;
        reg [0:0] or47;
        reg [0:0] or48;
        reg [69:0] or49;  // d
        reg [33:0] or50;
        reg [35:0] or51;  // o
        reg [0:0] or52;
        reg [35:0] or53;  // o
        reg [35:0] or54;  // o
        reg [0:0] or55;
        reg [1:0] or56;
        reg [0:0] or57;
        reg [35:0] or58;  // o
        reg [35:0] or59;  // o
        reg [35:0] or60;  // o
        reg [105:0] or61;
        localparam ol0 = 70'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol1 = 1'b1;
        localparam ol2 = 1'b1;
        localparam ol3 = 1'b0;
        localparam ol4 = 1'b1;
        localparam ol5 = 1'b0;
        localparam ol6 = 1'b0;
        localparam ol7 = 1'b1;
        localparam ol8 = 1'b0;
        localparam ol9 = 1'b1;
        localparam ol10 = 1'b1;
        localparam ol11 = 1'b1;
        localparam ol12 = 1'b0;
        localparam ol13 = 1'b1;
        localparam ol14 = 1'b0;
        localparam ol15 = 1'b0;
        localparam ol16 = 36'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol17 = 1'b1;
        localparam ol18 = 1'b1;
        begin
            or56 = arg_0;
            or3 = arg_1;
            or1 = arg_2;
            // let d = D::<T>::dont_care();
            //
            // let o = Out::<T>::dont_care();
            //
            // let sel = false;
            //
            // let main_en = false;
            //
            // let aux_en = false;
            //
            // let stop_out = false;
            //
            // let void_out = q.void_ff;
            //
            or0 = or1[68];
            // let will_stall = i.stop_in & (!i.void_in & !void_out);
            //
            or2 = or3[35];
            or4 = or3[34];
            or5 = ~(or4);
            or6 = ~(or0);
            or7 = or5 & or6;
            or8 = or2 & or7;
            // d.state_ff = q.state_ff;
            //
            or9 = or1[69];
            or10 = ol0; or10[69:69] = or9;
            // match q.state_ff {
            //    const State::Run => {
            //       if !i.stop_in | (!i.void_in & void_out) {
            //          main_en = true;
            //       }
            //        else if will_stall {
            //          d.state_ff = State :: Stall;
            //          aux_en = true;
            //       }
            //
            //    }
            //    ,
            //    const State::Stall => {
            //       if i.stop_in {
            //          stop_out = true;
            //       }
            //        else {
            //          sel = true;
            //          main_en = true;
            //          stop_out = true;
            //          d.state_ff = State :: Run;
            //       }
            //
            //    }
            //    ,
            // }
            //
            or11 = or1[69];
            // if !i.stop_in | (!i.void_in & void_out) {
            //    main_en = true;
            // }
            //  else if will_stall {
            //    d.state_ff = State :: Stall;
            //    aux_en = true;
            // }
            //
            //
            or12 = or3[35];
            or13 = ~(or12);
            or14 = or3[34];
            or15 = ~(or14);
            or16 = or15 & or0;
            or17 = or13 | or16;
            // main_en = true;
            //
            // d.state_ff = State :: Stall;
            //
            or18 = or10; or18[69:69] = ol1;
            // aux_en = true;
            //
            or19 = (or8) ? (ol2) : (ol3);
            or20 = (or8) ? (or18) : (or10);
            or21 = (or17) ? (ol3) : (or19);
            or22 = (or17) ? (or10) : (or20);
            or23 = (or17) ? (ol4) : (ol5);
            // if i.stop_in {
            //    stop_out = true;
            // }
            //  else {
            //    sel = true;
            //    main_en = true;
            //    stop_out = true;
            //    d.state_ff = State :: Run;
            // }
            //
            //
            or24 = or3[35];
            // stop_out = true;
            //
            // sel = true;
            //
            // main_en = true;
            //
            // stop_out = true;
            //
            // d.state_ff = State :: Run;
            //
            or25 = or10; or25[69:69] = ol6;
            or26 = (or24) ? (or10) : (or25);
            or27 = (or24) ? (ol5) : (ol7);
            or28 = (or24) ? (ol8) : (ol9);
            or29 = (or24) ? (ol10) : (ol11);
            case (or11)
                1'b0: or30 = or21;
                1'b1: or30 = ol3;
            endcase
            case (or11)
                1'b0: or31 = or22;
                1'b1: or31 = or26;
            endcase
            case (or11)
                1'b0: or32 = or23;
                1'b1: or32 = or27;
            endcase
            case (or11)
                1'b0: or33 = ol8;
                1'b1: or33 = or28;
            endcase
            case (or11)
                1'b0: or34 = ol14;
                1'b1: or34 = or29;
            endcase
            // d.aux_ff = if aux_en {
            //    i.data_in
            // }
            //  else {
            //    q.aux_ff
            // }
            // ;
            //
            // i.data_in
            //
            or35 = or3[33:0];
            // q.aux_ff
            //
            or36 = or1[67:34];
            or37 = (or30) ? (or35) : (or36);
            or38 = or31; or38[67:34] = or37;
            // let d_mux = if sel {
            //    q.aux_ff
            // }
            //  else {
            //    i.data_in
            // }
            // ;
            //
            // q.aux_ff
            //
            or39 = or1[67:34];
            // i.data_in
            //
            or40 = or3[33:0];
            or41 = (or33) ? (or39) : (or40);
            // d.main_ff = if main_en {
            //    d_mux
            // }
            //  else {
            //    q.main_ff
            // }
            // ;
            //
            // d_mux
            //
            // q.main_ff
            //
            or42 = or1[33:0];
            or43 = (or32) ? (or41) : (or42);
            or44 = or38; or44[33:0] = or43;
            // let v_mux = if sel {
            //    false
            // }
            //  else {
            //    i.void_in
            // }
            // ;
            //
            // false
            //
            // i.void_in
            //
            or45 = or3[34];
            or46 = (or33) ? (ol15) : (or45);
            // d.void_ff = if main_en {
            //    v_mux
            // }
            //  else {
            //    q.void_ff
            // }
            // ;
            //
            // v_mux
            //
            // q.void_ff
            //
            or47 = or1[68];
            or48 = (or32) ? (or46) : (or47);
            or49 = or44; or49[68:68] = or48;
            // o.data_out = q.main_ff;
            //
            or50 = or1[33:0];
            or51 = ol16; or51[33:0] = or50;
            // o.void_out = q.void_ff;
            //
            or52 = or1[68];
            or53 = or51; or53[34:34] = or52;
            // o.stop_out = stop_out;
            //
            or54 = or53; or54[35:35] = or34;
            // if cr.reset.any() {
            //    o.void_out = true;
            //    o.stop_out = true;
            // }
            //
            //
            or55 = or56[1];
            or57 = |(or55);
            // o.void_out = true;
            //
            or58 = or54; or58[34:34] = ol17;
            // o.stop_out = true;
            //
            or59 = or58; or59[35:35] = ol18;
            or60 = (or57) ? (or59) : (or54);
            // (o, d, )
            //
            or61 = { or49, or60 };
            kernel_carloni_kernel = or61;
        end
    endfunction
endmodule
//
module top_outbuf_inner_aux_ff(input wire [1:0] clock_reset, input wire [33:0] i, output reg [33:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 34'b0000000000000000000000000000000001;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 34'b0000000000000000000000000000000001;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_outbuf_inner_main_ff(input wire [1:0] clock_reset, input wire [33:0] i, output reg [33:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 34'b0000000000000000000000000000000001;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 34'b0000000000000000000000000000000001;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_outbuf_inner_state_ff(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b0;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b0;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_outbuf_inner_void_ff(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b1;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b1;
        end else begin
            o <= i;
        end
    end
endmodule
// synchronous circuit rhdl_fpga::axi4lite::core::controller::read::ReadController
module top_read_controller(input wire [1:0] clock_reset, input wire [69:0] i, output wire [68:0] o);
    wire [174:0] od;
    wire [105:0] d;
    wire [104:0] q;
    assign o = od[68:0];
    top_read_controller_inbuf c0 (.clock_reset(clock_reset),.i(d[33:0]),.o(q[33:0]));
    top_read_controller_map c1 (.clock_reset(clock_reset),.i(d[69:34]),.o(q[68:34]));
    top_read_controller_outbuf c2 (.clock_reset(clock_reset),.i(d[105:70]),.o(q[104:69]));
    assign od = kernel_kernel(clock_reset, i, q);
    assign d = od[174:69];
    function [174:0] kernel_kernel(input reg [1:0] arg_0, input reg [69:0] arg_1, input reg [104:0] arg_2);
        reg [32:0] or0;
        reg [69:0] or1;
        reg [105:0] or2;  // d
        reg [35:0] or3;
        reg [0:0] or4;
        reg [105:0] or5;  // d
        reg [35:0] or6;
        reg [1:0] or7;
        reg [35:0] or8;
        reg [31:0] or9;
        reg [33:0] or10;
        reg [33:0] or11;
        reg [105:0] or12;  // d
        reg [35:0] or13;
        reg [0:0] or14;
        reg [105:0] or15;  // d
        reg [34:0] or16;
        reg [104:0] or17;
        reg [0:0] or18;
        reg [105:0] or19;  // d
        reg [35:0] or20;
        reg [34:0] or21;
        reg [105:0] or22;  // d
        reg [0:0] or23;
        reg [105:0] or24;  // d
        reg [33:0] or25;
        reg [0:0] or26;
        reg [68:0] or27;  // o
        reg [34:0] or28;
        reg [33:0] or29;
        reg [68:0] or30;  // o
        reg [33:0] or31;
        reg [31:0] or32;
        reg [68:0] or33;  // o
        reg [33:0] or34;
        reg [0:0] or35;
        reg [68:0] or36;  // o
        reg [35:0] or37;
        reg [0:0] or38;
        reg [68:0] or39;  // o
        reg [174:0] or40;
        reg [1:0] or41;
        localparam ol0 = 106'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol1 = 34'b0000000000000000000000000000000000;
        localparam ol2 = 69'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        begin
            or41 = arg_0;
            or1 = arg_1;
            or17 = arg_2;
            // let d = D::dont_care();
            //
            // d.inbuf.data = i.req_data;
            //
            or0 = or1[68:36];
            or2 = ol0; or2[32:0] = or0;
            // d.inbuf.tready = i.axi.arready;
            //
            or3 = or1[35:0];
            or4 = or3[0];
            or5 = or2; or5[33:33] = or4;
            // d.outbuf.tdata = ReadResponse/* rhdl_fpga::axi4lite::types::ReadResponse */ {resp: i.axi.rresp, data: i.axi.rdata,};
            //
            or6 = or1[35:0];
            or7 = or6[34:33];
            or8 = or1[35:0];
            or9 = or8[32:1];
            or10 = ol1; or10[1:0] = or7;
            or11 = or10; or11[33:2] = or9;
            or12 = or5; or12[103:70] = or11;
            // d.outbuf.tvalid = i.axi.rvalid;
            //
            or13 = or1[35:0];
            or14 = or13[35];
            or15 = or12; or15[104:104] = or14;
            // d.outbuf.ready = q.map.ready;
            //
            or16 = or17[68:34];
            or18 = or16[34];
            or19 = or15; or19[105:105] = or18;
            // d.map.data = q.outbuf.data;
            //
            or20 = or17[104:69];
            or21 = or20[34:0];
            or22 = or19; or22[68:34] = or21;
            // d.map.ready = i.resp_ready;
            //
            or23 = or1[69];
            or24 = or22; or24[69:69] = or23;
            // let o = Out::dont_care();
            //
            // o.req_ready = q.inbuf.ready;
            //
            or25 = or17[33:0];
            or26 = or25[33];
            or27 = ol2; or27[34:34] = or26;
            // o.resp_data = q.map.data;
            //
            or28 = or17[68:34];
            or29 = or28[33:0];
            or30 = or27; or30[68:35] = or29;
            // o.axi.araddr = q.inbuf.tdata;
            //
            or31 = or17[33:0];
            or32 = or31[31:0];
            or33 = or30; or33[31:0] = or32;
            // o.axi.arvalid = q.inbuf.tvalid;
            //
            or34 = or17[33:0];
            or35 = or34[32];
            or36 = or33; or36[32:32] = or35;
            // o.axi.rready = q.outbuf.tready;
            //
            or37 = or17[104:69];
            or38 = or37[35];
            or39 = or36; or39[33:33] = or38;
            // (o, d, )
            //
            or40 = { or24, or39 };
            kernel_kernel = or40;
        end
    endfunction
endmodule
// synchronous circuit rhdl_fpga::axi4lite::stream::rhdl_to_axi::Rhdl2Axi<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U32>>
module top_read_controller_inbuf(input wire [1:0] clock_reset, input wire [33:0] i, output wire [33:0] o);
    wire [67:0] od;
    wire [33:0] d;
    wire [33:0] q;
    assign o = od[33:0];
    top_read_controller_inbuf_outbuf c0 (.clock_reset(clock_reset),.i(d[33:0]),.o(q[33:0]));
    assign od = kernel_kernel(clock_reset, i, q);
    assign d = od[67:34];
    function [67:0] kernel_kernel(input reg [1:0] arg_0, input reg [33:0] arg_1, input reg [33:0] arg_2);
        reg [32:0] or0;
        reg [33:0] or1;
        reg [0:0] or2;
        reg [31:0] or3;
        reg [31:0] or4;  // tdata
        reg [0:0] or5;  // tvalid
        reg [33:0] or6;  // d
        reg [0:0] or7;
        reg [33:0] or8;  // d
        reg [0:0] or9;
        reg [0:0] or10;
        reg [33:0] or11;  // d
        reg [33:0] or12;
        reg [31:0] or13;
        reg [33:0] or14;  // o
        reg [0:0] or15;
        reg [0:0] or16;
        reg [33:0] or17;  // o
        reg [0:0] or18;
        reg [0:0] or19;
        reg [0:0] or20;
        reg [0:0] or21;
        reg [33:0] or22;  // o
        reg [67:0] or23;
        reg [1:0] or24;
        localparam ol0 = 1'b1;
        localparam ol1 = 32'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol2 = 1'b1;
        localparam ol3 = 1'b0;
        localparam ol4 = 34'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol5 = 34'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol6 = 1'b0;
        begin
            or24 = arg_0;
            or1 = arg_1;
            or12 = arg_2;
            // let tdata = T::dont_care();
            //
            // let tvalid = false;
            //
            // if let Some(data, )#true = i.data{
            //    tdata = data;
            //    tvalid = true;
            // }
            //
            //
            or0 = or1[32:0];
            or2 = or0[32];
            or3 = or0[31:0];
            // tdata = data;
            //
            // tvalid = true;
            //
            case (or2)
                1'b1: or4 = or3;
                default: or4 = ol1;
            endcase
            case (or2)
                1'b1: or5 = ol2;
                default: or5 = ol3;
            endcase
            // let d = D::<T>::dont_care();
            //
            // let o = Out::<T>::dont_care();
            //
            // d.outbuf.data_in = tdata;
            //
            or6 = ol4; or6[31:0] = or4;
            // d.outbuf.void_in = !tvalid;
            //
            or7 = ~(or5);
            or8 = or6; or8[32:32] = or7;
            // d.outbuf.stop_in = !i.tready;
            //
            or9 = or1[33];
            or10 = ~(or9);
            or11 = or8; or11[33:33] = or10;
            // o.tdata = q.outbuf.data_out;
            //
            or13 = or12[31:0];
            or14 = ol5; or14[31:0] = or13;
            // o.tvalid = !q.outbuf.void_out;
            //
            or15 = or12[32];
            or16 = ~(or15);
            or17 = or14; or17[32:32] = or16;
            // o.ready = ready(!q.outbuf.stop_out);
            //
            or18 = or12[33];
            or19 = ~(or18);
            // Ready/* rhdl_fpga::stream::Ready<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U32>> */ {marker: PhantomData :: < T >, raw: raw,}
            //
            or20 = ol6;
            or21 = or20; or21[0:0] = or19;
            or22 = or17; or22[33:33] = or21;
            // (o, d, )
            //
            or23 = { or11, or22 };
            kernel_kernel = or23;
        end
    endfunction
endmodule
// synchronous circuit rhdl_fpga::lid::carloni::Carloni<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U32>>
module top_read_controller_inbuf_outbuf(input wire [1:0] clock_reset, input wire [33:0] i, output wire [33:0] o);
    wire [99:0] od;
    wire [65:0] d;
    wire [65:0] q;
    assign o = od[33:0];
    top_read_controller_inbuf_outbuf_aux_ff c0 (.clock_reset(clock_reset),.i(d[63:32]),.o(q[63:32]));
    top_read_controller_inbuf_outbuf_main_ff c1 (.clock_reset(clock_reset),.i(d[31:0]),.o(q[31:0]));
    top_read_controller_inbuf_outbuf_state_ff c2 (.clock_reset(clock_reset),.i(d[65]),.o(q[65]));
    top_read_controller_inbuf_outbuf_void_ff c3 (.clock_reset(clock_reset),.i(d[64]),.o(q[64]));
    assign od = kernel_carloni_kernel(clock_reset, i, q);
    assign d = od[99:34];
    function [99:0] kernel_carloni_kernel(input reg [1:0] arg_0, input reg [33:0] arg_1, input reg [65:0] arg_2);
        reg [0:0] or0;
        reg [65:0] or1;
        reg [0:0] or2;
        reg [33:0] or3;
        reg [0:0] or4;
        reg [0:0] or5;
        reg [0:0] or6;
        reg [0:0] or7;
        reg [0:0] or8;
        reg [0:0] or9;
        reg [65:0] or10;  // d
        reg [0:0] or11;
        reg [0:0] or12;
        reg [0:0] or13;
        reg [0:0] or14;
        reg [0:0] or15;
        reg [0:0] or16;
        reg [0:0] or17;
        reg [65:0] or18;  // d
        reg [0:0] or19;  // aux_en
        reg [65:0] or20;  // d
        reg [0:0] or21;  // aux_en
        reg [65:0] or22;  // d
        reg [0:0] or23;  // main_en
        reg [0:0] or24;
        reg [65:0] or25;  // d
        reg [65:0] or26;  // d
        reg [0:0] or27;  // main_en
        reg [0:0] or28;  // sel
        reg [0:0] or29;  // stop_out
        reg [0:0] or30;  // aux_en
        reg [65:0] or31;  // d
        reg [0:0] or32;  // main_en
        reg [0:0] or33;  // sel
        reg [0:0] or34;  // stop_out
        reg [31:0] or35;
        reg [31:0] or36;
        reg [31:0] or37;
        reg [65:0] or38;  // d
        reg [31:0] or39;
        reg [31:0] or40;
        reg [31:0] or41;
        reg [31:0] or42;
        reg [31:0] or43;
        reg [65:0] or44;  // d
        reg [0:0] or45;
        reg [0:0] or46;
        reg [0:0] or47;
        reg [0:0] or48;
        reg [65:0] or49;  // d
        reg [31:0] or50;
        reg [33:0] or51;  // o
        reg [0:0] or52;
        reg [33:0] or53;  // o
        reg [33:0] or54;  // o
        reg [0:0] or55;
        reg [1:0] or56;
        reg [0:0] or57;
        reg [33:0] or58;  // o
        reg [33:0] or59;  // o
        reg [33:0] or60;  // o
        reg [99:0] or61;
        localparam ol0 = 66'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol1 = 1'b1;
        localparam ol2 = 1'b1;
        localparam ol3 = 1'b0;
        localparam ol4 = 1'b1;
        localparam ol5 = 1'b0;
        localparam ol6 = 1'b0;
        localparam ol7 = 1'b1;
        localparam ol8 = 1'b0;
        localparam ol9 = 1'b1;
        localparam ol10 = 1'b1;
        localparam ol11 = 1'b1;
        localparam ol12 = 1'b0;
        localparam ol13 = 1'b1;
        localparam ol14 = 1'b0;
        localparam ol15 = 1'b0;
        localparam ol16 = 34'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol17 = 1'b1;
        localparam ol18 = 1'b1;
        begin
            or56 = arg_0;
            or3 = arg_1;
            or1 = arg_2;
            // let d = D::<T>::dont_care();
            //
            // let o = Out::<T>::dont_care();
            //
            // let sel = false;
            //
            // let main_en = false;
            //
            // let aux_en = false;
            //
            // let stop_out = false;
            //
            // let void_out = q.void_ff;
            //
            or0 = or1[64];
            // let will_stall = i.stop_in & (!i.void_in & !void_out);
            //
            or2 = or3[33];
            or4 = or3[32];
            or5 = ~(or4);
            or6 = ~(or0);
            or7 = or5 & or6;
            or8 = or2 & or7;
            // d.state_ff = q.state_ff;
            //
            or9 = or1[65];
            or10 = ol0; or10[65:65] = or9;
            // match q.state_ff {
            //    const State::Run => {
            //       if !i.stop_in | (!i.void_in & void_out) {
            //          main_en = true;
            //       }
            //        else if will_stall {
            //          d.state_ff = State :: Stall;
            //          aux_en = true;
            //       }
            //
            //    }
            //    ,
            //    const State::Stall => {
            //       if i.stop_in {
            //          stop_out = true;
            //       }
            //        else {
            //          sel = true;
            //          main_en = true;
            //          stop_out = true;
            //          d.state_ff = State :: Run;
            //       }
            //
            //    }
            //    ,
            // }
            //
            or11 = or1[65];
            // if !i.stop_in | (!i.void_in & void_out) {
            //    main_en = true;
            // }
            //  else if will_stall {
            //    d.state_ff = State :: Stall;
            //    aux_en = true;
            // }
            //
            //
            or12 = or3[33];
            or13 = ~(or12);
            or14 = or3[32];
            or15 = ~(or14);
            or16 = or15 & or0;
            or17 = or13 | or16;
            // main_en = true;
            //
            // d.state_ff = State :: Stall;
            //
            or18 = or10; or18[65:65] = ol1;
            // aux_en = true;
            //
            or19 = (or8) ? (ol2) : (ol3);
            or20 = (or8) ? (or18) : (or10);
            or21 = (or17) ? (ol3) : (or19);
            or22 = (or17) ? (or10) : (or20);
            or23 = (or17) ? (ol4) : (ol5);
            // if i.stop_in {
            //    stop_out = true;
            // }
            //  else {
            //    sel = true;
            //    main_en = true;
            //    stop_out = true;
            //    d.state_ff = State :: Run;
            // }
            //
            //
            or24 = or3[33];
            // stop_out = true;
            //
            // sel = true;
            //
            // main_en = true;
            //
            // stop_out = true;
            //
            // d.state_ff = State :: Run;
            //
            or25 = or10; or25[65:65] = ol6;
            or26 = (or24) ? (or10) : (or25);
            or27 = (or24) ? (ol5) : (ol7);
            or28 = (or24) ? (ol8) : (ol9);
            or29 = (or24) ? (ol10) : (ol11);
            case (or11)
                1'b0: or30 = or21;
                1'b1: or30 = ol3;
            endcase
            case (or11)
                1'b0: or31 = or22;
                1'b1: or31 = or26;
            endcase
            case (or11)
                1'b0: or32 = or23;
                1'b1: or32 = or27;
            endcase
            case (or11)
                1'b0: or33 = ol8;
                1'b1: or33 = or28;
            endcase
            case (or11)
                1'b0: or34 = ol14;
                1'b1: or34 = or29;
            endcase
            // d.aux_ff = if aux_en {
            //    i.data_in
            // }
            //  else {
            //    q.aux_ff
            // }
            // ;
            //
            // i.data_in
            //
            or35 = or3[31:0];
            // q.aux_ff
            //
            or36 = or1[63:32];
            or37 = (or30) ? (or35) : (or36);
            or38 = or31; or38[63:32] = or37;
            // let d_mux = if sel {
            //    q.aux_ff
            // }
            //  else {
            //    i.data_in
            // }
            // ;
            //
            // q.aux_ff
            //
            or39 = or1[63:32];
            // i.data_in
            //
            or40 = or3[31:0];
            or41 = (or33) ? (or39) : (or40);
            // d.main_ff = if main_en {
            //    d_mux
            // }
            //  else {
            //    q.main_ff
            // }
            // ;
            //
            // d_mux
            //
            // q.main_ff
            //
            or42 = or1[31:0];
            or43 = (or32) ? (or41) : (or42);
            or44 = or38; or44[31:0] = or43;
            // let v_mux = if sel {
            //    false
            // }
            //  else {
            //    i.void_in
            // }
            // ;
            //
            // false
            //
            // i.void_in
            //
            or45 = or3[32];
            or46 = (or33) ? (ol15) : (or45);
            // d.void_ff = if main_en {
            //    v_mux
            // }
            //  else {
            //    q.void_ff
            // }
            // ;
            //
            // v_mux
            //
            // q.void_ff
            //
            or47 = or1[64];
            or48 = (or32) ? (or46) : (or47);
            or49 = or44; or49[64:64] = or48;
            // o.data_out = q.main_ff;
            //
            or50 = or1[31:0];
            or51 = ol16; or51[31:0] = or50;
            // o.void_out = q.void_ff;
            //
            or52 = or1[64];
            or53 = or51; or53[32:32] = or52;
            // o.stop_out = stop_out;
            //
            or54 = or53; or54[33:33] = or34;
            // if cr.reset.any() {
            //    o.void_out = true;
            //    o.stop_out = true;
            // }
            //
            //
            or55 = or56[1];
            or57 = |(or55);
            // o.void_out = true;
            //
            or58 = or54; or58[32:32] = ol17;
            // o.stop_out = true;
            //
            or59 = or58; or59[33:33] = ol18;
            or60 = (or57) ? (or59) : (or54);
            // (o, d, )
            //
            or61 = { or49, or60 };
            kernel_carloni_kernel = or61;
        end
    endfunction
endmodule
//
module top_read_controller_inbuf_outbuf_aux_ff(input wire [1:0] clock_reset, input wire [31:0] i, output reg [31:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 32'b00000000000000000000000000000000;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 32'b00000000000000000000000000000000;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_read_controller_inbuf_outbuf_main_ff(input wire [1:0] clock_reset, input wire [31:0] i, output reg [31:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 32'b00000000000000000000000000000000;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 32'b00000000000000000000000000000000;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_read_controller_inbuf_outbuf_state_ff(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b0;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b0;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_read_controller_inbuf_outbuf_void_ff(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b1;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b1;
        end else begin
            o <= i;
        end
    end
endmodule
// synchronous circuit rhdl_fpga::stream::map::Map<rhdl_fpga::axi4lite::types::ReadResponse, core::result::Result<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U32>, rhdl_fpga::axi4lite::types::AXI4Error>>
module top_read_controller_map(input wire [1:0] clock_reset, input wire [35:0] i, output wire [34:0] o);
    wire [104:0] od;
    wire [69:0] d;
    wire [68:0] q;
    assign o = od[34:0];
    top_read_controller_map_func c0 (.clock_reset(clock_reset),.i(d[69:36]),.o(q[68:36]));
    top_read_controller_map_input_buffer c1 (.clock_reset(clock_reset),.i(d[35:0]),.o(q[35:0]));
    assign od = kernel_kernel(clock_reset, i, q);
    assign d = od[104:35];
    function [104:0] kernel_kernel(input reg [1:0] arg_0, input reg [35:0] arg_1, input reg [68:0] arg_2);
        reg [34:0] or0;
        reg [35:0] or1;
        reg [69:0] or2;  // d
        reg [0:0] or3;
        reg [0:0] or4;
        reg [0:0] or5;
        reg [69:0] or6;  // d
        reg [35:0] or7;
        reg [68:0] or8;
        reg [34:0] or9;
        reg [0:0] or10;
        reg [33:0] or11;
        reg [69:0] or12;  // d
        reg [32:0] or13;
        reg [33:0] or14;
        reg [32:0] or15;
        reg [69:0] or16;  // d
        reg [69:0] or17;  // d
        reg [33:0] or18;
        reg [35:0] or19;
        reg [0:0] or20;
        reg [34:0] or21;
        reg [34:0] or22;
        reg [104:0] or23;
        reg [1:0] or24;
        localparam ol0 = 70'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol1 = 1'b0;
        localparam ol2 = 1'b1;
        localparam ol3 = 34'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol4 = 1'b1;
        localparam ol5 = 34'b0000000000000000000000000000000000;
        localparam ol6 = 35'b00000000000000000000000000000000000;
        begin
            or24 = arg_0;
            or1 = arg_1;
            or8 = arg_2;
            // let d = D::<T,S>::dont_care();
            //
            // d.input_buffer.data = i.data;
            //
            or0 = or1[34:0];
            or2 = ol0; or2[34:0] = or0;
            // d.input_buffer.ready = ready_cast(i.ready);
            //
            or3 = or1[35];
            // Ready/* rhdl_fpga::stream::Ready<rhdl_fpga::axi4lite::types::ReadResponse> */ {marker: PhantomData :: < T >, raw: input.raw,}
            //
            or4 = ol1;
            or5 = or4; or5[0:0] = or3;
            or6 = or2; or6[35:35] = or5;
            // let o_data = if let Some(data, )#true = q.input_buffer.data{
            //    d.func = data;
            //    Some(q.func)
            // }
            //  else {
            //    d.func = T::dont_care();
            //    None()
            // }
            // ;
            //
            or7 = or8[35:0];
            or9 = or7[34:0];
            or10 = or9[34];
            or11 = or9[33:0];
            // d.func = data;
            //
            or12 = or6; or12[69:36] = or11;
            // Some(q.func)
            //
            or13 = or8[68:36];
            or15 = or13[32:0];
            or14 = { ol2, or15 };
            // d.func = T::dont_care();
            //
            or16 = or6; or16[69:36] = ol3;
            // None()
            //
            case (or10)
                1'b1: or17 = or12;
                default: or17 = or16;
            endcase
            case (or10)
                1'b1: or18 = or14;
                default: or18 = ol5;
            endcase
            // let o = Out/* rhdl_fpga::stream::StreamIO<core::result::Result<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U32>, rhdl_fpga::axi4lite::types::AXI4Error>rhdl_fpga::axi4lite::types::ReadResponse> */ {data: o_data, ready: q.input_buffer.ready,};
            //
            or19 = or8[35:0];
            or20 = or19[35];
            or21 = ol6; or21[33:0] = or18;
            or22 = or21; or22[34:34] = or20;
            // (o, d, )
            //
            or23 = { or17, or22 };
            kernel_kernel = or23;
        end
    endfunction
endmodule
// synchronous circuit rhdl::rhdl_core::circuit::func::Func<rhdl_fpga::axi4lite::types::ReadResponse, core::result::Result<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U32>, rhdl_fpga::axi4lite::types::AXI4Error>>
module top_read_controller_map_func(input wire [1:0] clock_reset, input wire [33:0] i, output wire [32:0] o);
    assign o = kernel_map_result(clock_reset, i);
    function [32:0] kernel_map_result(input reg [1:0] arg_0, input reg [33:0] arg_1);
        reg [1:0] or0;
        reg [33:0] or1;
        reg [31:0] or2;
        reg [32:0] or3;
        reg [31:0] or4;
        reg [32:0] or5;
        reg [32:0] or6;
        reg [1:0] or7;
        localparam ol0 = 33'b100000000000000000000000000000000;
        localparam ol1 = 33'b100000000000000000000000000000000;
        localparam ol2 = 2'b00;
        localparam ol3 = 2'b01;
        localparam ol4 = 2'b11;
        localparam ol5 = 33'b000000000000000000000000000000001;
        localparam ol6 = 2'b10;
        localparam ol7 = 33'b000000000000000000000000000000000;
        localparam ol8 = 33'b000000000000000000000000000000001;
        begin
            or7 = arg_0;
            or1 = arg_1;
            // match resp.resp {
            //    const response_codes::OKAY => ReadResult::Ok(resp.data),
            //    const response_codes::EXOKAY => ReadResult::Ok(resp.data),
            //    const response_codes::DECERR => ReadResult::Err(AXI4Error :: DECERR),
            //    const response_codes::SLVERR => ReadResult::Err(AXI4Error :: SLVERR),
            //    _ => ReadResult::Err(AXI4Error :: DECERR),
            // }
            //
            or0 = or1[1:0];
            or2 = or1[33:2];
            or3 = ol0; or3[31:0] = or2;
            or4 = or1[33:2];
            or5 = ol1; or5[31:0] = or4;
            case (or0)
                2'b00: or6 = or3;
                2'b01: or6 = or5;
                2'b11: or6 = ol5;
                2'b10: or6 = ol7;
                default: or6 = ol8;
            endcase
            kernel_map_result = or6;
        end
    endfunction
endmodule
// synchronous circuit rhdl_fpga::stream::stream_buffer::StreamBuffer<rhdl_fpga::axi4lite::types::ReadResponse>
module top_read_controller_map_input_buffer(input wire [1:0] clock_reset, input wire [35:0] i, output wire [35:0] o);
    wire [71:0] od;
    wire [35:0] d;
    wire [35:0] q;
    assign o = od[35:0];
    top_read_controller_map_input_buffer_inner c0 (.clock_reset(clock_reset),.i(d[35:0]),.o(q[35:0]));
    assign od = kernel_option_carloni_kernel(clock_reset, i, q);
    assign d = od[71:36];
    function [71:0] kernel_option_carloni_kernel(input reg [1:0] arg_0, input reg [35:0] arg_1, input reg [35:0] arg_2);
        reg [34:0] or0;
        reg [35:0] or1;
        reg [0:0] or2;
        reg [33:0] or3;
        reg [34:0] or4;
        reg [34:0] or5;
        reg [0:0] or6;
        reg [33:0] or7;
        reg [35:0] or8;  // d
        reg [0:0] or9;
        reg [35:0] or10;  // d
        reg [0:0] or11;
        reg [0:0] or12;
        reg [35:0] or13;  // d
        reg [35:0] or14;
        reg [0:0] or15;
        reg [0:0] or16;
        reg [0:0] or17;
        reg [0:0] or18;
        reg [35:0] or19;  // o
        reg [0:0] or20;
        reg [0:0] or21;
        reg [33:0] or22;
        reg [34:0] or23;
        reg [33:0] or24;
        reg [34:0] or25;
        reg [35:0] or26;  // o
        reg [71:0] or27;
        reg [1:0] or28;
        localparam ol0 = 1'b1;
        localparam ol1 = 1'b1;
        localparam ol2 = 1'b0;
        localparam ol3 = 35'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx0;
        localparam ol4 = 36'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol5 = 1'b0;
        localparam ol6 = 36'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol7 = 1'b1;
        localparam ol8 = 35'b00000000000000000000000000000000000;
        begin
            or28 = arg_0;
            or1 = arg_1;
            or14 = arg_2;
            // let d = D::<T>::dont_care();
            //
            // let (data_valid, data, ) = match i.data {
            //    Some(data, )#true => (true, data, ),
            //    _#false => (false, T::dont_care(), ),
            // };
            //
            or0 = or1[34:0];
            or2 = or0[34];
            or3 = or0[33:0];
            or4 = { or3, ol0 };
            case (or2)
                1'b1: or5 = or4;
                1'b0: or5 = ol3;
            endcase
            or6 = or5[0];
            or7 = or5[34:1];
            // d.inner.data_in = data;
            //
            or8 = ol4; or8[33:0] = or7;
            // d.inner.void_in = !data_valid;
            //
            or9 = ~(or6);
            or10 = or8; or10[34:34] = or9;
            // d.inner.stop_in = !i.ready.raw;
            //
            or11 = or1[35];
            or12 = ~(or11);
            or13 = or10; or13[35:35] = or12;
            // let o = Out::<T>::dont_care();
            //
            // o.ready = ready(!q.inner.stop_out);
            //
            or15 = or14[35];
            or16 = ~(or15);
            // Ready/* rhdl_fpga::stream::Ready<rhdl_fpga::axi4lite::types::ReadResponse> */ {marker: PhantomData :: < T >, raw: raw,}
            //
            or17 = ol5;
            or18 = or17; or18[0:0] = or16;
            or19 = ol6; or19[35:35] = or18;
            // o.data = pack(!q.inner.void_out, q.inner.data_out);
            //
            or20 = or14[34];
            or21 = ~(or20);
            or22 = or14[33:0];
            // if valid {
            //    Some(data)
            // }
            //  else {
            //    None()
            // }
            //
            //
            // Some(data)
            //
            or24 = or22[33:0];
            or23 = { ol7, or24 };
            // None()
            //
            or25 = (or21) ? (or23) : (ol8);
            or26 = or19; or26[34:0] = or25;
            // (o, d, )
            //
            or27 = { or13, or26 };
            kernel_option_carloni_kernel = or27;
        end
    endfunction
endmodule
// synchronous circuit rhdl_fpga::lid::carloni::Carloni<rhdl_fpga::axi4lite::types::ReadResponse>
module top_read_controller_map_input_buffer_inner(input wire [1:0] clock_reset, input wire [35:0] i, output wire [35:0] o);
    wire [105:0] od;
    wire [69:0] d;
    wire [69:0] q;
    assign o = od[35:0];
    top_read_controller_map_input_buffer_inner_aux_ff c0 (.clock_reset(clock_reset),.i(d[67:34]),.o(q[67:34]));
    top_read_controller_map_input_buffer_inner_main_ff c1 (.clock_reset(clock_reset),.i(d[33:0]),.o(q[33:0]));
    top_read_controller_map_input_buffer_inner_state_ff c2 (.clock_reset(clock_reset),.i(d[69]),.o(q[69]));
    top_read_controller_map_input_buffer_inner_void_ff c3 (.clock_reset(clock_reset),.i(d[68]),.o(q[68]));
    assign od = kernel_carloni_kernel(clock_reset, i, q);
    assign d = od[105:36];
    function [105:0] kernel_carloni_kernel(input reg [1:0] arg_0, input reg [35:0] arg_1, input reg [69:0] arg_2);
        reg [0:0] or0;
        reg [69:0] or1;
        reg [0:0] or2;
        reg [35:0] or3;
        reg [0:0] or4;
        reg [0:0] or5;
        reg [0:0] or6;
        reg [0:0] or7;
        reg [0:0] or8;
        reg [0:0] or9;
        reg [69:0] or10;  // d
        reg [0:0] or11;
        reg [0:0] or12;
        reg [0:0] or13;
        reg [0:0] or14;
        reg [0:0] or15;
        reg [0:0] or16;
        reg [0:0] or17;
        reg [69:0] or18;  // d
        reg [0:0] or19;  // aux_en
        reg [69:0] or20;  // d
        reg [0:0] or21;  // aux_en
        reg [69:0] or22;  // d
        reg [0:0] or23;  // main_en
        reg [0:0] or24;
        reg [69:0] or25;  // d
        reg [69:0] or26;  // d
        reg [0:0] or27;  // main_en
        reg [0:0] or28;  // sel
        reg [0:0] or29;  // stop_out
        reg [0:0] or30;  // aux_en
        reg [69:0] or31;  // d
        reg [0:0] or32;  // main_en
        reg [0:0] or33;  // sel
        reg [0:0] or34;  // stop_out
        reg [33:0] or35;
        reg [33:0] or36;
        reg [33:0] or37;
        reg [69:0] or38;  // d
        reg [33:0] or39;
        reg [33:0] or40;
        reg [33:0] or41;
        reg [33:0] or42;
        reg [33:0] or43;
        reg [69:0] or44;  // d
        reg [0:0] or45;
        reg [0:0] or46;
        reg [0:0] or47;
        reg [0:0] or48;
        reg [69:0] or49;  // d
        reg [33:0] or50;
        reg [35:0] or51;  // o
        reg [0:0] or52;
        reg [35:0] or53;  // o
        reg [35:0] or54;  // o
        reg [0:0] or55;
        reg [1:0] or56;
        reg [0:0] or57;
        reg [35:0] or58;  // o
        reg [35:0] or59;  // o
        reg [35:0] or60;  // o
        reg [105:0] or61;
        localparam ol0 = 70'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol1 = 1'b1;
        localparam ol2 = 1'b1;
        localparam ol3 = 1'b0;
        localparam ol4 = 1'b1;
        localparam ol5 = 1'b0;
        localparam ol6 = 1'b0;
        localparam ol7 = 1'b1;
        localparam ol8 = 1'b0;
        localparam ol9 = 1'b1;
        localparam ol10 = 1'b1;
        localparam ol11 = 1'b1;
        localparam ol12 = 1'b0;
        localparam ol13 = 1'b1;
        localparam ol14 = 1'b0;
        localparam ol15 = 1'b0;
        localparam ol16 = 36'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol17 = 1'b1;
        localparam ol18 = 1'b1;
        begin
            or56 = arg_0;
            or3 = arg_1;
            or1 = arg_2;
            // let d = D::<T>::dont_care();
            //
            // let o = Out::<T>::dont_care();
            //
            // let sel = false;
            //
            // let main_en = false;
            //
            // let aux_en = false;
            //
            // let stop_out = false;
            //
            // let void_out = q.void_ff;
            //
            or0 = or1[68];
            // let will_stall = i.stop_in & (!i.void_in & !void_out);
            //
            or2 = or3[35];
            or4 = or3[34];
            or5 = ~(or4);
            or6 = ~(or0);
            or7 = or5 & or6;
            or8 = or2 & or7;
            // d.state_ff = q.state_ff;
            //
            or9 = or1[69];
            or10 = ol0; or10[69:69] = or9;
            // match q.state_ff {
            //    const State::Run => {
            //       if !i.stop_in | (!i.void_in & void_out) {
            //          main_en = true;
            //       }
            //        else if will_stall {
            //          d.state_ff = State :: Stall;
            //          aux_en = true;
            //       }
            //
            //    }
            //    ,
            //    const State::Stall => {
            //       if i.stop_in {
            //          stop_out = true;
            //       }
            //        else {
            //          sel = true;
            //          main_en = true;
            //          stop_out = true;
            //          d.state_ff = State :: Run;
            //       }
            //
            //    }
            //    ,
            // }
            //
            or11 = or1[69];
            // if !i.stop_in | (!i.void_in & void_out) {
            //    main_en = true;
            // }
            //  else if will_stall {
            //    d.state_ff = State :: Stall;
            //    aux_en = true;
            // }
            //
            //
            or12 = or3[35];
            or13 = ~(or12);
            or14 = or3[34];
            or15 = ~(or14);
            or16 = or15 & or0;
            or17 = or13 | or16;
            // main_en = true;
            //
            // d.state_ff = State :: Stall;
            //
            or18 = or10; or18[69:69] = ol1;
            // aux_en = true;
            //
            or19 = (or8) ? (ol2) : (ol3);
            or20 = (or8) ? (or18) : (or10);
            or21 = (or17) ? (ol3) : (or19);
            or22 = (or17) ? (or10) : (or20);
            or23 = (or17) ? (ol4) : (ol5);
            // if i.stop_in {
            //    stop_out = true;
            // }
            //  else {
            //    sel = true;
            //    main_en = true;
            //    stop_out = true;
            //    d.state_ff = State :: Run;
            // }
            //
            //
            or24 = or3[35];
            // stop_out = true;
            //
            // sel = true;
            //
            // main_en = true;
            //
            // stop_out = true;
            //
            // d.state_ff = State :: Run;
            //
            or25 = or10; or25[69:69] = ol6;
            or26 = (or24) ? (or10) : (or25);
            or27 = (or24) ? (ol5) : (ol7);
            or28 = (or24) ? (ol8) : (ol9);
            or29 = (or24) ? (ol10) : (ol11);
            case (or11)
                1'b0: or30 = or21;
                1'b1: or30 = ol3;
            endcase
            case (or11)
                1'b0: or31 = or22;
                1'b1: or31 = or26;
            endcase
            case (or11)
                1'b0: or32 = or23;
                1'b1: or32 = or27;
            endcase
            case (or11)
                1'b0: or33 = ol8;
                1'b1: or33 = or28;
            endcase
            case (or11)
                1'b0: or34 = ol14;
                1'b1: or34 = or29;
            endcase
            // d.aux_ff = if aux_en {
            //    i.data_in
            // }
            //  else {
            //    q.aux_ff
            // }
            // ;
            //
            // i.data_in
            //
            or35 = or3[33:0];
            // q.aux_ff
            //
            or36 = or1[67:34];
            or37 = (or30) ? (or35) : (or36);
            or38 = or31; or38[67:34] = or37;
            // let d_mux = if sel {
            //    q.aux_ff
            // }
            //  else {
            //    i.data_in
            // }
            // ;
            //
            // q.aux_ff
            //
            or39 = or1[67:34];
            // i.data_in
            //
            or40 = or3[33:0];
            or41 = (or33) ? (or39) : (or40);
            // d.main_ff = if main_en {
            //    d_mux
            // }
            //  else {
            //    q.main_ff
            // }
            // ;
            //
            // d_mux
            //
            // q.main_ff
            //
            or42 = or1[33:0];
            or43 = (or32) ? (or41) : (or42);
            or44 = or38; or44[33:0] = or43;
            // let v_mux = if sel {
            //    false
            // }
            //  else {
            //    i.void_in
            // }
            // ;
            //
            // false
            //
            // i.void_in
            //
            or45 = or3[34];
            or46 = (or33) ? (ol15) : (or45);
            // d.void_ff = if main_en {
            //    v_mux
            // }
            //  else {
            //    q.void_ff
            // }
            // ;
            //
            // v_mux
            //
            // q.void_ff
            //
            or47 = or1[68];
            or48 = (or32) ? (or46) : (or47);
            or49 = or44; or49[68:68] = or48;
            // o.data_out = q.main_ff;
            //
            or50 = or1[33:0];
            or51 = ol16; or51[33:0] = or50;
            // o.void_out = q.void_ff;
            //
            or52 = or1[68];
            or53 = or51; or53[34:34] = or52;
            // o.stop_out = stop_out;
            //
            or54 = or53; or54[35:35] = or34;
            // if cr.reset.any() {
            //    o.void_out = true;
            //    o.stop_out = true;
            // }
            //
            //
            or55 = or56[1];
            or57 = |(or55);
            // o.void_out = true;
            //
            or58 = or54; or58[34:34] = ol17;
            // o.stop_out = true;
            //
            or59 = or58; or59[35:35] = ol18;
            or60 = (or57) ? (or59) : (or54);
            // (o, d, )
            //
            or61 = { or49, or60 };
            kernel_carloni_kernel = or61;
        end
    endfunction
endmodule
//
module top_read_controller_map_input_buffer_inner_aux_ff(input wire [1:0] clock_reset, input wire [33:0] i, output reg [33:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 34'b0000000000000000000000000000000000;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 34'b0000000000000000000000000000000000;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_read_controller_map_input_buffer_inner_main_ff(input wire [1:0] clock_reset, input wire [33:0] i, output reg [33:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 34'b0000000000000000000000000000000000;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 34'b0000000000000000000000000000000000;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_read_controller_map_input_buffer_inner_state_ff(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b0;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b0;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_read_controller_map_input_buffer_inner_void_ff(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b1;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b1;
        end else begin
            o <= i;
        end
    end
endmodule
// synchronous circuit rhdl_fpga::axi4lite::stream::axi_to_rhdl::Axi2Rhdl<rhdl_fpga::axi4lite::types::ReadResponse>
module top_read_controller_outbuf(input wire [1:0] clock_reset, input wire [35:0] i, output wire [35:0] o);
    wire [71:0] od;
    wire [35:0] d;
    wire [35:0] q;
    assign o = od[35:0];
    top_read_controller_outbuf_inbuf c0 (.clock_reset(clock_reset),.i(d[35:0]),.o(q[35:0]));
    assign od = kernel_kernel(clock_reset, i, q);
    assign d = od[71:36];
    function [71:0] kernel_kernel(input reg [1:0] arg_0, input reg [35:0] arg_1, input reg [35:0] arg_2);
        reg [33:0] or0;
        reg [35:0] or1;
        reg [35:0] or2;  // d
        reg [0:0] or3;
        reg [0:0] or4;
        reg [35:0] or5;  // d
        reg [0:0] or6;
        reg [0:0] or7;
        reg [35:0] or8;  // d
        reg [35:0] or9;
        reg [0:0] or10;
        reg [0:0] or11;
        reg [33:0] or12;
        reg [34:0] or13;
        reg [33:0] or14;
        reg [34:0] or15;
        reg [0:0] or16;
        reg [0:0] or17;
        reg [35:0] or18;  // o
        reg [35:0] or19;  // o
        reg [71:0] or20;
        reg [1:0] or21;
        localparam ol0 = 36'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol1 = 1'b1;
        localparam ol2 = 35'b00000000000000000000000000000000000;
        localparam ol3 = 36'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        begin
            or21 = arg_0;
            or1 = arg_1;
            or9 = arg_2;
            // let d = D::<T>::dont_care();
            //
            // d.inbuf.data_in = i.tdata;
            //
            or0 = or1[33:0];
            or2 = ol0; or2[33:0] = or0;
            // d.inbuf.void_in = !i.tvalid;
            //
            or3 = or1[34];
            or4 = ~(or3);
            or5 = or2; or5[34:34] = or4;
            // d.inbuf.stop_in = !i.ready.raw;
            //
            or6 = or1[35];
            or7 = ~(or6);
            or8 = or5; or8[35:35] = or7;
            // let packed = pack(!q.inbuf.void_out, q.inbuf.data_out);
            //
            or10 = or9[34];
            or11 = ~(or10);
            or12 = or9[33:0];
            // if valid {
            //    Some(data)
            // }
            //  else {
            //    None()
            // }
            //
            //
            // Some(data)
            //
            or14 = or12[33:0];
            or13 = { ol1, or14 };
            // None()
            //
            or15 = (or11) ? (or13) : (ol2);
            // let o = Out::<T>::dont_care();
            //
            // o.tready = !q.inbuf.stop_out;
            //
            or16 = or9[35];
            or17 = ~(or16);
            or18 = ol3; or18[35:35] = or17;
            // o.data = packed;
            //
            or19 = or18; or19[34:0] = or15;
            // (o, d, )
            //
            or20 = { or8, or19 };
            kernel_kernel = or20;
        end
    endfunction
endmodule
// synchronous circuit rhdl_fpga::lid::carloni::Carloni<rhdl_fpga::axi4lite::types::ReadResponse>
module top_read_controller_outbuf_inbuf(input wire [1:0] clock_reset, input wire [35:0] i, output wire [35:0] o);
    wire [105:0] od;
    wire [69:0] d;
    wire [69:0] q;
    assign o = od[35:0];
    top_read_controller_outbuf_inbuf_aux_ff c0 (.clock_reset(clock_reset),.i(d[67:34]),.o(q[67:34]));
    top_read_controller_outbuf_inbuf_main_ff c1 (.clock_reset(clock_reset),.i(d[33:0]),.o(q[33:0]));
    top_read_controller_outbuf_inbuf_state_ff c2 (.clock_reset(clock_reset),.i(d[69]),.o(q[69]));
    top_read_controller_outbuf_inbuf_void_ff c3 (.clock_reset(clock_reset),.i(d[68]),.o(q[68]));
    assign od = kernel_carloni_kernel(clock_reset, i, q);
    assign d = od[105:36];
    function [105:0] kernel_carloni_kernel(input reg [1:0] arg_0, input reg [35:0] arg_1, input reg [69:0] arg_2);
        reg [0:0] or0;
        reg [69:0] or1;
        reg [0:0] or2;
        reg [35:0] or3;
        reg [0:0] or4;
        reg [0:0] or5;
        reg [0:0] or6;
        reg [0:0] or7;
        reg [0:0] or8;
        reg [0:0] or9;
        reg [69:0] or10;  // d
        reg [0:0] or11;
        reg [0:0] or12;
        reg [0:0] or13;
        reg [0:0] or14;
        reg [0:0] or15;
        reg [0:0] or16;
        reg [0:0] or17;
        reg [69:0] or18;  // d
        reg [0:0] or19;  // aux_en
        reg [69:0] or20;  // d
        reg [0:0] or21;  // aux_en
        reg [69:0] or22;  // d
        reg [0:0] or23;  // main_en
        reg [0:0] or24;
        reg [69:0] or25;  // d
        reg [69:0] or26;  // d
        reg [0:0] or27;  // main_en
        reg [0:0] or28;  // sel
        reg [0:0] or29;  // stop_out
        reg [0:0] or30;  // aux_en
        reg [69:0] or31;  // d
        reg [0:0] or32;  // main_en
        reg [0:0] or33;  // sel
        reg [0:0] or34;  // stop_out
        reg [33:0] or35;
        reg [33:0] or36;
        reg [33:0] or37;
        reg [69:0] or38;  // d
        reg [33:0] or39;
        reg [33:0] or40;
        reg [33:0] or41;
        reg [33:0] or42;
        reg [33:0] or43;
        reg [69:0] or44;  // d
        reg [0:0] or45;
        reg [0:0] or46;
        reg [0:0] or47;
        reg [0:0] or48;
        reg [69:0] or49;  // d
        reg [33:0] or50;
        reg [35:0] or51;  // o
        reg [0:0] or52;
        reg [35:0] or53;  // o
        reg [35:0] or54;  // o
        reg [0:0] or55;
        reg [1:0] or56;
        reg [0:0] or57;
        reg [35:0] or58;  // o
        reg [35:0] or59;  // o
        reg [35:0] or60;  // o
        reg [105:0] or61;
        localparam ol0 = 70'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol1 = 1'b1;
        localparam ol2 = 1'b1;
        localparam ol3 = 1'b0;
        localparam ol4 = 1'b1;
        localparam ol5 = 1'b0;
        localparam ol6 = 1'b0;
        localparam ol7 = 1'b1;
        localparam ol8 = 1'b0;
        localparam ol9 = 1'b1;
        localparam ol10 = 1'b1;
        localparam ol11 = 1'b1;
        localparam ol12 = 1'b0;
        localparam ol13 = 1'b1;
        localparam ol14 = 1'b0;
        localparam ol15 = 1'b0;
        localparam ol16 = 36'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol17 = 1'b1;
        localparam ol18 = 1'b1;
        begin
            or56 = arg_0;
            or3 = arg_1;
            or1 = arg_2;
            // let d = D::<T>::dont_care();
            //
            // let o = Out::<T>::dont_care();
            //
            // let sel = false;
            //
            // let main_en = false;
            //
            // let aux_en = false;
            //
            // let stop_out = false;
            //
            // let void_out = q.void_ff;
            //
            or0 = or1[68];
            // let will_stall = i.stop_in & (!i.void_in & !void_out);
            //
            or2 = or3[35];
            or4 = or3[34];
            or5 = ~(or4);
            or6 = ~(or0);
            or7 = or5 & or6;
            or8 = or2 & or7;
            // d.state_ff = q.state_ff;
            //
            or9 = or1[69];
            or10 = ol0; or10[69:69] = or9;
            // match q.state_ff {
            //    const State::Run => {
            //       if !i.stop_in | (!i.void_in & void_out) {
            //          main_en = true;
            //       }
            //        else if will_stall {
            //          d.state_ff = State :: Stall;
            //          aux_en = true;
            //       }
            //
            //    }
            //    ,
            //    const State::Stall => {
            //       if i.stop_in {
            //          stop_out = true;
            //       }
            //        else {
            //          sel = true;
            //          main_en = true;
            //          stop_out = true;
            //          d.state_ff = State :: Run;
            //       }
            //
            //    }
            //    ,
            // }
            //
            or11 = or1[69];
            // if !i.stop_in | (!i.void_in & void_out) {
            //    main_en = true;
            // }
            //  else if will_stall {
            //    d.state_ff = State :: Stall;
            //    aux_en = true;
            // }
            //
            //
            or12 = or3[35];
            or13 = ~(or12);
            or14 = or3[34];
            or15 = ~(or14);
            or16 = or15 & or0;
            or17 = or13 | or16;
            // main_en = true;
            //
            // d.state_ff = State :: Stall;
            //
            or18 = or10; or18[69:69] = ol1;
            // aux_en = true;
            //
            or19 = (or8) ? (ol2) : (ol3);
            or20 = (or8) ? (or18) : (or10);
            or21 = (or17) ? (ol3) : (or19);
            or22 = (or17) ? (or10) : (or20);
            or23 = (or17) ? (ol4) : (ol5);
            // if i.stop_in {
            //    stop_out = true;
            // }
            //  else {
            //    sel = true;
            //    main_en = true;
            //    stop_out = true;
            //    d.state_ff = State :: Run;
            // }
            //
            //
            or24 = or3[35];
            // stop_out = true;
            //
            // sel = true;
            //
            // main_en = true;
            //
            // stop_out = true;
            //
            // d.state_ff = State :: Run;
            //
            or25 = or10; or25[69:69] = ol6;
            or26 = (or24) ? (or10) : (or25);
            or27 = (or24) ? (ol5) : (ol7);
            or28 = (or24) ? (ol8) : (ol9);
            or29 = (or24) ? (ol10) : (ol11);
            case (or11)
                1'b0: or30 = or21;
                1'b1: or30 = ol3;
            endcase
            case (or11)
                1'b0: or31 = or22;
                1'b1: or31 = or26;
            endcase
            case (or11)
                1'b0: or32 = or23;
                1'b1: or32 = or27;
            endcase
            case (or11)
                1'b0: or33 = ol8;
                1'b1: or33 = or28;
            endcase
            case (or11)
                1'b0: or34 = ol14;
                1'b1: or34 = or29;
            endcase
            // d.aux_ff = if aux_en {
            //    i.data_in
            // }
            //  else {
            //    q.aux_ff
            // }
            // ;
            //
            // i.data_in
            //
            or35 = or3[33:0];
            // q.aux_ff
            //
            or36 = or1[67:34];
            or37 = (or30) ? (or35) : (or36);
            or38 = or31; or38[67:34] = or37;
            // let d_mux = if sel {
            //    q.aux_ff
            // }
            //  else {
            //    i.data_in
            // }
            // ;
            //
            // q.aux_ff
            //
            or39 = or1[67:34];
            // i.data_in
            //
            or40 = or3[33:0];
            or41 = (or33) ? (or39) : (or40);
            // d.main_ff = if main_en {
            //    d_mux
            // }
            //  else {
            //    q.main_ff
            // }
            // ;
            //
            // d_mux
            //
            // q.main_ff
            //
            or42 = or1[33:0];
            or43 = (or32) ? (or41) : (or42);
            or44 = or38; or44[33:0] = or43;
            // let v_mux = if sel {
            //    false
            // }
            //  else {
            //    i.void_in
            // }
            // ;
            //
            // false
            //
            // i.void_in
            //
            or45 = or3[34];
            or46 = (or33) ? (ol15) : (or45);
            // d.void_ff = if main_en {
            //    v_mux
            // }
            //  else {
            //    q.void_ff
            // }
            // ;
            //
            // v_mux
            //
            // q.void_ff
            //
            or47 = or1[68];
            or48 = (or32) ? (or46) : (or47);
            or49 = or44; or49[68:68] = or48;
            // o.data_out = q.main_ff;
            //
            or50 = or1[33:0];
            or51 = ol16; or51[33:0] = or50;
            // o.void_out = q.void_ff;
            //
            or52 = or1[68];
            or53 = or51; or53[34:34] = or52;
            // o.stop_out = stop_out;
            //
            or54 = or53; or54[35:35] = or34;
            // if cr.reset.any() {
            //    o.void_out = true;
            //    o.stop_out = true;
            // }
            //
            //
            or55 = or56[1];
            or57 = |(or55);
            // o.void_out = true;
            //
            or58 = or54; or58[34:34] = ol17;
            // o.stop_out = true;
            //
            or59 = or58; or59[35:35] = ol18;
            or60 = (or57) ? (or59) : (or54);
            // (o, d, )
            //
            or61 = { or49, or60 };
            kernel_carloni_kernel = or61;
        end
    endfunction
endmodule
//
module top_read_controller_outbuf_inbuf_aux_ff(input wire [1:0] clock_reset, input wire [33:0] i, output reg [33:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 34'b0000000000000000000000000000000000;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 34'b0000000000000000000000000000000000;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_read_controller_outbuf_inbuf_main_ff(input wire [1:0] clock_reset, input wire [33:0] i, output reg [33:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 34'b0000000000000000000000000000000000;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 34'b0000000000000000000000000000000000;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_read_controller_outbuf_inbuf_state_ff(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b0;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b0;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_read_controller_outbuf_inbuf_void_ff(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b1;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b1;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_state(input wire [1:0] clock_reset, input wire [1:0] i, output reg [1:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 2'b00;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 2'b00;
        end else begin
            o <= i;
        end
    end
endmodule
// synchronous circuit rhdl_fpga::axi4lite::core::controller::write::WriteController
module top_write_controller(input wire [1:0] clock_reset, input wire [74:0] i, output wire [74:0] o);
    wire [225:0] od;
    wire [150:0] d;
    wire [150:0] q;
    assign o = od[74:0];
    top_write_controller_addr_buf c0 (.clock_reset(clock_reset),.i(d[104:71]),.o(q[104:71]));
    top_write_controller_data_buf c1 (.clock_reset(clock_reset),.i(d[142:105]),.o(q[142:105]));
    top_write_controller_map c2 (.clock_reset(clock_reset),.i(d[146:143]),.o(q[146:143]));
    top_write_controller_outbuf c3 (.clock_reset(clock_reset),.i(d[150:147]),.o(q[150:147]));
    top_write_controller_tee c4 (.clock_reset(clock_reset),.i(d[70:0]),.o(q[70:0]));
    assign od = kernel_kernel(clock_reset, i, q);
    assign d = od[225:75];
    function [225:0] kernel_kernel(input reg [1:0] arg_0, input reg [74:0] arg_1, input reg [150:0] arg_2);
        reg [68:0] or0;
        reg [74:0] or1;
        reg [0:0] or2;
        reg [67:0] or3;
        reg [31:0] or4;
        reg [35:0] or5;
        reg [67:0] or6;
        reg [68:0] or7;
        reg [67:0] or8;
        reg [150:0] or9;  // d
        reg [150:0] or10;  // d
        reg [70:0] or11;
        reg [150:0] or12;
        reg [32:0] or13;
        reg [150:0] or14;  // d
        reg [33:0] or15;
        reg [0:0] or16;
        reg [150:0] or17;  // d
        reg [70:0] or18;
        reg [36:0] or19;
        reg [150:0] or20;  // d
        reg [37:0] or21;
        reg [0:0] or22;
        reg [150:0] or23;  // d
        reg [70:0] or24;
        reg [0:0] or25;
        reg [0:0] or26;
        reg [0:0] or27;
        reg [74:0] or28;  // o
        reg [33:0] or29;
        reg [31:0] or30;
        reg [74:0] or31;  // o
        reg [33:0] or32;
        reg [0:0] or33;
        reg [74:0] or34;  // o
        reg [4:0] or35;
        reg [0:0] or36;
        reg [150:0] or37;  // d
        reg [37:0] or38;
        reg [35:0] or39;
        reg [31:0] or40;
        reg [74:0] or41;  // o
        reg [37:0] or42;
        reg [35:0] or43;
        reg [3:0] or44;
        reg [74:0] or45;  // o
        reg [37:0] or46;
        reg [0:0] or47;
        reg [74:0] or48;  // o
        reg [4:0] or49;
        reg [0:0] or50;
        reg [150:0] or51;  // d
        reg [4:0] or52;
        reg [1:0] or53;
        reg [150:0] or54;  // d
        reg [4:0] or55;
        reg [0:0] or56;
        reg [150:0] or57;  // d
        reg [3:0] or58;
        reg [0:0] or59;
        reg [74:0] or60;  // o
        reg [3:0] or61;
        reg [2:0] or62;
        reg [150:0] or63;  // d
        reg [3:0] or64;
        reg [0:0] or65;
        reg [150:0] or66;  // d
        reg [3:0] or67;
        reg [2:0] or68;
        reg [74:0] or69;  // o
        reg [0:0] or70;
        reg [150:0] or71;  // d
        reg [225:0] or72;
        reg [1:0] or73;
        localparam ol0 = 1'b1;
        localparam ol1 = 151'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx000000000000000000000000000000000000000000000000000000000000000000000;
        localparam ol2 = 1'b1;
        localparam ol3 = 1'b0;
        localparam ol4 = 75'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        begin
            or73 = arg_0;
            or1 = arg_1;
            or12 = arg_2;
            // let d = D::dont_care();
            //
            // d.tee.data = None();
            //
            // if let Some(cmd, )#true = i.req_data{
            //    d.tee.data = Some((cmd.addr, cmd.strobed_data, ));
            // }
            //
            //
            or0 = or1[73:5];
            or2 = or0[68];
            or3 = or0[67:0];
            // d.tee.data = Some((cmd.addr, cmd.strobed_data, ));
            //
            or4 = or3[31:0];
            or5 = or3[67:32];
            or6 = { or5, or4 };
            or8 = or6[67:0];
            or7 = { ol0, or8 };
            or9 = ol1; or9[68:0] = or7;
            case (or2)
                1'b1: or10 = or9;
                default: or10 = ol1;
            endcase
            // d.addr_buf.data = q.tee.s_data;
            //
            or11 = or12[70:0];
            or13 = or11[32:0];
            or14 = or10; or14[103:71] = or13;
            // d.tee.s_ready = q.addr_buf.ready;
            //
            or15 = or12[104:71];
            or16 = or15[33];
            or17 = or14; or17[69:69] = or16;
            // d.data_buf.data = q.tee.t_data;
            //
            or18 = or12[70:0];
            or19 = or18[69:33];
            or20 = or17; or20[141:105] = or19;
            // d.tee.t_ready = q.data_buf.ready;
            //
            or21 = or12[142:105];
            or22 = or21[37];
            or23 = or20; or23[70:70] = or22;
            // let o = Out::dont_care();
            //
            // o.req_ready = ready(q.tee.ready.raw);
            //
            or24 = or12[70:0];
            or25 = or24[70];
            // Ready/* rhdl_fpga::stream::Ready<rhdl_fpga::axi4lite::types::WriteCommand> */ {marker: PhantomData :: < T >, raw: raw,}
            //
            or26 = ol3;
            or27 = or26; or27[0:0] = or25;
            or28 = ol4; or28[71:71] = or27;
            // o.axi.awaddr = q.addr_buf.tdata;
            //
            or29 = or12[104:71];
            or30 = or29[31:0];
            or31 = or28; or31[31:0] = or30;
            // o.axi.awvalid = q.addr_buf.tvalid;
            //
            or32 = or12[104:71];
            or33 = or32[32];
            or34 = or31; or34[32:32] = or33;
            // d.addr_buf.tready = i.axi.awready;
            //
            or35 = or1[4:0];
            or36 = or35[0];
            or37 = or23; or37[104:104] = or36;
            // o.axi.wdata = q.data_buf.tdata.data;
            //
            or38 = or12[142:105];
            or39 = or38[35:0];
            or40 = or39[31:0];
            or41 = or34; or41[64:33] = or40;
            // o.axi.wstrb = q.data_buf.tdata.strobe;
            //
            or42 = or12[142:105];
            or43 = or42[35:0];
            or44 = or43[35:32];
            or45 = or41; or45[68:65] = or44;
            // o.axi.wvalid = q.data_buf.tvalid;
            //
            or46 = or12[142:105];
            or47 = or46[36];
            or48 = or45; or48[69:69] = or47;
            // d.data_buf.tready = i.axi.wready;
            //
            or49 = or1[4:0];
            or50 = or49[1];
            or51 = or37; or51[142:142] = or50;
            // d.outbuf.tdata = i.axi.bresp;
            //
            or52 = or1[4:0];
            or53 = or52[3:2];
            or54 = or51; or54[148:147] = or53;
            // d.outbuf.tvalid = i.axi.bvalid;
            //
            or55 = or1[4:0];
            or56 = or55[4];
            or57 = or54; or57[149:149] = or56;
            // o.axi.bready = q.outbuf.tready;
            //
            or58 = or12[150:147];
            or59 = or58[3];
            or60 = or48; or60[70:70] = or59;
            // d.map.data = q.outbuf.data;
            //
            or61 = or12[150:147];
            or62 = or61[2:0];
            or63 = or57; or63[145:143] = or62;
            // d.outbuf.ready = q.map.ready;
            //
            or64 = or12[146:143];
            or65 = or64[3];
            or66 = or63; or66[150:150] = or65;
            // o.resp_data = q.map.data;
            //
            or67 = or12[146:143];
            or68 = or67[2:0];
            or69 = or60; or69[74:72] = or68;
            // d.map.ready = i.resp_ready;
            //
            or70 = or1[74];
            or71 = or66; or71[146:146] = or70;
            // (o, d, )
            //
            or72 = { or71, or69 };
            kernel_kernel = or72;
        end
    endfunction
endmodule
// synchronous circuit rhdl_fpga::axi4lite::stream::rhdl_to_axi::Rhdl2Axi<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U32>>
module top_write_controller_addr_buf(input wire [1:0] clock_reset, input wire [33:0] i, output wire [33:0] o);
    wire [67:0] od;
    wire [33:0] d;
    wire [33:0] q;
    assign o = od[33:0];
    top_write_controller_addr_buf_outbuf c0 (.clock_reset(clock_reset),.i(d[33:0]),.o(q[33:0]));
    assign od = kernel_kernel(clock_reset, i, q);
    assign d = od[67:34];
    function [67:0] kernel_kernel(input reg [1:0] arg_0, input reg [33:0] arg_1, input reg [33:0] arg_2);
        reg [32:0] or0;
        reg [33:0] or1;
        reg [0:0] or2;
        reg [31:0] or3;
        reg [31:0] or4;  // tdata
        reg [0:0] or5;  // tvalid
        reg [33:0] or6;  // d
        reg [0:0] or7;
        reg [33:0] or8;  // d
        reg [0:0] or9;
        reg [0:0] or10;
        reg [33:0] or11;  // d
        reg [33:0] or12;
        reg [31:0] or13;
        reg [33:0] or14;  // o
        reg [0:0] or15;
        reg [0:0] or16;
        reg [33:0] or17;  // o
        reg [0:0] or18;
        reg [0:0] or19;
        reg [0:0] or20;
        reg [0:0] or21;
        reg [33:0] or22;  // o
        reg [67:0] or23;
        reg [1:0] or24;
        localparam ol0 = 1'b1;
        localparam ol1 = 32'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol2 = 1'b1;
        localparam ol3 = 1'b0;
        localparam ol4 = 34'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol5 = 34'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol6 = 1'b0;
        begin
            or24 = arg_0;
            or1 = arg_1;
            or12 = arg_2;
            // let tdata = T::dont_care();
            //
            // let tvalid = false;
            //
            // if let Some(data, )#true = i.data{
            //    tdata = data;
            //    tvalid = true;
            // }
            //
            //
            or0 = or1[32:0];
            or2 = or0[32];
            or3 = or0[31:0];
            // tdata = data;
            //
            // tvalid = true;
            //
            case (or2)
                1'b1: or4 = or3;
                default: or4 = ol1;
            endcase
            case (or2)
                1'b1: or5 = ol2;
                default: or5 = ol3;
            endcase
            // let d = D::<T>::dont_care();
            //
            // let o = Out::<T>::dont_care();
            //
            // d.outbuf.data_in = tdata;
            //
            or6 = ol4; or6[31:0] = or4;
            // d.outbuf.void_in = !tvalid;
            //
            or7 = ~(or5);
            or8 = or6; or8[32:32] = or7;
            // d.outbuf.stop_in = !i.tready;
            //
            or9 = or1[33];
            or10 = ~(or9);
            or11 = or8; or11[33:33] = or10;
            // o.tdata = q.outbuf.data_out;
            //
            or13 = or12[31:0];
            or14 = ol5; or14[31:0] = or13;
            // o.tvalid = !q.outbuf.void_out;
            //
            or15 = or12[32];
            or16 = ~(or15);
            or17 = or14; or17[32:32] = or16;
            // o.ready = ready(!q.outbuf.stop_out);
            //
            or18 = or12[33];
            or19 = ~(or18);
            // Ready/* rhdl_fpga::stream::Ready<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U32>> */ {marker: PhantomData :: < T >, raw: raw,}
            //
            or20 = ol6;
            or21 = or20; or21[0:0] = or19;
            or22 = or17; or22[33:33] = or21;
            // (o, d, )
            //
            or23 = { or11, or22 };
            kernel_kernel = or23;
        end
    endfunction
endmodule
// synchronous circuit rhdl_fpga::lid::carloni::Carloni<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U32>>
module top_write_controller_addr_buf_outbuf(input wire [1:0] clock_reset, input wire [33:0] i, output wire [33:0] o);
    wire [99:0] od;
    wire [65:0] d;
    wire [65:0] q;
    assign o = od[33:0];
    top_write_controller_addr_buf_outbuf_aux_ff c0 (.clock_reset(clock_reset),.i(d[63:32]),.o(q[63:32]));
    top_write_controller_addr_buf_outbuf_main_ff c1 (.clock_reset(clock_reset),.i(d[31:0]),.o(q[31:0]));
    top_write_controller_addr_buf_outbuf_state_ff c2 (.clock_reset(clock_reset),.i(d[65]),.o(q[65]));
    top_write_controller_addr_buf_outbuf_void_ff c3 (.clock_reset(clock_reset),.i(d[64]),.o(q[64]));
    assign od = kernel_carloni_kernel(clock_reset, i, q);
    assign d = od[99:34];
    function [99:0] kernel_carloni_kernel(input reg [1:0] arg_0, input reg [33:0] arg_1, input reg [65:0] arg_2);
        reg [0:0] or0;
        reg [65:0] or1;
        reg [0:0] or2;
        reg [33:0] or3;
        reg [0:0] or4;
        reg [0:0] or5;
        reg [0:0] or6;
        reg [0:0] or7;
        reg [0:0] or8;
        reg [0:0] or9;
        reg [65:0] or10;  // d
        reg [0:0] or11;
        reg [0:0] or12;
        reg [0:0] or13;
        reg [0:0] or14;
        reg [0:0] or15;
        reg [0:0] or16;
        reg [0:0] or17;
        reg [65:0] or18;  // d
        reg [0:0] or19;  // aux_en
        reg [65:0] or20;  // d
        reg [0:0] or21;  // aux_en
        reg [65:0] or22;  // d
        reg [0:0] or23;  // main_en
        reg [0:0] or24;
        reg [65:0] or25;  // d
        reg [65:0] or26;  // d
        reg [0:0] or27;  // main_en
        reg [0:0] or28;  // sel
        reg [0:0] or29;  // stop_out
        reg [0:0] or30;  // aux_en
        reg [65:0] or31;  // d
        reg [0:0] or32;  // main_en
        reg [0:0] or33;  // sel
        reg [0:0] or34;  // stop_out
        reg [31:0] or35;
        reg [31:0] or36;
        reg [31:0] or37;
        reg [65:0] or38;  // d
        reg [31:0] or39;
        reg [31:0] or40;
        reg [31:0] or41;
        reg [31:0] or42;
        reg [31:0] or43;
        reg [65:0] or44;  // d
        reg [0:0] or45;
        reg [0:0] or46;
        reg [0:0] or47;
        reg [0:0] or48;
        reg [65:0] or49;  // d
        reg [31:0] or50;
        reg [33:0] or51;  // o
        reg [0:0] or52;
        reg [33:0] or53;  // o
        reg [33:0] or54;  // o
        reg [0:0] or55;
        reg [1:0] or56;
        reg [0:0] or57;
        reg [33:0] or58;  // o
        reg [33:0] or59;  // o
        reg [33:0] or60;  // o
        reg [99:0] or61;
        localparam ol0 = 66'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol1 = 1'b1;
        localparam ol2 = 1'b1;
        localparam ol3 = 1'b0;
        localparam ol4 = 1'b1;
        localparam ol5 = 1'b0;
        localparam ol6 = 1'b0;
        localparam ol7 = 1'b1;
        localparam ol8 = 1'b0;
        localparam ol9 = 1'b1;
        localparam ol10 = 1'b1;
        localparam ol11 = 1'b1;
        localparam ol12 = 1'b0;
        localparam ol13 = 1'b1;
        localparam ol14 = 1'b0;
        localparam ol15 = 1'b0;
        localparam ol16 = 34'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol17 = 1'b1;
        localparam ol18 = 1'b1;
        begin
            or56 = arg_0;
            or3 = arg_1;
            or1 = arg_2;
            // let d = D::<T>::dont_care();
            //
            // let o = Out::<T>::dont_care();
            //
            // let sel = false;
            //
            // let main_en = false;
            //
            // let aux_en = false;
            //
            // let stop_out = false;
            //
            // let void_out = q.void_ff;
            //
            or0 = or1[64];
            // let will_stall = i.stop_in & (!i.void_in & !void_out);
            //
            or2 = or3[33];
            or4 = or3[32];
            or5 = ~(or4);
            or6 = ~(or0);
            or7 = or5 & or6;
            or8 = or2 & or7;
            // d.state_ff = q.state_ff;
            //
            or9 = or1[65];
            or10 = ol0; or10[65:65] = or9;
            // match q.state_ff {
            //    const State::Run => {
            //       if !i.stop_in | (!i.void_in & void_out) {
            //          main_en = true;
            //       }
            //        else if will_stall {
            //          d.state_ff = State :: Stall;
            //          aux_en = true;
            //       }
            //
            //    }
            //    ,
            //    const State::Stall => {
            //       if i.stop_in {
            //          stop_out = true;
            //       }
            //        else {
            //          sel = true;
            //          main_en = true;
            //          stop_out = true;
            //          d.state_ff = State :: Run;
            //       }
            //
            //    }
            //    ,
            // }
            //
            or11 = or1[65];
            // if !i.stop_in | (!i.void_in & void_out) {
            //    main_en = true;
            // }
            //  else if will_stall {
            //    d.state_ff = State :: Stall;
            //    aux_en = true;
            // }
            //
            //
            or12 = or3[33];
            or13 = ~(or12);
            or14 = or3[32];
            or15 = ~(or14);
            or16 = or15 & or0;
            or17 = or13 | or16;
            // main_en = true;
            //
            // d.state_ff = State :: Stall;
            //
            or18 = or10; or18[65:65] = ol1;
            // aux_en = true;
            //
            or19 = (or8) ? (ol2) : (ol3);
            or20 = (or8) ? (or18) : (or10);
            or21 = (or17) ? (ol3) : (or19);
            or22 = (or17) ? (or10) : (or20);
            or23 = (or17) ? (ol4) : (ol5);
            // if i.stop_in {
            //    stop_out = true;
            // }
            //  else {
            //    sel = true;
            //    main_en = true;
            //    stop_out = true;
            //    d.state_ff = State :: Run;
            // }
            //
            //
            or24 = or3[33];
            // stop_out = true;
            //
            // sel = true;
            //
            // main_en = true;
            //
            // stop_out = true;
            //
            // d.state_ff = State :: Run;
            //
            or25 = or10; or25[65:65] = ol6;
            or26 = (or24) ? (or10) : (or25);
            or27 = (or24) ? (ol5) : (ol7);
            or28 = (or24) ? (ol8) : (ol9);
            or29 = (or24) ? (ol10) : (ol11);
            case (or11)
                1'b0: or30 = or21;
                1'b1: or30 = ol3;
            endcase
            case (or11)
                1'b0: or31 = or22;
                1'b1: or31 = or26;
            endcase
            case (or11)
                1'b0: or32 = or23;
                1'b1: or32 = or27;
            endcase
            case (or11)
                1'b0: or33 = ol8;
                1'b1: or33 = or28;
            endcase
            case (or11)
                1'b0: or34 = ol14;
                1'b1: or34 = or29;
            endcase
            // d.aux_ff = if aux_en {
            //    i.data_in
            // }
            //  else {
            //    q.aux_ff
            // }
            // ;
            //
            // i.data_in
            //
            or35 = or3[31:0];
            // q.aux_ff
            //
            or36 = or1[63:32];
            or37 = (or30) ? (or35) : (or36);
            or38 = or31; or38[63:32] = or37;
            // let d_mux = if sel {
            //    q.aux_ff
            // }
            //  else {
            //    i.data_in
            // }
            // ;
            //
            // q.aux_ff
            //
            or39 = or1[63:32];
            // i.data_in
            //
            or40 = or3[31:0];
            or41 = (or33) ? (or39) : (or40);
            // d.main_ff = if main_en {
            //    d_mux
            // }
            //  else {
            //    q.main_ff
            // }
            // ;
            //
            // d_mux
            //
            // q.main_ff
            //
            or42 = or1[31:0];
            or43 = (or32) ? (or41) : (or42);
            or44 = or38; or44[31:0] = or43;
            // let v_mux = if sel {
            //    false
            // }
            //  else {
            //    i.void_in
            // }
            // ;
            //
            // false
            //
            // i.void_in
            //
            or45 = or3[32];
            or46 = (or33) ? (ol15) : (or45);
            // d.void_ff = if main_en {
            //    v_mux
            // }
            //  else {
            //    q.void_ff
            // }
            // ;
            //
            // v_mux
            //
            // q.void_ff
            //
            or47 = or1[64];
            or48 = (or32) ? (or46) : (or47);
            or49 = or44; or49[64:64] = or48;
            // o.data_out = q.main_ff;
            //
            or50 = or1[31:0];
            or51 = ol16; or51[31:0] = or50;
            // o.void_out = q.void_ff;
            //
            or52 = or1[64];
            or53 = or51; or53[32:32] = or52;
            // o.stop_out = stop_out;
            //
            or54 = or53; or54[33:33] = or34;
            // if cr.reset.any() {
            //    o.void_out = true;
            //    o.stop_out = true;
            // }
            //
            //
            or55 = or56[1];
            or57 = |(or55);
            // o.void_out = true;
            //
            or58 = or54; or58[32:32] = ol17;
            // o.stop_out = true;
            //
            or59 = or58; or59[33:33] = ol18;
            or60 = (or57) ? (or59) : (or54);
            // (o, d, )
            //
            or61 = { or49, or60 };
            kernel_carloni_kernel = or61;
        end
    endfunction
endmodule
//
module top_write_controller_addr_buf_outbuf_aux_ff(input wire [1:0] clock_reset, input wire [31:0] i, output reg [31:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 32'b00000000000000000000000000000000;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 32'b00000000000000000000000000000000;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_addr_buf_outbuf_main_ff(input wire [1:0] clock_reset, input wire [31:0] i, output reg [31:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 32'b00000000000000000000000000000000;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 32'b00000000000000000000000000000000;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_addr_buf_outbuf_state_ff(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b0;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b0;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_addr_buf_outbuf_void_ff(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b1;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b1;
        end else begin
            o <= i;
        end
    end
endmodule
// synchronous circuit rhdl_fpga::axi4lite::stream::rhdl_to_axi::Rhdl2Axi<rhdl_fpga::axi4lite::types::StrobedData>
module top_write_controller_data_buf(input wire [1:0] clock_reset, input wire [37:0] i, output wire [37:0] o);
    wire [75:0] od;
    wire [37:0] d;
    wire [37:0] q;
    assign o = od[37:0];
    top_write_controller_data_buf_outbuf c0 (.clock_reset(clock_reset),.i(d[37:0]),.o(q[37:0]));
    assign od = kernel_kernel(clock_reset, i, q);
    assign d = od[75:38];
    function [75:0] kernel_kernel(input reg [1:0] arg_0, input reg [37:0] arg_1, input reg [37:0] arg_2);
        reg [36:0] or0;
        reg [37:0] or1;
        reg [0:0] or2;
        reg [35:0] or3;
        reg [35:0] or4;  // tdata
        reg [0:0] or5;  // tvalid
        reg [37:0] or6;  // d
        reg [0:0] or7;
        reg [37:0] or8;  // d
        reg [0:0] or9;
        reg [0:0] or10;
        reg [37:0] or11;  // d
        reg [37:0] or12;
        reg [35:0] or13;
        reg [37:0] or14;  // o
        reg [0:0] or15;
        reg [0:0] or16;
        reg [37:0] or17;  // o
        reg [0:0] or18;
        reg [0:0] or19;
        reg [0:0] or20;
        reg [0:0] or21;
        reg [37:0] or22;  // o
        reg [75:0] or23;
        reg [1:0] or24;
        localparam ol0 = 1'b1;
        localparam ol1 = 36'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol2 = 1'b1;
        localparam ol3 = 1'b0;
        localparam ol4 = 38'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol5 = 38'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol6 = 1'b0;
        begin
            or24 = arg_0;
            or1 = arg_1;
            or12 = arg_2;
            // let tdata = T::dont_care();
            //
            // let tvalid = false;
            //
            // if let Some(data, )#true = i.data{
            //    tdata = data;
            //    tvalid = true;
            // }
            //
            //
            or0 = or1[36:0];
            or2 = or0[36];
            or3 = or0[35:0];
            // tdata = data;
            //
            // tvalid = true;
            //
            case (or2)
                1'b1: or4 = or3;
                default: or4 = ol1;
            endcase
            case (or2)
                1'b1: or5 = ol2;
                default: or5 = ol3;
            endcase
            // let d = D::<T>::dont_care();
            //
            // let o = Out::<T>::dont_care();
            //
            // d.outbuf.data_in = tdata;
            //
            or6 = ol4; or6[35:0] = or4;
            // d.outbuf.void_in = !tvalid;
            //
            or7 = ~(or5);
            or8 = or6; or8[36:36] = or7;
            // d.outbuf.stop_in = !i.tready;
            //
            or9 = or1[37];
            or10 = ~(or9);
            or11 = or8; or11[37:37] = or10;
            // o.tdata = q.outbuf.data_out;
            //
            or13 = or12[35:0];
            or14 = ol5; or14[35:0] = or13;
            // o.tvalid = !q.outbuf.void_out;
            //
            or15 = or12[36];
            or16 = ~(or15);
            or17 = or14; or17[36:36] = or16;
            // o.ready = ready(!q.outbuf.stop_out);
            //
            or18 = or12[37];
            or19 = ~(or18);
            // Ready/* rhdl_fpga::stream::Ready<rhdl_fpga::axi4lite::types::StrobedData> */ {marker: PhantomData :: < T >, raw: raw,}
            //
            or20 = ol6;
            or21 = or20; or21[0:0] = or19;
            or22 = or17; or22[37:37] = or21;
            // (o, d, )
            //
            or23 = { or11, or22 };
            kernel_kernel = or23;
        end
    endfunction
endmodule
// synchronous circuit rhdl_fpga::lid::carloni::Carloni<rhdl_fpga::axi4lite::types::StrobedData>
module top_write_controller_data_buf_outbuf(input wire [1:0] clock_reset, input wire [37:0] i, output wire [37:0] o);
    wire [111:0] od;
    wire [73:0] d;
    wire [73:0] q;
    assign o = od[37:0];
    top_write_controller_data_buf_outbuf_aux_ff c0 (.clock_reset(clock_reset),.i(d[71:36]),.o(q[71:36]));
    top_write_controller_data_buf_outbuf_main_ff c1 (.clock_reset(clock_reset),.i(d[35:0]),.o(q[35:0]));
    top_write_controller_data_buf_outbuf_state_ff c2 (.clock_reset(clock_reset),.i(d[73]),.o(q[73]));
    top_write_controller_data_buf_outbuf_void_ff c3 (.clock_reset(clock_reset),.i(d[72]),.o(q[72]));
    assign od = kernel_carloni_kernel(clock_reset, i, q);
    assign d = od[111:38];
    function [111:0] kernel_carloni_kernel(input reg [1:0] arg_0, input reg [37:0] arg_1, input reg [73:0] arg_2);
        reg [0:0] or0;
        reg [73:0] or1;
        reg [0:0] or2;
        reg [37:0] or3;
        reg [0:0] or4;
        reg [0:0] or5;
        reg [0:0] or6;
        reg [0:0] or7;
        reg [0:0] or8;
        reg [0:0] or9;
        reg [73:0] or10;  // d
        reg [0:0] or11;
        reg [0:0] or12;
        reg [0:0] or13;
        reg [0:0] or14;
        reg [0:0] or15;
        reg [0:0] or16;
        reg [0:0] or17;
        reg [73:0] or18;  // d
        reg [0:0] or19;  // aux_en
        reg [73:0] or20;  // d
        reg [0:0] or21;  // aux_en
        reg [73:0] or22;  // d
        reg [0:0] or23;  // main_en
        reg [0:0] or24;
        reg [73:0] or25;  // d
        reg [73:0] or26;  // d
        reg [0:0] or27;  // main_en
        reg [0:0] or28;  // sel
        reg [0:0] or29;  // stop_out
        reg [0:0] or30;  // aux_en
        reg [73:0] or31;  // d
        reg [0:0] or32;  // main_en
        reg [0:0] or33;  // sel
        reg [0:0] or34;  // stop_out
        reg [35:0] or35;
        reg [35:0] or36;
        reg [35:0] or37;
        reg [73:0] or38;  // d
        reg [35:0] or39;
        reg [35:0] or40;
        reg [35:0] or41;
        reg [35:0] or42;
        reg [35:0] or43;
        reg [73:0] or44;  // d
        reg [0:0] or45;
        reg [0:0] or46;
        reg [0:0] or47;
        reg [0:0] or48;
        reg [73:0] or49;  // d
        reg [35:0] or50;
        reg [37:0] or51;  // o
        reg [0:0] or52;
        reg [37:0] or53;  // o
        reg [37:0] or54;  // o
        reg [0:0] or55;
        reg [1:0] or56;
        reg [0:0] or57;
        reg [37:0] or58;  // o
        reg [37:0] or59;  // o
        reg [37:0] or60;  // o
        reg [111:0] or61;
        localparam ol0 = 74'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol1 = 1'b1;
        localparam ol2 = 1'b1;
        localparam ol3 = 1'b0;
        localparam ol4 = 1'b1;
        localparam ol5 = 1'b0;
        localparam ol6 = 1'b0;
        localparam ol7 = 1'b1;
        localparam ol8 = 1'b0;
        localparam ol9 = 1'b1;
        localparam ol10 = 1'b1;
        localparam ol11 = 1'b1;
        localparam ol12 = 1'b0;
        localparam ol13 = 1'b1;
        localparam ol14 = 1'b0;
        localparam ol15 = 1'b0;
        localparam ol16 = 38'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol17 = 1'b1;
        localparam ol18 = 1'b1;
        begin
            or56 = arg_0;
            or3 = arg_1;
            or1 = arg_2;
            // let d = D::<T>::dont_care();
            //
            // let o = Out::<T>::dont_care();
            //
            // let sel = false;
            //
            // let main_en = false;
            //
            // let aux_en = false;
            //
            // let stop_out = false;
            //
            // let void_out = q.void_ff;
            //
            or0 = or1[72];
            // let will_stall = i.stop_in & (!i.void_in & !void_out);
            //
            or2 = or3[37];
            or4 = or3[36];
            or5 = ~(or4);
            or6 = ~(or0);
            or7 = or5 & or6;
            or8 = or2 & or7;
            // d.state_ff = q.state_ff;
            //
            or9 = or1[73];
            or10 = ol0; or10[73:73] = or9;
            // match q.state_ff {
            //    const State::Run => {
            //       if !i.stop_in | (!i.void_in & void_out) {
            //          main_en = true;
            //       }
            //        else if will_stall {
            //          d.state_ff = State :: Stall;
            //          aux_en = true;
            //       }
            //
            //    }
            //    ,
            //    const State::Stall => {
            //       if i.stop_in {
            //          stop_out = true;
            //       }
            //        else {
            //          sel = true;
            //          main_en = true;
            //          stop_out = true;
            //          d.state_ff = State :: Run;
            //       }
            //
            //    }
            //    ,
            // }
            //
            or11 = or1[73];
            // if !i.stop_in | (!i.void_in & void_out) {
            //    main_en = true;
            // }
            //  else if will_stall {
            //    d.state_ff = State :: Stall;
            //    aux_en = true;
            // }
            //
            //
            or12 = or3[37];
            or13 = ~(or12);
            or14 = or3[36];
            or15 = ~(or14);
            or16 = or15 & or0;
            or17 = or13 | or16;
            // main_en = true;
            //
            // d.state_ff = State :: Stall;
            //
            or18 = or10; or18[73:73] = ol1;
            // aux_en = true;
            //
            or19 = (or8) ? (ol2) : (ol3);
            or20 = (or8) ? (or18) : (or10);
            or21 = (or17) ? (ol3) : (or19);
            or22 = (or17) ? (or10) : (or20);
            or23 = (or17) ? (ol4) : (ol5);
            // if i.stop_in {
            //    stop_out = true;
            // }
            //  else {
            //    sel = true;
            //    main_en = true;
            //    stop_out = true;
            //    d.state_ff = State :: Run;
            // }
            //
            //
            or24 = or3[37];
            // stop_out = true;
            //
            // sel = true;
            //
            // main_en = true;
            //
            // stop_out = true;
            //
            // d.state_ff = State :: Run;
            //
            or25 = or10; or25[73:73] = ol6;
            or26 = (or24) ? (or10) : (or25);
            or27 = (or24) ? (ol5) : (ol7);
            or28 = (or24) ? (ol8) : (ol9);
            or29 = (or24) ? (ol10) : (ol11);
            case (or11)
                1'b0: or30 = or21;
                1'b1: or30 = ol3;
            endcase
            case (or11)
                1'b0: or31 = or22;
                1'b1: or31 = or26;
            endcase
            case (or11)
                1'b0: or32 = or23;
                1'b1: or32 = or27;
            endcase
            case (or11)
                1'b0: or33 = ol8;
                1'b1: or33 = or28;
            endcase
            case (or11)
                1'b0: or34 = ol14;
                1'b1: or34 = or29;
            endcase
            // d.aux_ff = if aux_en {
            //    i.data_in
            // }
            //  else {
            //    q.aux_ff
            // }
            // ;
            //
            // i.data_in
            //
            or35 = or3[35:0];
            // q.aux_ff
            //
            or36 = or1[71:36];
            or37 = (or30) ? (or35) : (or36);
            or38 = or31; or38[71:36] = or37;
            // let d_mux = if sel {
            //    q.aux_ff
            // }
            //  else {
            //    i.data_in
            // }
            // ;
            //
            // q.aux_ff
            //
            or39 = or1[71:36];
            // i.data_in
            //
            or40 = or3[35:0];
            or41 = (or33) ? (or39) : (or40);
            // d.main_ff = if main_en {
            //    d_mux
            // }
            //  else {
            //    q.main_ff
            // }
            // ;
            //
            // d_mux
            //
            // q.main_ff
            //
            or42 = or1[35:0];
            or43 = (or32) ? (or41) : (or42);
            or44 = or38; or44[35:0] = or43;
            // let v_mux = if sel {
            //    false
            // }
            //  else {
            //    i.void_in
            // }
            // ;
            //
            // false
            //
            // i.void_in
            //
            or45 = or3[36];
            or46 = (or33) ? (ol15) : (or45);
            // d.void_ff = if main_en {
            //    v_mux
            // }
            //  else {
            //    q.void_ff
            // }
            // ;
            //
            // v_mux
            //
            // q.void_ff
            //
            or47 = or1[72];
            or48 = (or32) ? (or46) : (or47);
            or49 = or44; or49[72:72] = or48;
            // o.data_out = q.main_ff;
            //
            or50 = or1[35:0];
            or51 = ol16; or51[35:0] = or50;
            // o.void_out = q.void_ff;
            //
            or52 = or1[72];
            or53 = or51; or53[36:36] = or52;
            // o.stop_out = stop_out;
            //
            or54 = or53; or54[37:37] = or34;
            // if cr.reset.any() {
            //    o.void_out = true;
            //    o.stop_out = true;
            // }
            //
            //
            or55 = or56[1];
            or57 = |(or55);
            // o.void_out = true;
            //
            or58 = or54; or58[36:36] = ol17;
            // o.stop_out = true;
            //
            or59 = or58; or59[37:37] = ol18;
            or60 = (or57) ? (or59) : (or54);
            // (o, d, )
            //
            or61 = { or49, or60 };
            kernel_carloni_kernel = or61;
        end
    endfunction
endmodule
//
module top_write_controller_data_buf_outbuf_aux_ff(input wire [1:0] clock_reset, input wire [35:0] i, output reg [35:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 36'b000000000000000000000000000000000000;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 36'b000000000000000000000000000000000000;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_data_buf_outbuf_main_ff(input wire [1:0] clock_reset, input wire [35:0] i, output reg [35:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 36'b000000000000000000000000000000000000;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 36'b000000000000000000000000000000000000;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_data_buf_outbuf_state_ff(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b0;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b0;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_data_buf_outbuf_void_ff(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b1;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b1;
        end else begin
            o <= i;
        end
    end
endmodule
// synchronous circuit rhdl_fpga::stream::map::Map<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U2>, core::result::Result<(), rhdl_fpga::axi4lite::types::AXI4Error>>
module top_write_controller_map(input wire [1:0] clock_reset, input wire [3:0] i, output wire [3:0] o);
    wire [9:0] od;
    wire [5:0] d;
    wire [5:0] q;
    assign o = od[3:0];
    top_write_controller_map_func c0 (.clock_reset(clock_reset),.i(d[5:4]),.o(q[5:4]));
    top_write_controller_map_input_buffer c1 (.clock_reset(clock_reset),.i(d[3:0]),.o(q[3:0]));
    assign od = kernel_kernel(clock_reset, i, q);
    assign d = od[9:4];
    function [9:0] kernel_kernel(input reg [1:0] arg_0, input reg [3:0] arg_1, input reg [5:0] arg_2);
        reg [2:0] or0;
        reg [3:0] or1;
        reg [5:0] or2;  // d
        reg [0:0] or3;
        reg [0:0] or4;
        reg [0:0] or5;
        reg [5:0] or6;  // d
        reg [3:0] or7;
        reg [5:0] or8;
        reg [2:0] or9;
        reg [0:0] or10;
        reg [1:0] or11;
        reg [5:0] or12;  // d
        reg [1:0] or13;
        reg [2:0] or14;
        reg [1:0] or15;
        reg [5:0] or16;  // d
        reg [5:0] or17;  // d
        reg [2:0] or18;
        reg [3:0] or19;
        reg [0:0] or20;
        reg [3:0] or21;
        reg [3:0] or22;
        reg [9:0] or23;
        reg [1:0] or24;
        localparam ol0 = 6'bxxxxxx;
        localparam ol1 = 1'b0;
        localparam ol2 = 1'b1;
        localparam ol3 = 2'bxx;
        localparam ol4 = 1'b1;
        localparam ol5 = 3'b000;
        localparam ol6 = 4'b0000;
        begin
            or24 = arg_0;
            or1 = arg_1;
            or8 = arg_2;
            // let d = D::<T,S>::dont_care();
            //
            // d.input_buffer.data = i.data;
            //
            or0 = or1[2:0];
            or2 = ol0; or2[2:0] = or0;
            // d.input_buffer.ready = ready_cast(i.ready);
            //
            or3 = or1[3];
            // Ready/* rhdl_fpga::stream::Ready<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U2>> */ {marker: PhantomData :: < T >, raw: input.raw,}
            //
            or4 = ol1;
            or5 = or4; or5[0:0] = or3;
            or6 = or2; or6[3:3] = or5;
            // let o_data = if let Some(data, )#true = q.input_buffer.data{
            //    d.func = data;
            //    Some(q.func)
            // }
            //  else {
            //    d.func = T::dont_care();
            //    None()
            // }
            // ;
            //
            or7 = or8[3:0];
            or9 = or7[2:0];
            or10 = or9[2];
            or11 = or9[1:0];
            // d.func = data;
            //
            or12 = or6; or12[5:4] = or11;
            // Some(q.func)
            //
            or13 = or8[5:4];
            or15 = or13[1:0];
            or14 = { ol2, or15 };
            // d.func = T::dont_care();
            //
            or16 = or6; or16[5:4] = ol3;
            // None()
            //
            case (or10)
                1'b1: or17 = or12;
                default: or17 = or16;
            endcase
            case (or10)
                1'b1: or18 = or14;
                default: or18 = ol5;
            endcase
            // let o = Out/* rhdl_fpga::stream::StreamIO<core::result::Result<(), rhdl_fpga::axi4lite::types::AXI4Error>rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U2>> */ {data: o_data, ready: q.input_buffer.ready,};
            //
            or19 = or8[3:0];
            or20 = or19[3];
            or21 = ol6; or21[2:0] = or18;
            or22 = or21; or22[3:3] = or20;
            // (o, d, )
            //
            or23 = { or17, or22 };
            kernel_kernel = or23;
        end
    endfunction
endmodule
// synchronous circuit rhdl::rhdl_core::circuit::func::Func<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U2>, core::result::Result<(), rhdl_fpga::axi4lite::types::AXI4Error>>
module top_write_controller_map_func(input wire [1:0] clock_reset, input wire [1:0] i, output wire [1:0] o);
    assign o = kernel_map_result(clock_reset, i);
    function [1:0] kernel_map_result(input reg [1:0] arg_0, input reg [1:0] arg_1);
        reg [1:0] or0;
        reg [1:0] or1;
        reg [1:0] or2;
        localparam ol0 = 2'b00;
        localparam ol1 = 2'b10;
        localparam ol2 = 2'b01;
        localparam ol3 = 2'b10;
        localparam ol4 = 2'b10;
        localparam ol5 = 2'b00;
        localparam ol6 = 2'b01;
        begin
            or2 = arg_0;
            or1 = arg_1;
            // match resp {
            //    const response_codes::OKAY => Ok(()),
            //    const response_codes::EXOKAY => Ok(()),
            //    const response_codes::SLVERR => Err(AXI4Error :: SLVERR),
            //    _ => Err(AXI4Error :: DECERR),
            // }
            //
            case (or1)
                2'b00: or0 = ol1;
                2'b01: or0 = ol3;
                2'b10: or0 = ol5;
                default: or0 = ol6;
            endcase
            kernel_map_result = or0;
        end
    endfunction
endmodule
// synchronous circuit rhdl_fpga::stream::stream_buffer::StreamBuffer<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U2>>
module top_write_controller_map_input_buffer(input wire [1:0] clock_reset, input wire [3:0] i, output wire [3:0] o);
    wire [7:0] od;
    wire [3:0] d;
    wire [3:0] q;
    assign o = od[3:0];
    top_write_controller_map_input_buffer_inner c0 (.clock_reset(clock_reset),.i(d[3:0]),.o(q[3:0]));
    assign od = kernel_option_carloni_kernel(clock_reset, i, q);
    assign d = od[7:4];
    function [7:0] kernel_option_carloni_kernel(input reg [1:0] arg_0, input reg [3:0] arg_1, input reg [3:0] arg_2);
        reg [2:0] or0;
        reg [3:0] or1;
        reg [0:0] or2;
        reg [1:0] or3;
        reg [2:0] or4;
        reg [2:0] or5;
        reg [0:0] or6;
        reg [1:0] or7;
        reg [3:0] or8;  // d
        reg [0:0] or9;
        reg [3:0] or10;  // d
        reg [0:0] or11;
        reg [0:0] or12;
        reg [3:0] or13;  // d
        reg [3:0] or14;
        reg [0:0] or15;
        reg [0:0] or16;
        reg [0:0] or17;
        reg [0:0] or18;
        reg [3:0] or19;  // o
        reg [0:0] or20;
        reg [0:0] or21;
        reg [1:0] or22;
        reg [2:0] or23;
        reg [1:0] or24;
        reg [2:0] or25;
        reg [3:0] or26;  // o
        reg [7:0] or27;
        reg [1:0] or28;
        localparam ol0 = 1'b1;
        localparam ol1 = 1'b1;
        localparam ol2 = 1'b0;
        localparam ol3 = 3'bxx0;
        localparam ol4 = 4'bxxxx;
        localparam ol5 = 1'b0;
        localparam ol6 = 4'bxxxx;
        localparam ol7 = 1'b1;
        localparam ol8 = 3'b000;
        begin
            or28 = arg_0;
            or1 = arg_1;
            or14 = arg_2;
            // let d = D::<T>::dont_care();
            //
            // let (data_valid, data, ) = match i.data {
            //    Some(data, )#true => (true, data, ),
            //    _#false => (false, T::dont_care(), ),
            // };
            //
            or0 = or1[2:0];
            or2 = or0[2];
            or3 = or0[1:0];
            or4 = { or3, ol0 };
            case (or2)
                1'b1: or5 = or4;
                1'b0: or5 = ol3;
            endcase
            or6 = or5[0];
            or7 = or5[2:1];
            // d.inner.data_in = data;
            //
            or8 = ol4; or8[1:0] = or7;
            // d.inner.void_in = !data_valid;
            //
            or9 = ~(or6);
            or10 = or8; or10[2:2] = or9;
            // d.inner.stop_in = !i.ready.raw;
            //
            or11 = or1[3];
            or12 = ~(or11);
            or13 = or10; or13[3:3] = or12;
            // let o = Out::<T>::dont_care();
            //
            // o.ready = ready(!q.inner.stop_out);
            //
            or15 = or14[3];
            or16 = ~(or15);
            // Ready/* rhdl_fpga::stream::Ready<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U2>> */ {marker: PhantomData :: < T >, raw: raw,}
            //
            or17 = ol5;
            or18 = or17; or18[0:0] = or16;
            or19 = ol6; or19[3:3] = or18;
            // o.data = pack(!q.inner.void_out, q.inner.data_out);
            //
            or20 = or14[2];
            or21 = ~(or20);
            or22 = or14[1:0];
            // if valid {
            //    Some(data)
            // }
            //  else {
            //    None()
            // }
            //
            //
            // Some(data)
            //
            or24 = or22[1:0];
            or23 = { ol7, or24 };
            // None()
            //
            or25 = (or21) ? (or23) : (ol8);
            or26 = or19; or26[2:0] = or25;
            // (o, d, )
            //
            or27 = { or13, or26 };
            kernel_option_carloni_kernel = or27;
        end
    endfunction
endmodule
// synchronous circuit rhdl_fpga::lid::carloni::Carloni<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U2>>
module top_write_controller_map_input_buffer_inner(input wire [1:0] clock_reset, input wire [3:0] i, output wire [3:0] o);
    wire [9:0] od;
    wire [5:0] d;
    wire [5:0] q;
    assign o = od[3:0];
    top_write_controller_map_input_buffer_inner_aux_ff c0 (.clock_reset(clock_reset),.i(d[3:2]),.o(q[3:2]));
    top_write_controller_map_input_buffer_inner_main_ff c1 (.clock_reset(clock_reset),.i(d[1:0]),.o(q[1:0]));
    top_write_controller_map_input_buffer_inner_state_ff c2 (.clock_reset(clock_reset),.i(d[5]),.o(q[5]));
    top_write_controller_map_input_buffer_inner_void_ff c3 (.clock_reset(clock_reset),.i(d[4]),.o(q[4]));
    assign od = kernel_carloni_kernel(clock_reset, i, q);
    assign d = od[9:4];
    function [9:0] kernel_carloni_kernel(input reg [1:0] arg_0, input reg [3:0] arg_1, input reg [5:0] arg_2);
        reg [0:0] or0;
        reg [5:0] or1;
        reg [0:0] or2;
        reg [3:0] or3;
        reg [0:0] or4;
        reg [0:0] or5;
        reg [0:0] or6;
        reg [0:0] or7;
        reg [0:0] or8;
        reg [0:0] or9;
        reg [5:0] or10;  // d
        reg [0:0] or11;
        reg [0:0] or12;
        reg [0:0] or13;
        reg [0:0] or14;
        reg [0:0] or15;
        reg [0:0] or16;
        reg [0:0] or17;
        reg [5:0] or18;  // d
        reg [0:0] or19;  // aux_en
        reg [5:0] or20;  // d
        reg [0:0] or21;  // aux_en
        reg [5:0] or22;  // d
        reg [0:0] or23;  // main_en
        reg [0:0] or24;
        reg [5:0] or25;  // d
        reg [5:0] or26;  // d
        reg [0:0] or27;  // main_en
        reg [0:0] or28;  // sel
        reg [0:0] or29;  // stop_out
        reg [0:0] or30;  // aux_en
        reg [5:0] or31;  // d
        reg [0:0] or32;  // main_en
        reg [0:0] or33;  // sel
        reg [0:0] or34;  // stop_out
        reg [1:0] or35;
        reg [1:0] or36;
        reg [1:0] or37;
        reg [5:0] or38;  // d
        reg [1:0] or39;
        reg [1:0] or40;
        reg [1:0] or41;
        reg [1:0] or42;
        reg [1:0] or43;
        reg [5:0] or44;  // d
        reg [0:0] or45;
        reg [0:0] or46;
        reg [0:0] or47;
        reg [0:0] or48;
        reg [5:0] or49;  // d
        reg [1:0] or50;
        reg [3:0] or51;  // o
        reg [0:0] or52;
        reg [3:0] or53;  // o
        reg [3:0] or54;  // o
        reg [0:0] or55;
        reg [1:0] or56;
        reg [0:0] or57;
        reg [3:0] or58;  // o
        reg [3:0] or59;  // o
        reg [3:0] or60;  // o
        reg [9:0] or61;
        localparam ol0 = 6'bxxxxxx;
        localparam ol1 = 1'b1;
        localparam ol2 = 1'b1;
        localparam ol3 = 1'b0;
        localparam ol4 = 1'b1;
        localparam ol5 = 1'b0;
        localparam ol6 = 1'b0;
        localparam ol7 = 1'b1;
        localparam ol8 = 1'b0;
        localparam ol9 = 1'b1;
        localparam ol10 = 1'b1;
        localparam ol11 = 1'b1;
        localparam ol12 = 1'b0;
        localparam ol13 = 1'b1;
        localparam ol14 = 1'b0;
        localparam ol15 = 1'b0;
        localparam ol16 = 4'bxxxx;
        localparam ol17 = 1'b1;
        localparam ol18 = 1'b1;
        begin
            or56 = arg_0;
            or3 = arg_1;
            or1 = arg_2;
            // let d = D::<T>::dont_care();
            //
            // let o = Out::<T>::dont_care();
            //
            // let sel = false;
            //
            // let main_en = false;
            //
            // let aux_en = false;
            //
            // let stop_out = false;
            //
            // let void_out = q.void_ff;
            //
            or0 = or1[4];
            // let will_stall = i.stop_in & (!i.void_in & !void_out);
            //
            or2 = or3[3];
            or4 = or3[2];
            or5 = ~(or4);
            or6 = ~(or0);
            or7 = or5 & or6;
            or8 = or2 & or7;
            // d.state_ff = q.state_ff;
            //
            or9 = or1[5];
            or10 = ol0; or10[5:5] = or9;
            // match q.state_ff {
            //    const State::Run => {
            //       if !i.stop_in | (!i.void_in & void_out) {
            //          main_en = true;
            //       }
            //        else if will_stall {
            //          d.state_ff = State :: Stall;
            //          aux_en = true;
            //       }
            //
            //    }
            //    ,
            //    const State::Stall => {
            //       if i.stop_in {
            //          stop_out = true;
            //       }
            //        else {
            //          sel = true;
            //          main_en = true;
            //          stop_out = true;
            //          d.state_ff = State :: Run;
            //       }
            //
            //    }
            //    ,
            // }
            //
            or11 = or1[5];
            // if !i.stop_in | (!i.void_in & void_out) {
            //    main_en = true;
            // }
            //  else if will_stall {
            //    d.state_ff = State :: Stall;
            //    aux_en = true;
            // }
            //
            //
            or12 = or3[3];
            or13 = ~(or12);
            or14 = or3[2];
            or15 = ~(or14);
            or16 = or15 & or0;
            or17 = or13 | or16;
            // main_en = true;
            //
            // d.state_ff = State :: Stall;
            //
            or18 = or10; or18[5:5] = ol1;
            // aux_en = true;
            //
            or19 = (or8) ? (ol2) : (ol3);
            or20 = (or8) ? (or18) : (or10);
            or21 = (or17) ? (ol3) : (or19);
            or22 = (or17) ? (or10) : (or20);
            or23 = (or17) ? (ol4) : (ol5);
            // if i.stop_in {
            //    stop_out = true;
            // }
            //  else {
            //    sel = true;
            //    main_en = true;
            //    stop_out = true;
            //    d.state_ff = State :: Run;
            // }
            //
            //
            or24 = or3[3];
            // stop_out = true;
            //
            // sel = true;
            //
            // main_en = true;
            //
            // stop_out = true;
            //
            // d.state_ff = State :: Run;
            //
            or25 = or10; or25[5:5] = ol6;
            or26 = (or24) ? (or10) : (or25);
            or27 = (or24) ? (ol5) : (ol7);
            or28 = (or24) ? (ol8) : (ol9);
            or29 = (or24) ? (ol10) : (ol11);
            case (or11)
                1'b0: or30 = or21;
                1'b1: or30 = ol3;
            endcase
            case (or11)
                1'b0: or31 = or22;
                1'b1: or31 = or26;
            endcase
            case (or11)
                1'b0: or32 = or23;
                1'b1: or32 = or27;
            endcase
            case (or11)
                1'b0: or33 = ol8;
                1'b1: or33 = or28;
            endcase
            case (or11)
                1'b0: or34 = ol14;
                1'b1: or34 = or29;
            endcase
            // d.aux_ff = if aux_en {
            //    i.data_in
            // }
            //  else {
            //    q.aux_ff
            // }
            // ;
            //
            // i.data_in
            //
            or35 = or3[1:0];
            // q.aux_ff
            //
            or36 = or1[3:2];
            or37 = (or30) ? (or35) : (or36);
            or38 = or31; or38[3:2] = or37;
            // let d_mux = if sel {
            //    q.aux_ff
            // }
            //  else {
            //    i.data_in
            // }
            // ;
            //
            // q.aux_ff
            //
            or39 = or1[3:2];
            // i.data_in
            //
            or40 = or3[1:0];
            or41 = (or33) ? (or39) : (or40);
            // d.main_ff = if main_en {
            //    d_mux
            // }
            //  else {
            //    q.main_ff
            // }
            // ;
            //
            // d_mux
            //
            // q.main_ff
            //
            or42 = or1[1:0];
            or43 = (or32) ? (or41) : (or42);
            or44 = or38; or44[1:0] = or43;
            // let v_mux = if sel {
            //    false
            // }
            //  else {
            //    i.void_in
            // }
            // ;
            //
            // false
            //
            // i.void_in
            //
            or45 = or3[2];
            or46 = (or33) ? (ol15) : (or45);
            // d.void_ff = if main_en {
            //    v_mux
            // }
            //  else {
            //    q.void_ff
            // }
            // ;
            //
            // v_mux
            //
            // q.void_ff
            //
            or47 = or1[4];
            or48 = (or32) ? (or46) : (or47);
            or49 = or44; or49[4:4] = or48;
            // o.data_out = q.main_ff;
            //
            or50 = or1[1:0];
            or51 = ol16; or51[1:0] = or50;
            // o.void_out = q.void_ff;
            //
            or52 = or1[4];
            or53 = or51; or53[2:2] = or52;
            // o.stop_out = stop_out;
            //
            or54 = or53; or54[3:3] = or34;
            // if cr.reset.any() {
            //    o.void_out = true;
            //    o.stop_out = true;
            // }
            //
            //
            or55 = or56[1];
            or57 = |(or55);
            // o.void_out = true;
            //
            or58 = or54; or58[2:2] = ol17;
            // o.stop_out = true;
            //
            or59 = or58; or59[3:3] = ol18;
            or60 = (or57) ? (or59) : (or54);
            // (o, d, )
            //
            or61 = { or49, or60 };
            kernel_carloni_kernel = or61;
        end
    endfunction
endmodule
//
module top_write_controller_map_input_buffer_inner_aux_ff(input wire [1:0] clock_reset, input wire [1:0] i, output reg [1:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 2'b00;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 2'b00;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_map_input_buffer_inner_main_ff(input wire [1:0] clock_reset, input wire [1:0] i, output reg [1:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 2'b00;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 2'b00;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_map_input_buffer_inner_state_ff(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b0;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b0;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_map_input_buffer_inner_void_ff(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b1;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b1;
        end else begin
            o <= i;
        end
    end
endmodule
// synchronous circuit rhdl_fpga::axi4lite::stream::axi_to_rhdl::Axi2Rhdl<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U2>>
module top_write_controller_outbuf(input wire [1:0] clock_reset, input wire [3:0] i, output wire [3:0] o);
    wire [7:0] od;
    wire [3:0] d;
    wire [3:0] q;
    assign o = od[3:0];
    top_write_controller_outbuf_inbuf c0 (.clock_reset(clock_reset),.i(d[3:0]),.o(q[3:0]));
    assign od = kernel_kernel(clock_reset, i, q);
    assign d = od[7:4];
    function [7:0] kernel_kernel(input reg [1:0] arg_0, input reg [3:0] arg_1, input reg [3:0] arg_2);
        reg [1:0] or0;
        reg [3:0] or1;
        reg [3:0] or2;  // d
        reg [0:0] or3;
        reg [0:0] or4;
        reg [3:0] or5;  // d
        reg [0:0] or6;
        reg [0:0] or7;
        reg [3:0] or8;  // d
        reg [3:0] or9;
        reg [0:0] or10;
        reg [0:0] or11;
        reg [1:0] or12;
        reg [2:0] or13;
        reg [1:0] or14;
        reg [2:0] or15;
        reg [0:0] or16;
        reg [0:0] or17;
        reg [3:0] or18;  // o
        reg [3:0] or19;  // o
        reg [7:0] or20;
        reg [1:0] or21;
        localparam ol0 = 4'bxxxx;
        localparam ol1 = 1'b1;
        localparam ol2 = 3'b000;
        localparam ol3 = 4'bxxxx;
        begin
            or21 = arg_0;
            or1 = arg_1;
            or9 = arg_2;
            // let d = D::<T>::dont_care();
            //
            // d.inbuf.data_in = i.tdata;
            //
            or0 = or1[1:0];
            or2 = ol0; or2[1:0] = or0;
            // d.inbuf.void_in = !i.tvalid;
            //
            or3 = or1[2];
            or4 = ~(or3);
            or5 = or2; or5[2:2] = or4;
            // d.inbuf.stop_in = !i.ready.raw;
            //
            or6 = or1[3];
            or7 = ~(or6);
            or8 = or5; or8[3:3] = or7;
            // let packed = pack(!q.inbuf.void_out, q.inbuf.data_out);
            //
            or10 = or9[2];
            or11 = ~(or10);
            or12 = or9[1:0];
            // if valid {
            //    Some(data)
            // }
            //  else {
            //    None()
            // }
            //
            //
            // Some(data)
            //
            or14 = or12[1:0];
            or13 = { ol1, or14 };
            // None()
            //
            or15 = (or11) ? (or13) : (ol2);
            // let o = Out::<T>::dont_care();
            //
            // o.tready = !q.inbuf.stop_out;
            //
            or16 = or9[3];
            or17 = ~(or16);
            or18 = ol3; or18[3:3] = or17;
            // o.data = packed;
            //
            or19 = or18; or19[2:0] = or15;
            // (o, d, )
            //
            or20 = { or8, or19 };
            kernel_kernel = or20;
        end
    endfunction
endmodule
// synchronous circuit rhdl_fpga::lid::carloni::Carloni<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U2>>
module top_write_controller_outbuf_inbuf(input wire [1:0] clock_reset, input wire [3:0] i, output wire [3:0] o);
    wire [9:0] od;
    wire [5:0] d;
    wire [5:0] q;
    assign o = od[3:0];
    top_write_controller_outbuf_inbuf_aux_ff c0 (.clock_reset(clock_reset),.i(d[3:2]),.o(q[3:2]));
    top_write_controller_outbuf_inbuf_main_ff c1 (.clock_reset(clock_reset),.i(d[1:0]),.o(q[1:0]));
    top_write_controller_outbuf_inbuf_state_ff c2 (.clock_reset(clock_reset),.i(d[5]),.o(q[5]));
    top_write_controller_outbuf_inbuf_void_ff c3 (.clock_reset(clock_reset),.i(d[4]),.o(q[4]));
    assign od = kernel_carloni_kernel(clock_reset, i, q);
    assign d = od[9:4];
    function [9:0] kernel_carloni_kernel(input reg [1:0] arg_0, input reg [3:0] arg_1, input reg [5:0] arg_2);
        reg [0:0] or0;
        reg [5:0] or1;
        reg [0:0] or2;
        reg [3:0] or3;
        reg [0:0] or4;
        reg [0:0] or5;
        reg [0:0] or6;
        reg [0:0] or7;
        reg [0:0] or8;
        reg [0:0] or9;
        reg [5:0] or10;  // d
        reg [0:0] or11;
        reg [0:0] or12;
        reg [0:0] or13;
        reg [0:0] or14;
        reg [0:0] or15;
        reg [0:0] or16;
        reg [0:0] or17;
        reg [5:0] or18;  // d
        reg [0:0] or19;  // aux_en
        reg [5:0] or20;  // d
        reg [0:0] or21;  // aux_en
        reg [5:0] or22;  // d
        reg [0:0] or23;  // main_en
        reg [0:0] or24;
        reg [5:0] or25;  // d
        reg [5:0] or26;  // d
        reg [0:0] or27;  // main_en
        reg [0:0] or28;  // sel
        reg [0:0] or29;  // stop_out
        reg [0:0] or30;  // aux_en
        reg [5:0] or31;  // d
        reg [0:0] or32;  // main_en
        reg [0:0] or33;  // sel
        reg [0:0] or34;  // stop_out
        reg [1:0] or35;
        reg [1:0] or36;
        reg [1:0] or37;
        reg [5:0] or38;  // d
        reg [1:0] or39;
        reg [1:0] or40;
        reg [1:0] or41;
        reg [1:0] or42;
        reg [1:0] or43;
        reg [5:0] or44;  // d
        reg [0:0] or45;
        reg [0:0] or46;
        reg [0:0] or47;
        reg [0:0] or48;
        reg [5:0] or49;  // d
        reg [1:0] or50;
        reg [3:0] or51;  // o
        reg [0:0] or52;
        reg [3:0] or53;  // o
        reg [3:0] or54;  // o
        reg [0:0] or55;
        reg [1:0] or56;
        reg [0:0] or57;
        reg [3:0] or58;  // o
        reg [3:0] or59;  // o
        reg [3:0] or60;  // o
        reg [9:0] or61;
        localparam ol0 = 6'bxxxxxx;
        localparam ol1 = 1'b1;
        localparam ol2 = 1'b1;
        localparam ol3 = 1'b0;
        localparam ol4 = 1'b1;
        localparam ol5 = 1'b0;
        localparam ol6 = 1'b0;
        localparam ol7 = 1'b1;
        localparam ol8 = 1'b0;
        localparam ol9 = 1'b1;
        localparam ol10 = 1'b1;
        localparam ol11 = 1'b1;
        localparam ol12 = 1'b0;
        localparam ol13 = 1'b1;
        localparam ol14 = 1'b0;
        localparam ol15 = 1'b0;
        localparam ol16 = 4'bxxxx;
        localparam ol17 = 1'b1;
        localparam ol18 = 1'b1;
        begin
            or56 = arg_0;
            or3 = arg_1;
            or1 = arg_2;
            // let d = D::<T>::dont_care();
            //
            // let o = Out::<T>::dont_care();
            //
            // let sel = false;
            //
            // let main_en = false;
            //
            // let aux_en = false;
            //
            // let stop_out = false;
            //
            // let void_out = q.void_ff;
            //
            or0 = or1[4];
            // let will_stall = i.stop_in & (!i.void_in & !void_out);
            //
            or2 = or3[3];
            or4 = or3[2];
            or5 = ~(or4);
            or6 = ~(or0);
            or7 = or5 & or6;
            or8 = or2 & or7;
            // d.state_ff = q.state_ff;
            //
            or9 = or1[5];
            or10 = ol0; or10[5:5] = or9;
            // match q.state_ff {
            //    const State::Run => {
            //       if !i.stop_in | (!i.void_in & void_out) {
            //          main_en = true;
            //       }
            //        else if will_stall {
            //          d.state_ff = State :: Stall;
            //          aux_en = true;
            //       }
            //
            //    }
            //    ,
            //    const State::Stall => {
            //       if i.stop_in {
            //          stop_out = true;
            //       }
            //        else {
            //          sel = true;
            //          main_en = true;
            //          stop_out = true;
            //          d.state_ff = State :: Run;
            //       }
            //
            //    }
            //    ,
            // }
            //
            or11 = or1[5];
            // if !i.stop_in | (!i.void_in & void_out) {
            //    main_en = true;
            // }
            //  else if will_stall {
            //    d.state_ff = State :: Stall;
            //    aux_en = true;
            // }
            //
            //
            or12 = or3[3];
            or13 = ~(or12);
            or14 = or3[2];
            or15 = ~(or14);
            or16 = or15 & or0;
            or17 = or13 | or16;
            // main_en = true;
            //
            // d.state_ff = State :: Stall;
            //
            or18 = or10; or18[5:5] = ol1;
            // aux_en = true;
            //
            or19 = (or8) ? (ol2) : (ol3);
            or20 = (or8) ? (or18) : (or10);
            or21 = (or17) ? (ol3) : (or19);
            or22 = (or17) ? (or10) : (or20);
            or23 = (or17) ? (ol4) : (ol5);
            // if i.stop_in {
            //    stop_out = true;
            // }
            //  else {
            //    sel = true;
            //    main_en = true;
            //    stop_out = true;
            //    d.state_ff = State :: Run;
            // }
            //
            //
            or24 = or3[3];
            // stop_out = true;
            //
            // sel = true;
            //
            // main_en = true;
            //
            // stop_out = true;
            //
            // d.state_ff = State :: Run;
            //
            or25 = or10; or25[5:5] = ol6;
            or26 = (or24) ? (or10) : (or25);
            or27 = (or24) ? (ol5) : (ol7);
            or28 = (or24) ? (ol8) : (ol9);
            or29 = (or24) ? (ol10) : (ol11);
            case (or11)
                1'b0: or30 = or21;
                1'b1: or30 = ol3;
            endcase
            case (or11)
                1'b0: or31 = or22;
                1'b1: or31 = or26;
            endcase
            case (or11)
                1'b0: or32 = or23;
                1'b1: or32 = or27;
            endcase
            case (or11)
                1'b0: or33 = ol8;
                1'b1: or33 = or28;
            endcase
            case (or11)
                1'b0: or34 = ol14;
                1'b1: or34 = or29;
            endcase
            // d.aux_ff = if aux_en {
            //    i.data_in
            // }
            //  else {
            //    q.aux_ff
            // }
            // ;
            //
            // i.data_in
            //
            or35 = or3[1:0];
            // q.aux_ff
            //
            or36 = or1[3:2];
            or37 = (or30) ? (or35) : (or36);
            or38 = or31; or38[3:2] = or37;
            // let d_mux = if sel {
            //    q.aux_ff
            // }
            //  else {
            //    i.data_in
            // }
            // ;
            //
            // q.aux_ff
            //
            or39 = or1[3:2];
            // i.data_in
            //
            or40 = or3[1:0];
            or41 = (or33) ? (or39) : (or40);
            // d.main_ff = if main_en {
            //    d_mux
            // }
            //  else {
            //    q.main_ff
            // }
            // ;
            //
            // d_mux
            //
            // q.main_ff
            //
            or42 = or1[1:0];
            or43 = (or32) ? (or41) : (or42);
            or44 = or38; or44[1:0] = or43;
            // let v_mux = if sel {
            //    false
            // }
            //  else {
            //    i.void_in
            // }
            // ;
            //
            // false
            //
            // i.void_in
            //
            or45 = or3[2];
            or46 = (or33) ? (ol15) : (or45);
            // d.void_ff = if main_en {
            //    v_mux
            // }
            //  else {
            //    q.void_ff
            // }
            // ;
            //
            // v_mux
            //
            // q.void_ff
            //
            or47 = or1[4];
            or48 = (or32) ? (or46) : (or47);
            or49 = or44; or49[4:4] = or48;
            // o.data_out = q.main_ff;
            //
            or50 = or1[1:0];
            or51 = ol16; or51[1:0] = or50;
            // o.void_out = q.void_ff;
            //
            or52 = or1[4];
            or53 = or51; or53[2:2] = or52;
            // o.stop_out = stop_out;
            //
            or54 = or53; or54[3:3] = or34;
            // if cr.reset.any() {
            //    o.void_out = true;
            //    o.stop_out = true;
            // }
            //
            //
            or55 = or56[1];
            or57 = |(or55);
            // o.void_out = true;
            //
            or58 = or54; or58[2:2] = ol17;
            // o.stop_out = true;
            //
            or59 = or58; or59[3:3] = ol18;
            or60 = (or57) ? (or59) : (or54);
            // (o, d, )
            //
            or61 = { or49, or60 };
            kernel_carloni_kernel = or61;
        end
    endfunction
endmodule
//
module top_write_controller_outbuf_inbuf_aux_ff(input wire [1:0] clock_reset, input wire [1:0] i, output reg [1:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 2'b00;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 2'b00;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_outbuf_inbuf_main_ff(input wire [1:0] clock_reset, input wire [1:0] i, output reg [1:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 2'b00;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 2'b00;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_outbuf_inbuf_state_ff(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b0;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b0;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_outbuf_inbuf_void_ff(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b1;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b1;
        end else begin
            o <= i;
        end
    end
endmodule
// synchronous circuit rhdl_fpga::stream::tee::Tee<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U32>, rhdl_fpga::axi4lite::types::StrobedData>
module top_write_controller_tee(input wire [1:0] clock_reset, input wire [70:0] i, output wire [70:0] o);
    wire [212:0] od;
    wire [141:0] d;
    wire [144:0] q;
    assign o = od[70:0];
    top_write_controller_tee_in_buffer c0 (.clock_reset(clock_reset),.i(d[69:0]),.o(q[70:0]));
    top_write_controller_tee_s_buffer c1 (.clock_reset(clock_reset),.i(d[103:70]),.o(q[105:71]));
    top_write_controller_tee_t_buffer c2 (.clock_reset(clock_reset),.i(d[141:104]),.o(q[144:106]));
    assign od = kernel_kernel(clock_reset, i, q);
    assign d = od[212:71];
    function [212:0] kernel_kernel(input reg [1:0] arg_0, input reg [70:0] arg_1, input reg [144:0] arg_2);
        reg [34:0] or0;
        reg [144:0] or1;
        reg [0:0] or2;
        reg [38:0] or3;
        reg [0:0] or4;
        reg [0:0] or5;
        reg [0:0] or6;
        reg [70:0] or7;
        reg [68:0] or8;
        reg [0:0] or9;
        reg [67:0] or10;
        reg [31:0] or11;
        reg [32:0] or12;
        reg [31:0] or13;
        reg [35:0] or14;
        reg [36:0] or15;
        reg [35:0] or16;
        reg [0:0] or17;  // next
        reg [32:0] or18;  // s_val
        reg [36:0] or19;  // t_val
        reg [0:0] or20;  // next
        reg [32:0] or21;  // s_val
        reg [36:0] or22;  // t_val
        reg [141:0] or23;  // d
        reg [141:0] or24;  // d
        reg [141:0] or25;  // d
        reg [68:0] or26;
        reg [70:0] or27;
        reg [141:0] or28;  // d
        reg [0:0] or29;
        reg [141:0] or30;  // d
        reg [0:0] or31;
        reg [141:0] or32;  // d
        reg [34:0] or33;
        reg [32:0] or34;
        reg [38:0] or35;
        reg [36:0] or36;
        reg [70:0] or37;
        reg [0:0] or38;
        reg [70:0] or39;
        reg [70:0] or40;
        reg [70:0] or41;
        reg [212:0] or42;
        reg [1:0] or43;
        localparam ol0 = 1'b1;
        localparam ol1 = 1'b1;
        localparam ol2 = 1'b1;
        localparam ol3 = 1'b1;
        localparam ol4 = 1'b0;
        localparam ol5 = 33'b000000000000000000000000000000000;
        localparam ol6 = 37'b0000000000000000000000000000000000000;
        localparam ol7 = 142'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol8 = 71'b00000000000000000000000000000000000000000000000000000000000000000000000;
        begin
            or43 = arg_0;
            or27 = arg_1;
            or1 = arg_2;
            // let d = D::<S,T>::dont_care();
            //
            // let s_val = None();
            //
            // let t_val = None();
            //
            // let full = q.s_buffer.full || q.t_buffer.full;
            //
            or0 = or1[105:71];
            or2 = or0[33];
            or3 = or1[144:106];
            or4 = or3[37];
            or5 = or2 | or4;
            // let next = false;
            //
            // if !full {
            //    if let Some(data, )#true = q.in_buffer.data{
            //       s_val = Some(data.0);
            //       t_val = Some(data.1);
            //       next = true;
            //    }
            //
            // }
            //
            //
            or6 = ~(or5);
            // if let Some(data, )#true = q.in_buffer.data{
            //    s_val = Some(data.0);
            //    t_val = Some(data.1);
            //    next = true;
            // }
            //
            //
            or7 = or1[70:0];
            or8 = or7[68:0];
            or9 = or8[68];
            or10 = or8[67:0];
            // s_val = Some(data.0);
            //
            or11 = or10[31:0];
            or13 = or11[31:0];
            or12 = { ol0, or13 };
            // t_val = Some(data.1);
            //
            or14 = or10[67:32];
            or16 = or14[35:0];
            or15 = { ol1, or16 };
            // next = true;
            //
            case (or9)
                1'b1: or17 = ol3;
                default: or17 = ol4;
            endcase
            case (or9)
                1'b1: or18 = or12;
                default: or18 = ol5;
            endcase
            case (or9)
                1'b1: or19 = or15;
                default: or19 = ol6;
            endcase
            or20 = (or6) ? (or17) : (ol4);
            or21 = (or6) ? (or18) : (ol5);
            or22 = (or6) ? (or19) : (ol6);
            // d.s_buffer.data = s_val;
            //
            or23 = ol7; or23[102:70] = or21;
            // d.t_buffer.data = t_val;
            //
            or24 = or23; or24[140:104] = or22;
            // d.in_buffer.next = next;
            //
            or25 = or24; or25[69:69] = or20;
            // d.in_buffer.data = i.data;
            //
            or26 = or27[68:0];
            or28 = or25; or28[68:0] = or26;
            // d.s_buffer.ready = i.s_ready;
            //
            or29 = or27[69];
            or30 = or28; or30[103:103] = or29;
            // d.t_buffer.ready = i.t_ready;
            //
            or31 = or27[70];
            or32 = or30; or32[141:141] = or31;
            // let o = Out/* rhdl_fpga::stream::tee::Out<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U32>rhdl_fpga::axi4lite::types::StrobedData> */ {s_data: q.s_buffer.data, t_data: q.t_buffer.data, ready: q.in_buffer.ready,};
            //
            or33 = or1[105:71];
            or34 = or33[32:0];
            or35 = or1[144:106];
            or36 = or35[36:0];
            or37 = or1[70:0];
            or38 = or37[69];
            or39 = ol8; or39[32:0] = or34;
            or40 = or39; or40[69:33] = or36;
            or41 = or40; or41[70:70] = or38;
            // (o, d, )
            //
            or42 = { or32, or41 };
            kernel_kernel = or42;
        end
    endfunction
endmodule
// synchronous circuit rhdl_fpga::stream::stream_to_fifo::StreamToFIFO<(rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U32>, rhdl_fpga::axi4lite::types::StrobedData)>
module top_write_controller_tee_in_buffer(input wire [1:0] clock_reset, input wire [69:0] i, output wire [70:0] o);
    wire [210:0] od;
    wire [139:0] d;
    wire [139:0] q;
    assign o = od[70:0];
    top_write_controller_tee_in_buffer_one_slot c0 (.clock_reset(clock_reset),.i(d[137:70]),.o(q[137:70]));
    top_write_controller_tee_in_buffer_read_slot c1 (.clock_reset(clock_reset),.i(d[139]),.o(q[139]));
    top_write_controller_tee_in_buffer_state c2 (.clock_reset(clock_reset),.i(d[1:0]),.o(q[1:0]));
    top_write_controller_tee_in_buffer_write_slot c3 (.clock_reset(clock_reset),.i(d[138]),.o(q[138]));
    top_write_controller_tee_in_buffer_zero_slot c4 (.clock_reset(clock_reset),.i(d[69:2]),.o(q[69:2]));
    assign od = kernel_kernel(clock_reset, i, q);
    assign d = od[210:71];
    function [210:0] kernel_kernel(input reg [1:0] arg_0, input reg [69:0] arg_1, input reg [139:0] arg_2);
        reg [0:0] or0;
        reg [69:0] or1;
        reg [68:0] or2;
        reg [0:0] or3;
        reg [0:0] or4;
        reg [1:0] or5;
        reg [139:0] or6;
        reg [1:0] or7;
        reg [1:0] or8;
        reg [1:0] or9;
        reg [1:0] or10;
        reg [1:0] or11;
        reg [1:0] or12;
        reg [139:0] or13;  // d
        reg [1:0] or14;
        reg [0:0] or15;
        reg [1:0] or16;
        reg [0:0] or17;
        reg [0:0] or18;
        reg [0:0] or19;
        reg [67:0] or20;
        reg [139:0] or21;  // d
        reg [67:0] or22;
        reg [139:0] or23;  // d
        reg [68:0] or24;
        reg [0:0] or25;
        reg [67:0] or26;
        reg [0:0] or27;
        reg [139:0] or28;  // d
        reg [139:0] or29;  // d
        reg [139:0] or30;  // d
        reg [139:0] or31;  // d
        reg [139:0] or32;  // d
        reg [0:0] or33;
        reg [0:0] or34;
        reg [139:0] or35;  // d
        reg [0:0] or36;
        reg [0:0] or37;
        reg [139:0] or38;  // d
        reg [1:0] or39;
        reg [0:0] or40;
        reg [0:0] or41;
        reg [0:0] or42;
        reg [67:0] or43;
        reg [68:0] or44;
        reg [67:0] or45;
        reg [70:0] or46;  // o
        reg [67:0] or47;
        reg [68:0] or48;
        reg [67:0] or49;
        reg [70:0] or50;  // o
        reg [70:0] or51;  // o
        reg [70:0] or52;  // o
        reg [0:0] or53;
        reg [0:0] or54;
        reg [70:0] or55;  // o
        reg [1:0] or56;
        reg [0:0] or57;
        reg [70:0] or58;  // o
        reg [210:0] or59;
        reg [1:0] or60;
        localparam ol0 = 1'b1;
        localparam ol1 = 1'b1;
        localparam ol2 = 1'b0;
        localparam ol3 = 1'b0;
        localparam ol4 = 2'b11;
        localparam ol5 = 2'b00;
        localparam ol6 = 2'b01;
        localparam ol7 = 2'b00;
        localparam ol8 = 2'b01;
        localparam ol9 = 2'b01;
        localparam ol10 = 2'b10;
        localparam ol11 = 2'b10;
        localparam ol12 = 2'b00;
        localparam ol13 = 2'b11;
        localparam ol14 = 2'b01;
        localparam ol15 = 2'b01;
        localparam ol16 = 2'b10;
        localparam ol17 = 2'b00;
        localparam ol18 = 2'b01;
        localparam ol19 = 2'b10;
        localparam ol20 = 2'b11;
        localparam ol21 = 2'b11;
        localparam ol22 = 140'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol23 = 2'b10;
        localparam ol24 = 2'b11;
        localparam ol25 = 1'b1;
        localparam ol26 = 2'b00;
        localparam ol27 = 1'b1;
        localparam ol28 = 71'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol29 = 1'b1;
        localparam ol30 = 71'bxx000000000000000000000000000000000000000000000000000000000000000000000;
        localparam ol31 = 1'b0;
        localparam ol32 = 2'b11;
        begin
            or60 = arg_0;
            or1 = arg_1;
            or6 = arg_2;
            // let d = D::<T>::dont_care();
            //
            // let will_read = i.next;
            //
            or0 = or1[69];
            // let can_write = is_some(i.data);
            //
            or2 = or1[68:0];
            // match x {
            //    Some(_, )#true => true,
            //    _#false => false,
            // }
            //
            or3 = or2[68];
            case (or3)
                1'b1: or4 = ol1;
                1'b0: or4 = ol3;
            endcase
            // d.state = match q.state {
            //    const State::Empty => {
            //       if can_write {
            //          State :: OneLoaded
            //       }
            //        else if will_read {
            //          State :: Error
            //       }
            //        else {
            //          State :: Empty
            //       }
            //
            //    }
            //    ,
            //    const State::OneLoaded => match (can_write, will_read, ) {
            //       const (false,false) => State :: OneLoaded,
            //       const (true,false) => State :: TwoLoaded,
            //       const (false,true) => State :: Empty,
            //       const (true,true) => State :: OneLoaded,
            //    },
            //    const State::TwoLoaded => {
            //       if will_read {
            //          State :: OneLoaded
            //       }
            //        else {
            //          State :: TwoLoaded
            //       }
            //
            //    }
            //    ,
            //    const State::Error => State :: Error,
            // };
            //
            or5 = or6[1:0];
            // if can_write {
            //    State :: OneLoaded
            // }
            //  else if will_read {
            //    State :: Error
            // }
            //  else {
            //    State :: Empty
            // }
            //
            //
            // State :: OneLoaded
            //
            // State :: Error
            //
            // State :: Empty
            //
            or7 = (or0) ? (ol4) : (ol5);
            or8 = (or4) ? (ol6) : (or7);
            or9 = { or0, or4 };
            case (or9)
                2'b00: or10 = ol8;
                2'b01: or10 = ol10;
                2'b10: or10 = ol12;
                2'b11: or10 = ol14;
            endcase
            // if will_read {
            //    State :: OneLoaded
            // }
            //  else {
            //    State :: TwoLoaded
            // }
            //
            //
            // State :: OneLoaded
            //
            // State :: TwoLoaded
            //
            or11 = (or0) ? (ol15) : (ol16);
            case (or5)
                2'b00: or12 = or8;
                2'b01: or12 = or10;
                2'b10: or12 = or11;
                2'b11: or12 = ol21;
            endcase
            or13 = ol22; or13[1:0] = or12;
            // let write_is_allowed = q.state != State :: TwoLoaded && q.state != State :: Error;
            //
            or14 = or6[1:0];
            or15 = or14 != ol23;
            or16 = or6[1:0];
            or17 = or16 != ol24;
            or18 = or15 & or17;
            // let will_write = can_write && write_is_allowed;
            //
            or19 = or4 & or18;
            // d.zero_slot = q.zero_slot;
            //
            or20 = or6[69:2];
            or21 = or13; or21[69:2] = or20;
            // d.one_slot = q.one_slot;
            //
            or22 = or6[137:70];
            or23 = or21; or23[137:70] = or22;
            // if let Some(data, )#true = i.data{
            //    if will_write {
            //       if q.write_slot {
            //          d.one_slot = data;
            //       }
            //        else {
            //          d.zero_slot = data;
            //       }
            //
            //    }
            //
            // }
            //
            //
            or24 = or1[68:0];
            or25 = or24[68];
            or26 = or24[67:0];
            // if will_write {
            //    if q.write_slot {
            //       d.one_slot = data;
            //    }
            //     else {
            //       d.zero_slot = data;
            //    }
            //
            // }
            //
            //
            // if q.write_slot {
            //    d.one_slot = data;
            // }
            //  else {
            //    d.zero_slot = data;
            // }
            //
            //
            or27 = or6[138];
            // d.one_slot = data;
            //
            or28 = or23; or28[137:70] = or26;
            // d.zero_slot = data;
            //
            or29 = or23; or29[69:2] = or26;
            or30 = (or27) ? (or28) : (or29);
            or31 = (or19) ? (or30) : (or23);
            case (or25)
                1'b1: or32 = or31;
                default: or32 = or23;
            endcase
            // d.write_slot = will_write ^ q.write_slot;
            //
            or33 = or6[138];
            or34 = or19 ^ or33;
            or35 = or32; or35[138:138] = or34;
            // d.read_slot = will_read ^ q.read_slot;
            //
            or36 = or6[139];
            or37 = or0 ^ or36;
            or38 = or35; or38[139:139] = or37;
            // let o = Out::<T>::dont_care();
            //
            // if q.state == State :: Empty {
            //    o.data = None();
            // }
            //  else if !q.read_slot {
            //    o.data = Some(q.zero_slot);
            // }
            //  else {
            //    o.data = Some(q.one_slot);
            // }
            //
            //
            or39 = or6[1:0];
            or40 = or39 == ol26;
            // o.data = None();
            //
            or41 = or6[139];
            or42 = ~(or41);
            // o.data = Some(q.zero_slot);
            //
            or43 = or6[69:2];
            or45 = or43[67:0];
            or44 = { ol27, or45 };
            or46 = ol28; or46[68:0] = or44;
            // o.data = Some(q.one_slot);
            //
            or47 = or6[137:70];
            or49 = or47[67:0];
            or48 = { ol29, or49 };
            or50 = ol28; or50[68:0] = or48;
            or51 = (or42) ? (or46) : (or50);
            or52 = (or40) ? (ol30) : (or51);
            // o.ready = ready(write_is_allowed);
            //
            // Ready/* rhdl_fpga::stream::Ready<(rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U32>, rhdl_fpga::axi4lite::types::StrobedData)> */ {marker: PhantomData :: < T >, raw: raw,}
            //
            or53 = ol31;
            or54 = or53; or54[0:0] = or18;
            or55 = or52; or55[69:69] = or54;
            // o.error = q.state == State :: Error;
            //
            or56 = or6[1:0];
            or57 = or56 == ol32;
            or58 = or55; or58[70:70] = or57;
            // (o, d, )
            //
            or59 = { or38, or58 };
            kernel_kernel = or59;
        end
    endfunction
endmodule
//
module top_write_controller_tee_in_buffer_one_slot(input wire [1:0] clock_reset, input wire [67:0] i, output reg [67:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 68'b00000000000000000000000000000000000000000000000000000000000000000000;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 68'b00000000000000000000000000000000000000000000000000000000000000000000;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_tee_in_buffer_read_slot(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b0;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b0;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_tee_in_buffer_state(input wire [1:0] clock_reset, input wire [1:0] i, output reg [1:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 2'b00;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 2'b00;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_tee_in_buffer_write_slot(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b0;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b0;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_tee_in_buffer_zero_slot(input wire [1:0] clock_reset, input wire [67:0] i, output reg [67:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 68'b00000000000000000000000000000000000000000000000000000000000000000000;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 68'b00000000000000000000000000000000000000000000000000000000000000000000;
        end else begin
            o <= i;
        end
    end
endmodule
// synchronous circuit rhdl_fpga::stream::fifo_to_stream::FIFOToStream<rhdl::rhdl_bits::bits_impl::Bits<rhdl::rhdl_typenum::consts::U32>>
module top_write_controller_tee_s_buffer(input wire [1:0] clock_reset, input wire [33:0] i, output wire [34:0] o);
    wire [102:0] od;
    wire [67:0] d;
    wire [67:0] q;
    assign o = od[34:0];
    top_write_controller_tee_s_buffer_one_slot c0 (.clock_reset(clock_reset),.i(d[65:34]),.o(q[65:34]));
    top_write_controller_tee_s_buffer_read_slot c1 (.clock_reset(clock_reset),.i(d[67]),.o(q[67]));
    top_write_controller_tee_s_buffer_state c2 (.clock_reset(clock_reset),.i(d[1:0]),.o(q[1:0]));
    top_write_controller_tee_s_buffer_write_slot c3 (.clock_reset(clock_reset),.i(d[66]),.o(q[66]));
    top_write_controller_tee_s_buffer_zero_slot c4 (.clock_reset(clock_reset),.i(d[33:2]),.o(q[33:2]));
    assign od = kernel_kernel(clock_reset, i, q);
    assign d = od[102:35];
    function [102:0] kernel_kernel(input reg [1:0] arg_0, input reg [33:0] arg_1, input reg [67:0] arg_2);
        reg [32:0] or0;
        reg [33:0] or1;
        reg [0:0] or2;
        reg [0:0] or3;
        reg [0:0] or4;
        reg [1:0] or5;
        reg [67:0] or6;
        reg [1:0] or7;
        reg [1:0] or8;
        reg [1:0] or9;
        reg [1:0] or10;
        reg [1:0] or11;
        reg [1:0] or12;
        reg [67:0] or13;  // d
        reg [31:0] or14;
        reg [67:0] or15;  // d
        reg [31:0] or16;
        reg [67:0] or17;  // d
        reg [32:0] or18;
        reg [0:0] or19;
        reg [31:0] or20;
        reg [0:0] or21;
        reg [0:0] or22;
        reg [67:0] or23;  // d
        reg [67:0] or24;  // d
        reg [67:0] or25;  // d
        reg [67:0] or26;  // d
        reg [1:0] or27;
        reg [0:0] or28;
        reg [0:0] or29;
        reg [1:0] or30;
        reg [0:0] or31;
        reg [0:0] or32;
        reg [0:0] or33;
        reg [0:0] or34;
        reg [67:0] or35;  // d
        reg [0:0] or36;
        reg [0:0] or37;
        reg [67:0] or38;  // d
        reg [1:0] or39;
        reg [0:0] or40;
        reg [0:0] or41;
        reg [0:0] or42;
        reg [31:0] or43;
        reg [32:0] or44;
        reg [31:0] or45;
        reg [34:0] or46;  // o
        reg [31:0] or47;
        reg [32:0] or48;
        reg [31:0] or49;
        reg [34:0] or50;  // o
        reg [34:0] or51;  // o
        reg [34:0] or52;  // o
        reg [1:0] or53;
        reg [0:0] or54;
        reg [34:0] or55;  // o
        reg [1:0] or56;
        reg [0:0] or57;
        reg [34:0] or58;  // o
        reg [102:0] or59;
        reg [1:0] or60;
        localparam ol0 = 1'b1;
        localparam ol1 = 1'b1;
        localparam ol2 = 1'b0;
        localparam ol3 = 1'b0;
        localparam ol4 = 2'b01;
        localparam ol5 = 2'b00;
        localparam ol6 = 2'b00;
        localparam ol7 = 2'b01;
        localparam ol8 = 2'b01;
        localparam ol9 = 2'b10;
        localparam ol10 = 2'b10;
        localparam ol11 = 2'b00;
        localparam ol12 = 2'b11;
        localparam ol13 = 2'b01;
        localparam ol14 = 2'b01;
        localparam ol15 = 2'b10;
        localparam ol16 = 2'b11;
        localparam ol17 = 2'b00;
        localparam ol18 = 2'b01;
        localparam ol19 = 2'b10;
        localparam ol20 = 2'b11;
        localparam ol21 = 2'b11;
        localparam ol22 = 68'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol23 = 1'b1;
        localparam ol24 = 2'b11;
        localparam ol25 = 2'b00;
        localparam ol26 = 1'b1;
        localparam ol27 = 35'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol28 = 1'b1;
        localparam ol29 = 35'bxx000000000000000000000000000000000;
        localparam ol30 = 2'b10;
        localparam ol31 = 2'b11;
        begin
            or60 = arg_0;
            or1 = arg_1;
            or6 = arg_2;
            // let d = D::<T>::dont_care();
            //
            // let will_write = is_some(i.data);
            //
            or0 = or1[32:0];
            // match x {
            //    Some(_, )#true => true,
            //    _#false => false,
            // }
            //
            or2 = or0[32];
            case (or2)
                1'b1: or3 = ol1;
                1'b0: or3 = ol3;
            endcase
            // let can_read = i.ready;
            //
            or4 = or1[33];
            // d.state = match q.state {
            //    const State::Empty => {
            //       if will_write {
            //          State :: OneLoaded
            //       }
            //        else {
            //          State :: Empty
            //       }
            //
            //    }
            //    ,
            //    const State::OneLoaded => match (will_write, can_read.raw, ) {
            //       const (false,false) => State :: OneLoaded,
            //       const (true,false) => State :: TwoLoaded,
            //       const (false,true) => State :: Empty,
            //       const (true,true) => State :: OneLoaded,
            //    },
            //    const State::TwoLoaded => {
            //       if will_write {
            //          State :: Error
            //       }
            //        else if can_read.raw {
            //          State :: OneLoaded
            //       }
            //        else {
            //          State :: TwoLoaded
            //       }
            //
            //    }
            //    ,
            //    const State::Error => State :: Error,
            // };
            //
            or5 = or6[1:0];
            // if will_write {
            //    State :: OneLoaded
            // }
            //  else {
            //    State :: Empty
            // }
            //
            //
            // State :: OneLoaded
            //
            // State :: Empty
            //
            or7 = (or3) ? (ol4) : (ol5);
            or8 = { or4, or3 };
            case (or8)
                2'b00: or9 = ol7;
                2'b01: or9 = ol9;
                2'b10: or9 = ol11;
                2'b11: or9 = ol13;
            endcase
            // if will_write {
            //    State :: Error
            // }
            //  else if can_read.raw {
            //    State :: OneLoaded
            // }
            //  else {
            //    State :: TwoLoaded
            // }
            //
            //
            // State :: Error
            //
            // State :: OneLoaded
            //
            // State :: TwoLoaded
            //
            or10 = (or4) ? (ol14) : (ol15);
            or11 = (or3) ? (ol16) : (or10);
            case (or5)
                2'b00: or12 = or7;
                2'b01: or12 = or9;
                2'b10: or12 = or11;
                2'b11: or12 = ol21;
            endcase
            or13 = ol22; or13[1:0] = or12;
            // d.zero_slot = q.zero_slot;
            //
            or14 = or6[33:2];
            or15 = or13; or15[33:2] = or14;
            // d.one_slot = q.one_slot;
            //
            or16 = or6[65:34];
            or17 = or15; or17[65:34] = or16;
            // if let Some(data, )#true = i.data{
            //    if !q.write_slot {
            //       d.zero_slot = data;
            //    }
            //     else {
            //       d.one_slot = data;
            //    }
            //
            // }
            //
            //
            or18 = or1[32:0];
            or19 = or18[32];
            or20 = or18[31:0];
            // if !q.write_slot {
            //    d.zero_slot = data;
            // }
            //  else {
            //    d.one_slot = data;
            // }
            //
            //
            or21 = or6[66];
            or22 = ~(or21);
            // d.zero_slot = data;
            //
            or23 = or17; or23[33:2] = or20;
            // d.one_slot = data;
            //
            or24 = or17; or24[65:34] = or20;
            or25 = (or22) ? (or23) : (or24);
            case (or19)
                1'b1: or26 = or25;
                default: or26 = or17;
            endcase
            // let next_item = can_read.raw && q.state != State :: Empty && q.state != State :: Error;
            //
            or27 = or6[1:0];
            or28 = |(or27);
            or29 = or4 & or28;
            or30 = or6[1:0];
            or31 = or30 != ol24;
            or32 = or29 & or31;
            // d.write_slot = will_write ^ q.write_slot;
            //
            or33 = or6[66];
            or34 = or3 ^ or33;
            or35 = or26; or35[66:66] = or34;
            // d.read_slot = next_item ^ q.read_slot;
            //
            or36 = or6[67];
            or37 = or32 ^ or36;
            or38 = or35; or38[67:67] = or37;
            // let o = Out::<T>::dont_care();
            //
            // if q.state == State :: Empty {
            //    o.data = None()
            // }
            //  else if !q.read_slot {
            //    o.data = Some(q.zero_slot);
            // }
            //  else {
            //    o.data = Some(q.one_slot);
            // }
            // ;
            //
            or39 = or6[1:0];
            or40 = or39 == ol25;
            // o.data = None()
            //
            or41 = or6[67];
            or42 = ~(or41);
            // o.data = Some(q.zero_slot);
            //
            or43 = or6[33:2];
            or45 = or43[31:0];
            or44 = { ol26, or45 };
            or46 = ol27; or46[32:0] = or44;
            // o.data = Some(q.one_slot);
            //
            or47 = or6[65:34];
            or49 = or47[31:0];
            or48 = { ol28, or49 };
            or50 = ol27; or50[32:0] = or48;
            or51 = (or42) ? (or46) : (or50);
            or52 = (or40) ? (ol29) : (or51);
            // o.full = q.state == State :: TwoLoaded;
            //
            or53 = or6[1:0];
            or54 = or53 == ol30;
            or55 = or52; or55[33:33] = or54;
            // o.error = q.state == State :: Error;
            //
            or56 = or6[1:0];
            or57 = or56 == ol31;
            or58 = or55; or58[34:34] = or57;
            // (o, d, )
            //
            or59 = { or38, or58 };
            kernel_kernel = or59;
        end
    endfunction
endmodule
//
module top_write_controller_tee_s_buffer_one_slot(input wire [1:0] clock_reset, input wire [31:0] i, output reg [31:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 32'b00000000000000000000000000000000;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 32'b00000000000000000000000000000000;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_tee_s_buffer_read_slot(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b0;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b0;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_tee_s_buffer_state(input wire [1:0] clock_reset, input wire [1:0] i, output reg [1:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 2'b00;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 2'b00;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_tee_s_buffer_write_slot(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b0;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b0;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_tee_s_buffer_zero_slot(input wire [1:0] clock_reset, input wire [31:0] i, output reg [31:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 32'b00000000000000000000000000000000;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 32'b00000000000000000000000000000000;
        end else begin
            o <= i;
        end
    end
endmodule
// synchronous circuit rhdl_fpga::stream::fifo_to_stream::FIFOToStream<rhdl_fpga::axi4lite::types::StrobedData>
module top_write_controller_tee_t_buffer(input wire [1:0] clock_reset, input wire [37:0] i, output wire [38:0] o);
    wire [114:0] od;
    wire [75:0] d;
    wire [75:0] q;
    assign o = od[38:0];
    top_write_controller_tee_t_buffer_one_slot c0 (.clock_reset(clock_reset),.i(d[73:38]),.o(q[73:38]));
    top_write_controller_tee_t_buffer_read_slot c1 (.clock_reset(clock_reset),.i(d[75]),.o(q[75]));
    top_write_controller_tee_t_buffer_state c2 (.clock_reset(clock_reset),.i(d[1:0]),.o(q[1:0]));
    top_write_controller_tee_t_buffer_write_slot c3 (.clock_reset(clock_reset),.i(d[74]),.o(q[74]));
    top_write_controller_tee_t_buffer_zero_slot c4 (.clock_reset(clock_reset),.i(d[37:2]),.o(q[37:2]));
    assign od = kernel_kernel(clock_reset, i, q);
    assign d = od[114:39];
    function [114:0] kernel_kernel(input reg [1:0] arg_0, input reg [37:0] arg_1, input reg [75:0] arg_2);
        reg [36:0] or0;
        reg [37:0] or1;
        reg [0:0] or2;
        reg [0:0] or3;
        reg [0:0] or4;
        reg [1:0] or5;
        reg [75:0] or6;
        reg [1:0] or7;
        reg [1:0] or8;
        reg [1:0] or9;
        reg [1:0] or10;
        reg [1:0] or11;
        reg [1:0] or12;
        reg [75:0] or13;  // d
        reg [35:0] or14;
        reg [75:0] or15;  // d
        reg [35:0] or16;
        reg [75:0] or17;  // d
        reg [36:0] or18;
        reg [0:0] or19;
        reg [35:0] or20;
        reg [0:0] or21;
        reg [0:0] or22;
        reg [75:0] or23;  // d
        reg [75:0] or24;  // d
        reg [75:0] or25;  // d
        reg [75:0] or26;  // d
        reg [1:0] or27;
        reg [0:0] or28;
        reg [0:0] or29;
        reg [1:0] or30;
        reg [0:0] or31;
        reg [0:0] or32;
        reg [0:0] or33;
        reg [0:0] or34;
        reg [75:0] or35;  // d
        reg [0:0] or36;
        reg [0:0] or37;
        reg [75:0] or38;  // d
        reg [1:0] or39;
        reg [0:0] or40;
        reg [0:0] or41;
        reg [0:0] or42;
        reg [35:0] or43;
        reg [36:0] or44;
        reg [35:0] or45;
        reg [38:0] or46;  // o
        reg [35:0] or47;
        reg [36:0] or48;
        reg [35:0] or49;
        reg [38:0] or50;  // o
        reg [38:0] or51;  // o
        reg [38:0] or52;  // o
        reg [1:0] or53;
        reg [0:0] or54;
        reg [38:0] or55;  // o
        reg [1:0] or56;
        reg [0:0] or57;
        reg [38:0] or58;  // o
        reg [114:0] or59;
        reg [1:0] or60;
        localparam ol0 = 1'b1;
        localparam ol1 = 1'b1;
        localparam ol2 = 1'b0;
        localparam ol3 = 1'b0;
        localparam ol4 = 2'b01;
        localparam ol5 = 2'b00;
        localparam ol6 = 2'b00;
        localparam ol7 = 2'b01;
        localparam ol8 = 2'b01;
        localparam ol9 = 2'b10;
        localparam ol10 = 2'b10;
        localparam ol11 = 2'b00;
        localparam ol12 = 2'b11;
        localparam ol13 = 2'b01;
        localparam ol14 = 2'b01;
        localparam ol15 = 2'b10;
        localparam ol16 = 2'b11;
        localparam ol17 = 2'b00;
        localparam ol18 = 2'b01;
        localparam ol19 = 2'b10;
        localparam ol20 = 2'b11;
        localparam ol21 = 2'b11;
        localparam ol22 = 76'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol23 = 1'b1;
        localparam ol24 = 2'b11;
        localparam ol25 = 2'b00;
        localparam ol26 = 1'b1;
        localparam ol27 = 39'bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
        localparam ol28 = 1'b1;
        localparam ol29 = 39'bxx0000000000000000000000000000000000000;
        localparam ol30 = 2'b10;
        localparam ol31 = 2'b11;
        begin
            or60 = arg_0;
            or1 = arg_1;
            or6 = arg_2;
            // let d = D::<T>::dont_care();
            //
            // let will_write = is_some(i.data);
            //
            or0 = or1[36:0];
            // match x {
            //    Some(_, )#true => true,
            //    _#false => false,
            // }
            //
            or2 = or0[36];
            case (or2)
                1'b1: or3 = ol1;
                1'b0: or3 = ol3;
            endcase
            // let can_read = i.ready;
            //
            or4 = or1[37];
            // d.state = match q.state {
            //    const State::Empty => {
            //       if will_write {
            //          State :: OneLoaded
            //       }
            //        else {
            //          State :: Empty
            //       }
            //
            //    }
            //    ,
            //    const State::OneLoaded => match (will_write, can_read.raw, ) {
            //       const (false,false) => State :: OneLoaded,
            //       const (true,false) => State :: TwoLoaded,
            //       const (false,true) => State :: Empty,
            //       const (true,true) => State :: OneLoaded,
            //    },
            //    const State::TwoLoaded => {
            //       if will_write {
            //          State :: Error
            //       }
            //        else if can_read.raw {
            //          State :: OneLoaded
            //       }
            //        else {
            //          State :: TwoLoaded
            //       }
            //
            //    }
            //    ,
            //    const State::Error => State :: Error,
            // };
            //
            or5 = or6[1:0];
            // if will_write {
            //    State :: OneLoaded
            // }
            //  else {
            //    State :: Empty
            // }
            //
            //
            // State :: OneLoaded
            //
            // State :: Empty
            //
            or7 = (or3) ? (ol4) : (ol5);
            or8 = { or4, or3 };
            case (or8)
                2'b00: or9 = ol7;
                2'b01: or9 = ol9;
                2'b10: or9 = ol11;
                2'b11: or9 = ol13;
            endcase
            // if will_write {
            //    State :: Error
            // }
            //  else if can_read.raw {
            //    State :: OneLoaded
            // }
            //  else {
            //    State :: TwoLoaded
            // }
            //
            //
            // State :: Error
            //
            // State :: OneLoaded
            //
            // State :: TwoLoaded
            //
            or10 = (or4) ? (ol14) : (ol15);
            or11 = (or3) ? (ol16) : (or10);
            case (or5)
                2'b00: or12 = or7;
                2'b01: or12 = or9;
                2'b10: or12 = or11;
                2'b11: or12 = ol21;
            endcase
            or13 = ol22; or13[1:0] = or12;
            // d.zero_slot = q.zero_slot;
            //
            or14 = or6[37:2];
            or15 = or13; or15[37:2] = or14;
            // d.one_slot = q.one_slot;
            //
            or16 = or6[73:38];
            or17 = or15; or17[73:38] = or16;
            // if let Some(data, )#true = i.data{
            //    if !q.write_slot {
            //       d.zero_slot = data;
            //    }
            //     else {
            //       d.one_slot = data;
            //    }
            //
            // }
            //
            //
            or18 = or1[36:0];
            or19 = or18[36];
            or20 = or18[35:0];
            // if !q.write_slot {
            //    d.zero_slot = data;
            // }
            //  else {
            //    d.one_slot = data;
            // }
            //
            //
            or21 = or6[74];
            or22 = ~(or21);
            // d.zero_slot = data;
            //
            or23 = or17; or23[37:2] = or20;
            // d.one_slot = data;
            //
            or24 = or17; or24[73:38] = or20;
            or25 = (or22) ? (or23) : (or24);
            case (or19)
                1'b1: or26 = or25;
                default: or26 = or17;
            endcase
            // let next_item = can_read.raw && q.state != State :: Empty && q.state != State :: Error;
            //
            or27 = or6[1:0];
            or28 = |(or27);
            or29 = or4 & or28;
            or30 = or6[1:0];
            or31 = or30 != ol24;
            or32 = or29 & or31;
            // d.write_slot = will_write ^ q.write_slot;
            //
            or33 = or6[74];
            or34 = or3 ^ or33;
            or35 = or26; or35[74:74] = or34;
            // d.read_slot = next_item ^ q.read_slot;
            //
            or36 = or6[75];
            or37 = or32 ^ or36;
            or38 = or35; or38[75:75] = or37;
            // let o = Out::<T>::dont_care();
            //
            // if q.state == State :: Empty {
            //    o.data = None()
            // }
            //  else if !q.read_slot {
            //    o.data = Some(q.zero_slot);
            // }
            //  else {
            //    o.data = Some(q.one_slot);
            // }
            // ;
            //
            or39 = or6[1:0];
            or40 = or39 == ol25;
            // o.data = None()
            //
            or41 = or6[75];
            or42 = ~(or41);
            // o.data = Some(q.zero_slot);
            //
            or43 = or6[37:2];
            or45 = or43[35:0];
            or44 = { ol26, or45 };
            or46 = ol27; or46[36:0] = or44;
            // o.data = Some(q.one_slot);
            //
            or47 = or6[73:38];
            or49 = or47[35:0];
            or48 = { ol28, or49 };
            or50 = ol27; or50[36:0] = or48;
            or51 = (or42) ? (or46) : (or50);
            or52 = (or40) ? (ol29) : (or51);
            // o.full = q.state == State :: TwoLoaded;
            //
            or53 = or6[1:0];
            or54 = or53 == ol30;
            or55 = or52; or55[37:37] = or54;
            // o.error = q.state == State :: Error;
            //
            or56 = or6[1:0];
            or57 = or56 == ol31;
            or58 = or55; or58[38:38] = or57;
            // (o, d, )
            //
            or59 = { or38, or58 };
            kernel_kernel = or59;
        end
    endfunction
endmodule
//
module top_write_controller_tee_t_buffer_one_slot(input wire [1:0] clock_reset, input wire [35:0] i, output reg [35:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 36'b000000000000000000000000000000000000;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 36'b000000000000000000000000000000000000;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_tee_t_buffer_read_slot(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b0;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b0;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_tee_t_buffer_state(input wire [1:0] clock_reset, input wire [1:0] i, output reg [1:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 2'b00;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 2'b00;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_tee_t_buffer_write_slot(input wire [1:0] clock_reset, input wire [0:0] i, output reg [0:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 1'b0;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 1'b0;
        end else begin
            o <= i;
        end
    end
endmodule
//
module top_write_controller_tee_t_buffer_zero_slot(input wire [1:0] clock_reset, input wire [35:0] i, output reg [35:0] o);
    wire [0:0] clock;
    wire [0:0] reset;
    initial begin
        o = 36'b000000000000000000000000000000000000;
    end
    assign clock = clock_reset[0];
    assign reset = clock_reset[1];
    always @(posedge clock) begin
        if (reset)
        begin
            o <= 36'b000000000000000000000000000000000000;
        end else begin
            o <= i;
        end
    end
endmodule
