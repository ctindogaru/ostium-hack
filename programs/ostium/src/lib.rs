pub mod account;
pub mod context;
use anchor_lang::prelude::*;
use anchor_spl::token;
use context::*;

declare_id!("DVCuZ7CgEi3WJrr1RMUhEP2eYW8PFKZXxw67RK9B9W6y");

#[program]
pub mod ostium {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        msg!("Ostium: DEPOSIT");
        let state = &mut ctx.accounts.state;

        let seeds = &[
            &state.to_account_info().key.to_bytes(),
            &[state.bump_seed][..],
        ];

        let signer = &[&seeds[..]];
        token::transfer(
            ctx.accounts.into_transfer_context().with_signer(signer),
            amount,
        )?;

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
