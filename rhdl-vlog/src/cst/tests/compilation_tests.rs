use crate::cst::{
    ModuleList,
    tests::common::{test_compilation, test_parse},
};

/// Test that an empty module declaration generates compilable code
#[test]
fn test_empty_module() {
    let module = syn::parse_str::<ModuleList>(
        "
            module foo;
            endmodule
    ",
    )
    .unwrap();
    test_compilation("empty", module);
}

#[test]
fn test_module_with_ports() {
    let module = syn::parse_str::<ModuleList>(
        "
            module foo(input wire[1:0] a, output wire[1:0] b);
            endmodule
    ",
    )
    .unwrap();
    test_compilation("module_with_ports", module);
}

#[test]
fn test_multiple_modules_with_ports() {
    let modules = syn::parse_str::<ModuleList>(
        "
            module foo(input wire[1:0] a, output wire[1:0] b);
            endmodule

            module bar(input wire[1:0] c, output wire[1:0] d);
            endmodule
    ",
    )
    .unwrap();
    test_compilation("multiple_modules_with_ports", modules);
}

#[test]
// Test with and without signed, with widths and both reg and wire
fn test_module_with_different_port_types() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output wire[1:0] b, inout reg[3:0] c);
            endmodule

            module bar(input wire signed[1:0] c, output reg signed [1:0] d);
            endmodule
    ",
    )?;
    test_compilation("module_with_different_port_types", modules);
    Ok(())
}

#[test]
fn test_if_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output wire[1:0] b);
            if (a) begin
                b = 1;
            end else begin
                b = 0;
            end
            endmodule
    ",
    )?;
    test_compilation("if_statement", modules);
    Ok(())
}

#[test]
fn test_if_else_if_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output wire[1:0] b);
            if (a) begin
                b = 1;
            end else if (a == 1) begin
                b = 2;
            end else begin
                b = 0;
            end
            endmodule
    ",
    )?;
    test_compilation("if_else_if_statement", modules);
    Ok(())
}

#[test]
fn test_if_no_else_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output wire[1:0] b);
            if (a) begin
                b = 1;
            end
            endmodule
    ",
    )?;
    test_compilation("if_no_else_statement", modules);
    Ok(())
}

#[test]
fn test_always_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output reg[1:0] b);
            always @(posedge a, b) begin
                b <= 1;
            end
            endmodule
    ",
    )?;
    test_compilation("always_statement", modules);
    Ok(())
}

#[test]
fn test_case_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output reg[1:0] b);
            case (a)
                2'b00: b = 1;
                2'b01: b = 2;
                2'b10: b = 3;
                default: b = 4;
            endcase
            endmodule
    ",
    )?;
    test_compilation("case_statement", modules);
    Ok(())
}

#[test]
fn test_local_param_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output reg[1:0] b);
            localparam my_param = 5'b1_1001;
            always @(posedge a) begin
                b <= my_param;
            end
            endmodule
    ",
    )?;
    test_compilation("local_param_statement", modules);
    Ok(())
}

#[test]
fn test_continuous_assign_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output wire[1:0] b);
            assign b = a + 1;
            endmodule
    ",
    )?;
    test_compilation("continuous_assign_statement", modules);
    Ok(())
}

#[test]
fn test_non_block_assignment_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output reg[1:0] b);
            always @(posedge a) begin
                b <= 1;
            end
            endmodule
    ",
    )?;
    test_compilation("non_block_assignment_statement", modules);
    Ok(())
}

#[test]
fn test_instance_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output reg[1:0] b);
                bar_0 bar(.c(a), .d(b));
            endmodule
    ",
    )?;
    test_compilation("instance_statement", modules);
    Ok(())
}

#[test]
fn test_delay_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output reg[1:0] b);
            always @(posedge a, negedge b, *) begin
                b <= 1;
                # 10;
            end
            endmodule
    ",
    )?;
    test_compilation("delay_statement", modules);
    Ok(())
}

#[test]
fn test_concat_assignment_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, input wire[1:0] c, output reg[1:0] b);
                {a, c} = {1'b0, a};
            endmodule
    ",
    )?;
    test_compilation("concat_assignment_statement", modules);
    Ok(())
}

