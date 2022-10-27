pub mod account;
pub mod context;
use anchor_lang::prelude::*;
use anchor_spl::token;
use context::*;

const OSTIUM_SEED: &str = "Ostium";

declare_id!("DVCuZ7CgEi3WJrr1RMUhEP2eYW8PFKZXxw67RK9B9W6y");

#[program]
pub mod ostium {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, bump: u8) -> Result<()> {
        msg!("Ostium: INITIALIZE");
        let state = &mut ctx.accounts.state;

        state.is_initialized = true;
        state.bump_seed = bump;
        state.admin = *ctx.accounts.admin.key;

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        msg!("Ostium: DEPOSIT");
        let state = &mut ctx.accounts.state;

        let seeds = &[OSTIUM_SEED.as_bytes(), &[state.bump_seed]];
        let signer = &[&seeds[..]];
        token::transfer(
            ctx.accounts.into_transfer_context().with_signer(signer),
            amount,
        )?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        msg!("Ostium: WITHDRAW");
        let state = &mut ctx.accounts.state;

        let seeds = &[OSTIUM_SEED.as_bytes(), &[state.bump_seed]];
        let signer = &[&seeds[..]];
        token::transfer(
            ctx.accounts.into_transfer_context().with_signer(signer),
            amount,
        )?;

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
