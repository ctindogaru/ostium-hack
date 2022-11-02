pub mod account;
pub mod context;
pub mod error;
pub mod utils;
use account::*;
use anchor_lang::prelude::*;
use anchor_spl::token;
use context::*;

const OSTIUM_SEED: &str = "ostium";

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
        position_manager.owner = *ctx.accounts.signer.key;
        position_manager.no_of_positions = 0;

        Ok(())
    }

    pub fn deposit_collateral(ctx: Context<DepositCollateral>, amount: u64) -> Result<()> {
        msg!("Ostium: DEPOSIT COLLATERAL");

        let position = &mut ctx.accounts.position;
        require!(
            position.owner == *ctx.accounts.signer.key,
            error::ErrorCode::PermissionDenied
        );

        position.collateral += amount;

        token::transfer(ctx.accounts.into_transfer_context(), amount)?;

        Ok(())
    }

    pub fn withdraw_collateral(ctx: Context<WithdrawCollateral>, amount: u64) -> Result<()> {
        msg!("Ostium: WITHDRAW COLLATERAL");

        let position = &mut ctx.accounts.position;
        require!(
            position.owner == *ctx.accounts.signer.key,
            error::ErrorCode::PermissionDenied
        );
        require!(
            position.collateral >= amount,
            error::ErrorCode::InsufficientFunds
        );

        position.collateral -= amount;

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
        require!(
            position_manager.owner == *ctx.accounts.signer.key,
            error::ErrorCode::PermissionDenied
        );

        // let price_account_info = &ctx.accounts.price_account_info;
        position.is_initialized = true;
        position.owner = *ctx.accounts.signer.key;
        position.asset = *ctx.accounts.price_account_info.key;
        // position.entry_price = get_current_price(price_account_info);
        position.entry_price = 1650 * 10u64.pow(6);
        position.entry_timestamp = Clock::get()?.unix_timestamp as u64;
        position.exit_price = 0;
        position.exit_timestamp = 0;
        position.quantity = quantity;
        position.leverage = leverage;
        position.status = PositionStatus::Open;
        position_manager.no_of_positions += 1;

        let initial_collateral = position.entry_price * position.quantity;
        position.collateral = initial_collateral;
        token::transfer(ctx.accounts.into_transfer_context(), initial_collateral)?;

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
            position.owner == *ctx.accounts.signer.key,
            error::ErrorCode::PermissionDenied
        );
        require!(
            position.status == PositionStatus::Open,
            error::ErrorCode::PositionNotOpened
        );
        require!(
            position.asset == *ctx.accounts.price_account_info.key,
            error::ErrorCode::WrongAsset
        );

        position.status = PositionStatus::Closed;

        // let price_account_info = &ctx.accounts.price_account_info;
        // let current_price = get_current_price(price_account_info);
        let current_price = 1800 * 10u64.pow(6);
        position.exit_price = current_price;
        position.exit_timestamp = Clock::get()?.unix_timestamp as u64;

        let pnl = (current_price as i64 - position.entry_price as i64)
            * position.quantity as i64
            * position.leverage as i64;
        let transfer_amount = position.collateral as i64 + pnl;
        if transfer_amount > 0 {
            let state = &mut ctx.accounts.state;
            let seeds = &[OSTIUM_SEED.as_bytes(), &[state.bump_seed]];
            let signer = &[&seeds[..]];
            token::transfer(
                ctx.accounts.into_transfer_context().with_signer(signer),
                transfer_amount as u64,
            )?;
        }

        Ok(())
    }

    pub fn liquidate_position(_ctx: Context<LiquidatePosition>) -> Result<()> {
        msg!("Ostium: LIQUIDATE POSITION ");
        Ok(())
    }
}