#[test]
fn test_function_call_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[1:0] a, output reg[1:0] b);
            $my_function(a, b);
            $finish;
            $display(\"Hello World\");
            endmodule
    ",
    )?;
    test_compilation("function_call_statement", modules);
    Ok(())
}

#[test]
fn test_splice_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            a[1] = b;
            a[1:0] = {b, b};
            endmodule
    ",
    )?;
    test_compilation("splice_statement", modules);
    Ok(())
}

#[test]
fn test_dynamic_splice_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            a[b +: 1] = 1;
            endmodule
    ",
    )?;
    test_compilation("dynamic_splice_statement", modules);
    Ok(())
}

#[test]
fn test_local_declaration_statement() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            wire [4:0] val1;
            reg signed [3:0] val2;
            endmodule
    ",
    )?;
    test_compilation("local_declaration_statement", modules);
    Ok(())
}

#[test]
fn test_function_def_in_module() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            function [1:0] my_function(input wire[1:0] x);
                assign my_function = x + 1;
            endfunction
            b = my_function(a);
            endmodule
    ",
    )?;
    test_compilation("function_def_in_module", modules);
    Ok(())
}

#[test]
fn test_initial_statement_in_module() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            initial begin
                b = 0;
            end
            endmodule
    ",
    )?;
    test_compilation("initial_statement_in_module", modules);
    Ok(())
}

#[test]
fn test_unary_expressions() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            always @(*) begin
                b = +a;
                b = -a;
                b = ~a;
                b = !a;
                b = &a;
                b = ^a;
                b = |a;
            end
            endmodule
    ",
    )?;
    test_compilation("unary_expressions", modules);
    Ok(())
}

#[test]
fn test_ternary_expression() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            always @(*) begin
                b = (a == 1) ? 2 : 3;
            end
            endmodule
    ",
    )?;
    test_compilation("ternary_expression", modules);
    Ok(())
}

#[test]
fn test_replica_expression() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            always @(*) begin
                b = {3{a}};
            end
            endmodule
    ",
    )?;
    test_compilation("replica_expression", modules);
    Ok(())
}

#[test]
fn test_index_expressions() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            always @(*) begin
                b = a[1];
                b = a[1:0];
                b = a[b+:2];
            end
            endmodule
    ",
    )?;
    test_compilation("index_expressions", modules);
    Ok(())
}

#[test]
fn test_string_expression() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        r#"
            module foo(input wire[2:0] a, output reg[1:0] b);
            always @(*) begin
                $display("Hello, World!");
            end
            endmodule
            "#,
    )?;
    test_compilation("string_expression", modules);
    Ok(())
}

#[test]
fn test_binary_expressions() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module foo(input wire[2:0] a, output reg[1:0] b);
            always @(*) begin
                b = a << 1;
                b = a >>> 2;
                b = a >> 2;
                b = a && a;
                b = a || a;
                b = a === 1;
                b = a !== 1;
                b = a != 1;
                b = a == 1;
                b = a >= 1;
                b = a <= 1;
                b = a > 1;
                b = a < 1;
                b = a + b;
                b = a - b;
                b = a & b;
                b = a | b;
                b = a ^ b;
                b = a % b;
                b = a * b;
            end
            endmodule
    ",
    )?;
    test_compilation("binary_expressions", modules);
    Ok(())
}

#[test]
fn test_mixed_assignments_with_constants() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module test_mixed(input wire clk, input wire[3:0] data_in, output reg[7:0] data_out, output wire valid);
            localparam INIT_VAL = 8'hAA;
            localparam THRESHOLD = 4'b1010;
            localparam NAME = \"Test Module\";
            
            always @(posedge clk) begin
                if (data_in >= THRESHOLD) begin
                    data_out <= {data_in, 4'b0000};
                end else begin
                    data_out <= INIT_VAL;
                end
            end
            
            assign valid = (data_out != 8'h00);
            endmodule
    ",
    )?;
    test_compilation("mixed_assignments_with_constants", modules);
    Ok(())
}

#[test]
fn test_complex_conditional_logic() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module complex_logic(input wire[2:0] sel, input wire[7:0] data, output reg[7:0] result);
            always @(*) begin
                case (sel)
                    3'b000: result = data ^ 8'hFF;
                    3'b001: result = data << 2;
                    3'b010: result = {data[3:0], data[7:4]};
                    3'b011: result = (data > 8'h80) ? data - 8'h80 : data + 8'h80;
                    default: begin
                        if (data[7]) begin
                            result = ~data;
                        end else begin
                            result = data | 8'h0F;
                        end
                    end
                endcase
            end
            endmodule
    ",
    )?;
    test_compilation("complex_conditional_logic", modules);
    Ok(())
}

