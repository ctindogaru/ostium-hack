pub mod account;
pub mod context;
pub mod error;
pub mod utils;
use account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};
use context::*;
use utils::*;

const OSTIUM_SEED: &str = "ostium";
const FEE_COLLECTOR_SEED: &str = "fee-collector";

declare_id!("DVCuZ7CgEi3WJrr1RMUhEP2eYW8PFKZXxw67RK9B9W6y");

#[program]
pub mod ostium {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        ostium_seed: u8,
        fee_collector_seed: u8,
    ) -> Result<()> {
        msg!("Ostium: INITIALIZE");
        let state = &mut ctx.accounts.state;

        require!(!state.is_initialized, error::ErrorCode::AlreadyInitialized);

        state.is_initialized = true;
        state.admin = *ctx.accounts.signer.key;
        state.fee_collector_seed = fee_collector_seed;
        state.ostium_seed = ostium_seed;

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

    pub fn collect_fees(ctx: Context<CollectFees>, amount: u64) -> Result<()> {
        msg!("Ostium: COLLECT FEES");

        let state = &mut ctx.accounts.state;
        require!(state.is_initialized, error::ErrorCode::NotInitialized);
        require!(
            state.admin == *ctx.accounts.signer.key,
            error::ErrorCode::PermissionDenied
        );

        let seeds = &[FEE_COLLECTOR_SEED.as_bytes(), &[state.fee_collector_seed]];
        let signer = &[&seeds[..]];
        token::transfer(
            ctx.accounts.into_transfer_context().with_signer(signer),
            amount,
        )?;

        Ok(())
    }

    pub fn deposit_collateral(ctx: Context<DepositCollateral>, amount: u64) -> Result<()> {
        msg!("Ostium: DEPOSIT COLLATERAL");

        let position = &mut ctx.accounts.position;
        require!(position.is_initialized, error::ErrorCode::NotInitialized);
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
        require!(position.is_initialized, error::ErrorCode::NotInitialized);
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
        let seeds = &[OSTIUM_SEED.as_bytes(), &[state.ostium_seed]];
        let signer = &[&seeds[..]];
        token::transfer(
            ctx.accounts.into_transfer_context().with_signer(signer),
            amount,
        )?;

        Ok(())
    }

    pub fn open_position(
        ctx: Context<OpenPosition>,
        quantity: u64,
        leverage: u64,
        pos_type: PositionType,
    ) -> Result<()> {
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
        require!(quantity > MIN_QUANTITY, error::ErrorCode::MinimumQuantity);

        let fee_in_quantity = get_ostium_fee(quantity * leverage);

        // let price_account_info = &ctx.accounts.price_account_info;
        position.is_initialized = true;
        position.owner = *ctx.accounts.signer.key;
        position.asset = *ctx.accounts.price_account_info.key;
        // position.entry_price = get_current_price(price_account_info);
        position.entry_price = 1650 * 10u64.pow(6);
        position.entry_timestamp = Clock::get()?.unix_timestamp as u64;
        position.exit_price = 0;
        position.exit_timestamp = 0;
        position.quantity = quantity - fee_in_quantity;
        position.leverage = leverage;
        position.pos_status = PositionStatus::Open;
        position.pos_type = pos_type;
        position_manager.no_of_positions += 1;

        {
            let initial_collateral =
                position.entry_price * position.quantity / UNITS_IN_ONE_QUANTITY;
            position.collateral = initial_collateral;

            let cpi_accounts = Transfer {
                from: ctx.accounts.transfer_from.to_account_info(),
                to: ctx.accounts.transfer_to.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            };
            let cpi_context =
                CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
            token::transfer(cpi_context, initial_collateral)?;
        }

        {
            let fee_in_usdc = position.entry_price * fee_in_quantity / UNITS_IN_ONE_QUANTITY;

            let cpi_accounts = Transfer {
                from: ctx.accounts.transfer_from.to_account_info(),
                to: ctx.accounts.fee_collector.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            };
            let cpi_context =
                CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
            token::transfer(cpi_context, fee_in_usdc)?;
        }

        Ok(())
    }

    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        msg!("Ostium: CLOSE POSITION");
        let position = &mut ctx.accounts.position;

        require!(position.is_initialized, error::ErrorCode::NotInitialized);
        require!(
            position.owner == *ctx.accounts.signer.key,
            error::ErrorCode::PermissionDenied
        );
        require!(
            position.pos_status == PositionStatus::Open,
            error::ErrorCode::PositionNotOpened
        );
        require!(
            position.asset == *ctx.accounts.price_account_info.key,
            error::ErrorCode::WrongAsset
        );

        position.pos_status = PositionStatus::Closed;

        // let price_account_info = &ctx.accounts.price_account_info;
        // let current_price = get_current_price(price_account_info);
        let current_price = 1800 * 10u64.pow(6);
        position.exit_price = current_price;
        position.exit_timestamp = Clock::get()?.unix_timestamp as u64;

        let mut pnl = (current_price as i64 - position.entry_price as i64)
            * position.leverage as i64
            * position.quantity as i64
            / UNITS_IN_ONE_QUANTITY as i64;
        if position.pos_type == PositionType::Short {
            pnl *= -1;
        }

        let transfer_amount = position.collateral as i64 + pnl;
        if transfer_amount > 0 {
            let state = &mut ctx.accounts.state;
            let seeds = &[OSTIUM_SEED.as_bytes(), &[state.ostium_seed]];
            let signer = &[&seeds[..]];
            token::transfer(
                ctx.accounts.into_transfer_context().with_signer(signer),
                transfer_amount as u64,
            )?;
        }

        Ok(())
    }

    pub fn liquidate_position(ctx: Context<LiquidatePosition>) -> Result<()> {
        msg!("Ostium: LIQUIDATE POSITION ");

        let position = &mut ctx.accounts.position;

        require!(position.is_initialized, error::ErrorCode::NotInitialized);
        require!(
            position.pos_status == PositionStatus::Open,
            error::ErrorCode::PositionNotOpened
        );
        require!(
            position.asset == *ctx.accounts.price_account_info.key,
            error::ErrorCode::WrongAsset
        );

        // let price_account_info = &ctx.accounts.price_account_info;
        // let current_price = get_current_price(price_account_info);
        let current_price = 1800 * 10u64.pow(6);

        let mut pnl = (current_price as i64 - position.entry_price as i64)
            * position.leverage as i64
            * position.quantity as i64
            / UNITS_IN_ONE_QUANTITY as i64;
        if position.pos_type == PositionType::Short {
            pnl *= -1;
        }

        if should_be_liquidated(position.collateral as i64, pnl) {
            position.pos_status = PositionStatus::Liquidated;
            position.exit_price = current_price;
            position.exit_timestamp = Clock::get()?.unix_timestamp as u64;

            let transfer_amount = position.collateral as i64 + pnl;
            if transfer_amount > 0 {
                let state = &mut ctx.accounts.state;
                let seeds = &[OSTIUM_SEED.as_bytes(), &[state.ostium_seed]];
                let signer = &[&seeds[..]];
                token::transfer(
                    ctx.accounts.into_transfer_context().with_signer(signer),
                    transfer_amount as u64,
                )?;
            }
        }

        Ok(())
    }
}
