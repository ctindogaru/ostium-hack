use anchor_lang::prelude::*;

#[account]
pub struct State {
    pub is_initialized: bool,
    pub admin: Pubkey,
    pub bump_seed: u8,
}

#[account]
pub struct Position {
    pub is_initialized: bool,
    pub entry_price: u64,
    pub quantity: u64,
    pub leverage: u8,
    pub status: PositionStatus,
}

#[account]
pub struct PositionManager {
    pub is_initialized: bool,
    pub no_of_positions: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum PositionStatus {
    Open = 0,
    Closed = 1,
    Liquidated = 2,
}