#[test]
fn test_nested_module_instances() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module top_level(input wire clk, input wire[3:0] in_data, output wire[3:0] out_data);
            wire[3:0] intermediate1, intermediate2;
            
            first_stage stage1(.clk(clk), .data_in(in_data), .data_out(intermediate1));
            second_stage stage2(.clk(clk), .data_in(intermediate1), .data_out(intermediate2));
            third_stage stage3(.clk(clk), .data_in(intermediate2), .data_out(out_data));
            endmodule
            
            module first_stage(input wire clk, input wire[3:0] data_in, output reg[3:0] data_out);
            always @(posedge clk) begin
                data_out <= data_in + 4'b0001;
            end
            endmodule
    ",
    )?;
    test_compilation("nested_module_instances", modules);
    Ok(())
}

#[test]
fn test_arithmetic_and_bit_manipulation() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module bit_ops(input wire[7:0] a, input wire[7:0] b, output reg[15:0] result);
            wire[7:0] temp1, temp2, temp3;
            
            assign temp1 = a & b;
            assign temp2 = a | b;
            assign temp3 = a ^ b;
            
            always @(*) begin
                result[15:8] = {temp1[6:0], temp2[7]};
                result[7:0] = temp3 + (a * b[3:0]);
                
                if (|a) begin
                    result[0] = &b;
                end else begin
                    result[0] = ^b;
                end
            end
            endmodule
    ",
    )?;
    test_compilation("arithmetic_and_bit_manipulation", modules);
    Ok(())
}

#[test]
fn test_edge_triggered_state_machine() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module state_machine(input wire clk, input wire rst, input wire start, output reg[1:0] state, output reg done);
            localparam IDLE = 2'b00;
            localparam ACTIVE = 2'b01;
            localparam WAIT = 2'b10;
            localparam COMPLETE = 2'b11;
            
            reg[3:0] counter;
            
            always @(posedge clk, posedge rst) begin
                if (rst) begin
                    state <= IDLE;
                    counter <= 4'b0000;
                    done <= 1'b0;
                end else begin
                    case (state)
                        IDLE: begin
                            if (start) begin
                                state <= ACTIVE;
                                counter <= 4'b0000;
                            end
                            done <= 1'b0;
                        end
                        ACTIVE: begin
                            counter <= counter + 1;
                            if (counter == 4'b1111) begin
                                state <= WAIT;
                            end
                        end
                        WAIT: begin
                            state <= COMPLETE;
                        end
                        COMPLETE: begin
                            done <= 1'b1;
                            state <= IDLE;
                        end
                    endcase
                end
            end
            endmodule
    ",
    )?;
    test_compilation("edge_triggered_state_machine", modules);
    Ok(())
}

#[test]
fn test_dynamic_indexing_variations() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module indexing_test(input wire[7:0] data, input wire[2:0] offset, output reg[15:0] result);
            
            always @(*) begin
                result[7:0] = data[offset +: 4];
                result[15:8] = data[offset -: 3];
                result[3:0] = data[2 +: 4];
                result[11:8] = data[6 -: 4];
            end
            endmodule
    ",
    )?;
    test_compilation("dynamic_indexing_variations", modules);
    Ok(())
}

#[test]
fn test_mixed_literal_formats() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module literal_formats(input wire[7:0] data, output reg[31:0] result);
            localparam DECIMAL_VAL = 42;
            localparam HEX_VAL = 8'hA5;
            localparam BIN_VAL = 8'b1010_0101;
            localparam OCT_VAL = 8'o245;
            
            always @(*) begin
                result[7:0] = data + DECIMAL_VAL;
                result[15:8] = data ^ HEX_VAL;
                result[23:16] = data & BIN_VAL;
                result[31:24] = data | OCT_VAL;
            end
            endmodule
    ",
    )?;
    test_compilation("mixed_literal_formats", modules);
    Ok(())
}

