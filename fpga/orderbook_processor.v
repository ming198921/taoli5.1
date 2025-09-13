// Ultra-Low Latency Order Book Processor for 5.1 Arbitrage System
// Target: 1-2 clock cycles @ 300MHz = 3.3-6.6ns processing latency

`timescale 1ns / 1ps

module orderbook_processor #(
    parameter PRICE_WIDTH = 64,      // 64-bit fixed point price
    parameter QTY_WIDTH = 64,        // 64-bit quantity
    parameter DEPTH = 20,            // 20-level order book depth
    parameter SYMBOL_WIDTH = 32      // Symbol identifier
)(
    input wire clk,
    input wire rst_n,
    
    // Market data input (Direct from 10G Ethernet MAC)
    input wire [511:0] data_in,      // 64-byte wide data bus
    input wire data_valid,
    input wire data_sop,             // Start of packet
    input wire data_eop,             // End of packet
    
    // Order book output (parallel output for all levels)
    output reg [PRICE_WIDTH-1:0] bid_prices [0:DEPTH-1],
    output reg [QTY_WIDTH-1:0] bid_quantities [0:DEPTH-1],
    output reg [PRICE_WIDTH-1:0] ask_prices [0:DEPTH-1],
    output reg [QTY_WIDTH-1:0] ask_quantities [0:DEPTH-1],
    
    // Best bid/ask for quick access
    output reg [PRICE_WIDTH-1:0] best_bid,
    output reg [PRICE_WIDTH-1:0] best_ask,
    output reg [QTY_WIDTH-1:0] best_bid_qty,
    output reg [QTY_WIDTH-1:0] best_ask_qty,
    
    // Arbitrage signals
    output reg opportunity_detected,
    output reg [31:0] profit_bps,    // Profit in basis points
    
    // Performance counters
    output reg [63:0] messages_processed,
    output reg [63:0] opportunities_found
);

    // Internal state machine
    localparam IDLE = 2'b00;
    localparam PARSE_HEADER = 2'b01;
    localparam UPDATE_BOOK = 2'b10;
    localparam DETECT_ARB = 2'b11;
    
    reg [1:0] state;
    reg [31:0] msg_type;
    reg [SYMBOL_WIDTH-1:0] symbol;
    reg [31:0] exchange_id;
    
    // Parallel comparators for order book updates
    wire [DEPTH-1:0] bid_update_mask;
    wire [DEPTH-1:0] ask_update_mask;
    
    // Pipeline registers for timing closure
    reg [PRICE_WIDTH-1:0] new_price;
    reg [QTY_WIDTH-1:0] new_qty;
    reg is_bid;
    
    // Cross-exchange arbitrage detection registers
    reg [PRICE_WIDTH-1:0] binance_bid, binance_ask;
    reg [PRICE_WIDTH-1:0] coinbase_bid, coinbase_ask;
    reg [PRICE_WIDTH-1:0] okx_bid, okx_ask;
    
    // Message parser - extracts fields in parallel
    always @(posedge clk) begin
        if (!rst_n) begin
            state <= IDLE;
            messages_processed <= 0;
        end else begin
            case (state)
                IDLE: begin
                    if (data_valid && data_sop) begin
                        state <= PARSE_HEADER;
                        // Extract message type and symbol in parallel
                        msg_type <= data_in[31:0];
                        symbol <= data_in[63:32];
                        exchange_id <= data_in[95:64];
                    end
                end
                
                PARSE_HEADER: begin
                    // Single cycle parsing of order book update
                    new_price <= data_in[127:64];
                    new_qty <= data_in[191:128];
                    is_bid <= data_in[192];
                    state <= UPDATE_BOOK;
                end
                
                UPDATE_BOOK: begin
                    // Parallel update of all order book levels
                    if (is_bid) begin
                        // Find insertion point using parallel comparators
                        for (integer i = 0; i < DEPTH; i = i + 1) begin
                            if (new_price > bid_prices[i]) begin
                                // Shift and insert in single cycle
                                bid_prices[i] <= new_price;
                                bid_quantities[i] <= new_qty;
                                // Shift remaining levels
                                for (integer j = i+1; j < DEPTH; j = j + 1) begin
                                    bid_prices[j] <= bid_prices[j-1];
                                    bid_quantities[j] <= bid_quantities[j-1];
                                end
                                break;
                            end
                        end
                        best_bid <= bid_prices[0];
                        best_bid_qty <= bid_quantities[0];
                    end else begin
                        // Similar logic for asks
                        for (integer i = 0; i < DEPTH; i = i + 1) begin
                            if (new_price < ask_prices[i]) begin
                                ask_prices[i] <= new_price;
                                ask_quantities[i] <= new_qty;
                                for (integer j = i+1; j < DEPTH; j = j + 1) begin
                                    ask_prices[j] <= ask_prices[j-1];
                                    ask_quantities[j] <= ask_quantities[j-1];
                                end
                                break;
                            end
                        end
                        best_ask <= ask_prices[0];
                        best_ask_qty <= ask_quantities[0];
                    end
                    
                    messages_processed <= messages_processed + 1;
                    state <= DETECT_ARB;
                end
                
                DETECT_ARB: begin
                    // Store best prices by exchange for cross-exchange arbitrage
                    case (exchange_id)
                        32'h00000001: begin  // Binance
                            binance_bid <= best_bid;
                            binance_ask <= best_ask;
                        end
                        32'h00000002: begin  // Coinbase
                            coinbase_bid <= best_bid;
                            coinbase_ask <= best_ask;
                        end
                        32'h00000003: begin  // OKX
                            okx_bid <= best_bid;
                            okx_ask <= best_ask;
                        end
                    endcase
                    
                    // Detect arbitrage opportunities in parallel
                    // Check all exchange pairs simultaneously
                    if ((binance_bid > coinbase_ask) && 
                        ((binance_bid - coinbase_ask) * 10000 / coinbase_ask > 10)) begin
                        // Arbitrage: Buy on Coinbase, Sell on Binance
                        opportunity_detected <= 1'b1;
                        profit_bps <= (binance_bid - coinbase_ask) * 10000 / coinbase_ask;
                        opportunities_found <= opportunities_found + 1;
                    end else if ((coinbase_bid > binance_ask) && 
                                ((coinbase_bid - binance_ask) * 10000 / binance_ask > 10)) begin
                        // Arbitrage: Buy on Binance, Sell on Coinbase
                        opportunity_detected <= 1'b1;
                        profit_bps <= (coinbase_bid - binance_ask) * 10000 / binance_ask;
                        opportunities_found <= opportunities_found + 1;
                    end else if ((okx_bid > binance_ask) && 
                                ((okx_bid - binance_ask) * 10000 / binance_ask > 10)) begin
                        // Arbitrage: Buy on Binance, Sell on OKX
                        opportunity_detected <= 1'b1;
                        profit_bps <= (okx_bid - binance_ask) * 10000 / binance_ask;
                        opportunities_found <= opportunities_found + 1;
                    end else begin
                        opportunity_detected <= 1'b0;
                        profit_bps <= 0;
                    end
                    
                    state <= IDLE;
                end
            endcase
        end
    end
    
endmodule

// Triangular Arbitrage Detector Module
module triangular_arbitrage_detector #(
    parameter PRICE_WIDTH = 64
)(
    input wire clk,
    input wire rst_n,
    
    // Input prices for triangle (e.g., BTC/USDT, ETH/USDT, ETH/BTC)
    input wire [PRICE_WIDTH-1:0] pair1_bid,  // BTC/USDT bid
    input wire [PRICE_WIDTH-1:0] pair1_ask,  // BTC/USDT ask
    input wire [PRICE_WIDTH-1:0] pair2_bid,  // ETH/USDT bid
    input wire [PRICE_WIDTH-1:0] pair2_ask,  // ETH/USDT ask
    input wire [PRICE_WIDTH-1:0] pair3_bid,  // ETH/BTC bid
    input wire [PRICE_WIDTH-1:0] pair3_ask,  // ETH/BTC ask
    
    // Arbitrage detection output
    output reg triangle_opportunity,
    output reg [31:0] triangle_profit_bps,
    output reg [2:0] best_path  // Which path through triangle is profitable
);

    // DSP-optimized multipliers for price calculations
    wire [2*PRICE_WIDTH-1:0] path1_result;
    wire [2*PRICE_WIDTH-1:0] path2_result;
    
    // Path 1: USDT -> BTC -> ETH -> USDT
    // Buy BTC with USDT, Buy ETH with BTC, Sell ETH for USDT
    assign path1_result = (1000000 * pair3_bid * pair2_bid) / (pair1_ask * 1000000);
    
    // Path 2: USDT -> ETH -> BTC -> USDT  
    // Buy ETH with USDT, Sell ETH for BTC, Sell BTC for USDT
    assign path2_result = (1000000 * pair1_bid * 1000000) / (pair2_ask * pair3_ask);
    
    always @(posedge clk) begin
        if (!rst_n) begin
            triangle_opportunity <= 1'b0;
            triangle_profit_bps <= 0;
            best_path <= 3'b000;
        end else begin
            // Check both paths in parallel
            if (path1_result > 1001000) begin  // >0.1% profit after fees
                triangle_opportunity <= 1'b1;
                triangle_profit_bps <= (path1_result - 1000000) / 100;
                best_path <= 3'b001;
            end else if (path2_result > 1001000) begin
                triangle_opportunity <= 1'b1;
                triangle_profit_bps <= (path2_result - 1000000) / 100;
                best_path <= 3'b010;
            end else begin
                triangle_opportunity <= 1'b0;
                triangle_profit_bps <= 0;
                best_path <= 3'b000;
            end
        end
    end
    
endmodule