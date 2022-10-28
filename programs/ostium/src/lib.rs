pub mod account;
pub mod context;
pub mod error;
pub mod utils;
use account::*;
use anchor_lang::prelude::*;
use anchor_spl::token;
use context::*;
use utils::*;

const OSTIUM_SEED: &str = "Ostium";

declare_id!("DVCuZ7CgEi3WJrr1RMUhEP2eYW8PFKZXxw67RK9B9W6y");

#[program]
pub mod ostium {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, bump: u8) -> Result<()> {
        msg!("Ostium: INITIALIZE");
        let state = &mut ctx.accounts.state;

        require!(!state.is_initialized, error::ErrorCode::AlreadyInitialized);

        state.is_initialized = true;
        state.bump_seed = bump;
        state.admin = *ctx.accounts.admin.key;

        Ok(())
    }

    pub fn initialize_position_manager(ctx: Context<InitializePositionManager>) -> Result<()> {
        msg!("Ostium: INITIALIZE POSITION MANAGER");
        let position_manager = &mut ctx.accounts.position_manager;

        require!(
            !position_manager.is_initialized,
            error::ErrorCode::AlreadyInitialized
        );

        position_manager.is_initialized = true;
        position_manager.no_of_positions = 0;

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        msg!("Ostium: DEPOSIT");

        token::transfer(ctx.accounts.into_transfer_context(), amount)?;

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

    pub fn open_position(ctx: Context<OpenPosition>, quantity: u64, leverage: u8) -> Result<()> {
        msg!("Ostium: OPEN POSITION");
        let position = &mut ctx.accounts.position;
        let position_manager = &mut ctx.accounts.position_manager;

        require!(
            !position.is_initialized,
            error::ErrorCode::AlreadyInitialized
        );
        require!(
            position_manager.is_initialized,
            error::ErrorCode::NotInitialized
        );

        // let price_account_info = &ctx.accounts.price_account_info;
        position.is_initialized = true;
        // position.entry_price = get_current_price(price_account_info);
        position.entry_price = 1650;
        position.quantity = quantity;
        position.leverage = leverage;
        position.status = PositionStatus::Open;

        position_manager.no_of_positions += 1;

        Ok(())
    }

    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        msg!("Ostium: CLOSE POSITION");
        let position = &mut ctx.accounts.position;

        require!(
            position.is_initialized,
            error::ErrorCode::AlreadyInitialized
        );
        require!(
            position.status == PositionStatus::Open,
            error::ErrorCode::PositionNotOpened
        );

        position.status = PositionStatus::Closed;

        // let price_account_info = &ctx.accounts.price_account_info;
        // let current_price = get_current_price(price_account_info);
        let current_price = 1800;
        // we assume a long and profitable position for now
        let _pnl =
            (current_price - position.entry_price) * position.quantity * position.leverage as u64;

        // TODO: send pnl to the user

        Ok(())
    }

    pub fn liquidate_position(_ctx: Context<LiquidatePosition>) -> Result<()> {
        msg!("Ostium: LIQUIDATE POSITION ");
        Ok(())
    }
}