#[test]
fn test_complex_sensitivity_lists() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module sensitivity_test(input wire clk1, input wire clk2, input wire rst, input wire[3:0] data, output reg[7:0] out1, output reg[7:0] out2);
            
            always @(posedge clk1, negedge rst) begin
                if (!rst) begin
                    out1 <= 8'h00;
                end else begin
                    out1 <= data + 1;
                end
            end
            
            always @(negedge clk2, posedge clk1, data) begin
                out2 <= {data, data};
            end
            
            always @(*) begin
                if (out1 > out2) begin
                    out2 = out1 - out2;
                end
            end
            endmodule
    ",
    )?;
    test_compilation("complex_sensitivity_lists", modules);
    Ok(())
}

#[test]
fn test_nested_concatenations_and_replications() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module concat_test(input wire[3:0] a, input wire[3:0] b, input wire[1:0] sel, output reg[15:0] result);
            wire[7:0] temp1, temp2;
            
            assign temp1 = {2{a}};
            assign temp2 = {b, a};
            
            always @(*) begin
                case (sel)
                    2'b00: result = {2{temp1}};
                    2'b01: result = {temp2, temp1};
                    2'b10: result = {{3{a[0]}}, {2{b[1:0]}}, temp1[6:0]};
                    2'b11: result = {4{a}} ^ {4{b}};
                endcase
            end
            endmodule
    ",
    )?;
    test_compilation("nested_concatenations_and_replications", modules);
    Ok(())
}

#[test]
fn test_signed_unsigned_operations() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module signed_ops(input wire signed[7:0] signed_data, input wire[7:0] unsigned_data, output reg signed[15:0] signed_result, output reg[15:0] unsigned_result);
            wire signed[7:0] temp_signed;
            wire[7:0] temp_unsigned;
            
            assign temp_signed = signed_data >>> 2;
            assign temp_unsigned = unsigned_data >> 2;
            
            always @(*) begin
                signed_result = signed_data * temp_signed;
                unsigned_result = unsigned_data * temp_unsigned;
                
                if (signed_data < 0) begin
                    signed_result = -signed_result;
                end
                
                if (signed_result[15]) begin
                    unsigned_result = ~unsigned_result + 1;
                end
            end
            endmodule
    ",
    )?;
    test_compilation("signed_unsigned_operations", modules);
    Ok(())
}

#[test]
fn test_memory_declaration_and_access() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module memory_test(input wire clk, input wire[2:0] addr, input wire[7:0] data_in, input wire we, output reg[7:0] data_out);
            reg[7:0] memory[0:7];
            
            always @(posedge clk) begin
                if (we) begin
                    memory[addr] <= data_in;
                end
                data_out <= memory[addr];
            end
            endmodule
    ",
    )?;
    test_compilation("memory_declaration_and_access", modules);
    Ok(())
}

#[test]
fn test_multi_dimensional_memory_operations() -> miette::Result<()> {
    let modules = test_parse::<ModuleList>(
        "
            module dual_port_memory(input wire clk, input wire[3:0] addr_a, input wire[3:0] addr_b, input wire[15:0] data_in_a, input wire[15:0] data_in_b, input wire we_a, input wire we_b, output reg[15:0] data_out_a, output reg[15:0] data_out_b);
            reg[15:0] mem_bank_0[0:15];
            reg[15:0] mem_bank_1[0:15];
            wire bank_sel_a, bank_sel_b;
            
            assign bank_sel_a = addr_a[3];
            assign bank_sel_b = addr_b[3];
            
            always @(posedge clk) begin
                if (we_a) begin
                    if (bank_sel_a) begin
                        mem_bank_1[addr_a[2:0]] <= data_in_a;
                    end else begin
                        mem_bank_0[addr_a[2:0]] <= data_in_a;
                    end
                end
                
                if (we_b) begin
                    if (bank_sel_b) begin
                        mem_bank_1[addr_b[2:0]] <= data_in_b;
                    end else begin
                        mem_bank_0[addr_b[2:0]] <= data_in_b;
                    end
                end
                
                data_out_a <= bank_sel_a ? mem_bank_1[addr_a[2:0]] : mem_bank_0[addr_a[2:0]];
                data_out_b <= bank_sel_b ? mem_bank_1[addr_b[2:0]] : mem_bank_0[addr_b[2:0]];
            end
            endmodule
    ",
    )?;
    test_compilation("multi_dimensional_memory_operations", modules);
    Ok(())
}
