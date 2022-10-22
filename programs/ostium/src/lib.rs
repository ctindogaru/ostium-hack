pub mod context;
use anchor_lang::prelude::*;
use context::*;

declare_id!("DVCuZ7CgEi3WJrr1RMUhEP2eYW8PFKZXxw67RK9B9W6y");

#[program]
pub mod ostium {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn deposit(_ctx: Context<Deposit>) -> Result<()> {
        Ok(())
    }

    pub fn withdraw(_ctx: Context<Withdraw>) -> Result<()> {
        Ok(())
    }

    pub fn open_position(_ctx: Context<OpenPosition>) -> Result<()> {
        Ok(())
    }

    pub fn close_position(_ctx: Context<ClosePosition>) -> Result<()> {
        Ok(())
    }

    pub fn liquidate_position(_ctx: Context<LiquidatePosition>) -> Result<()> {
        Ok(())
    }
}
