use anchor_lang::prelude::*;

#[account]
pub struct State {
    pub is_initialized: bool,
    pub bump_seed: u8,
}

#[account]
pub struct Position {
    pub is_initialized: bool,       // true if initialized, false otherwise
    pub owner: Pubkey,              // the user which opened this position
    pub asset: Pubkey,              // the asset that it's being traded (ex: silver/gold)
    pub collateral: u64,            // collateral in this position
    pub entry_price: u64,           // the asset price when this position was opened
    pub entry_timestamp: u64,       // the timestamp when this position was opened
    pub exit_price: u64,            // the asset price when this position was closed
    pub exit_timestamp: u64,        // the timestamp when this position was closed
    pub quantity: u64, // the asset quantity, with no leverage. The smaller unit is 0.0001. (ex: 1 ounce of gold will be 10000, 0.5 ounce of gold will be 5000)
    pub leverage: u16, // if the leverage is 10 and the quantity is 0.5, it means that we are actually trading with 5 ounces of gold (0.5 from the user, 4.5 borrowed).
    pub pos_type: PositionType, // type of this position
    pub pos_status: PositionStatus, // status of this position
}

#[account]
pub struct PositionManager {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub no_of_positions: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum PositionType {
    Long = 0,
    Short = 1,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum PositionStatus {
    Open = 0,
    Closed = 1,
    Liquidated = 2,
}
