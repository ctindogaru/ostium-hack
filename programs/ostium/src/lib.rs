use anchor_lang::prelude::*;
use anchor_spl::token;

declare_id!("41wefNSrftGHqJTuoh6n4J2G31Z8NuhR9yc5zVcKD4n9");

#[program]
pub mod ostium {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn open_new_position(
        ctx: Context<OpenNewPosition>,
        amount_to_lock: u64,
        user_pubkey: Pubkey,
        _asset: u64,
        leverage: u8,
        _oracle_price: u64,
    ) -> Result<()> {
        // Need current price from oracle
        // Leverage

        // Transfer amount_to_lock from user to program

        let cpi_accounts = token::Transfer {
            from: ctx.accounts.source_info.clone(),
            to: ctx.accounts.destination_info.clone(),
            authority: ctx.accounts.authority.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx_transfer = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(cpi_ctx_transfer, amount_to_lock)?;

        // Call pyth oracle to get most recent price

        // How do u represent an open position

        let open_order = &mut ctx.accounts.open_order;

        open_order.amount_collateral = amount_to_lock;
        open_order.user_pubkey = user_pubkey;
        open_order.leverage = leverage;
        open_order.is_liquidated = false;

        // let account = &mut ctx.accounts.account;
        // account.name = name;
        // account.owner = *ctx.accounts.owner.key;
        // account.balance = 0;
        Ok(())
    }

    // Insert new function here (for example, liquidate or close)

    pub fn liquidate_position(ctx: Context<LiquidatePosition>) -> Result<()> {
        // Liquidate position
        // Transfer collateral to liquidator
        // Transfer asset to liquidator
        // Burn asset
        // Burn collateral

        // Check to make sure liquidate_position is a valid call

        // Check the openOrder account and make sure conditions are met

        let open_order = &mut ctx.accounts.open_order;

        if open_order.amount_collateral < 10 {
            // Close the open_order account
            open_order.is_liquidated = true;
        }

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let open_order = &mut ctx.accounts.open_order;

        if open_order.is_liquidated == false {
            // Withdraw
        } else {
            // fuck off
        }

        Ok(())
    }
}

// This section tells you what accounts u need pass in on the client side and what accounts the program has access to on the backend
#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct OpenNewPosition<'info> {
    pub open_order: Account<'info, OpenOrder>,
    // Whatever else u need to access with ctx.accounts
    /// CHECK: TBD
    pub authority: AccountInfo<'info>,
    // pub amm: Account<'info, AMM>,
    // #[account(signer)]
    // pub signer: AccountInfo<'info>,
    /// CHECK: TBD
    #[account(mut)]
    pub source_info: AccountInfo<'info>,
    /// CHECK: TBD
    #[account(mut)]
    pub destination_info: AccountInfo<'info>,
    // #[account(mut)]
    // pub swap_source: Account<'info, TokenAccount>,
    // #[account(mut)]
    // pub swap_destination: Account<'info, TokenAccount>,
    // #[account(mut)]
    // pub pool_mint: Account<'info, Mint>,
    // #[account(mut)]
    // pub fee_account: Account<'info, TokenAccount>,
    /// CHECK: TBD
    pub token_program: AccountInfo<'info>,
    // pub host_fee_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct LiquidatePosition<'info> {
    pub open_order: Account<'info, OpenOrder>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    pub open_order: Account<'info, OpenOrder>,
}

// while true:
// Get price from oracle from pyth
// Ask is open order's account for this user have enough collateral for current oracle price

// Define structs that act as schema for account layout
#[account]
pub struct OpenOrder {
    // Amount of collateral
    pub amount_collateral: u64,
    // User
    pub user_pubkey: Pubkey,
    // Leverage
    pub leverage: u8,
    // Is liquidated?
    pub is_liquidated: bool,
    //

    // /// Is the swap initialized, with data written to it
    // pub is_initialized: bool,
    // /// Bump seed used to generate the program address / authority
    // pub bump_seed: u8,
    // /// Token program ID associated with the swap
    // pub token_program_id: Pubkey,
    // /// Address of token A liquidity account
    // pub token_a_account: Pubkey,
    // /// Address of token B liquidity account
    // pub token_b_account: Pubkey,
    // /// Address of pool token mint
    // pub pool_mint: Pubkey,
    // /// Address of token A mint
    // pub token_a_mint: Pubkey,
    // /// Address of token B mint
    // pub token_b_mint: Pubkey,
    // /// Address of pool fee account
    // pub pool_fee_account: Pubkey,
    // /// Fees associated with swap
    // pub fees: FeesInput,
    // /// Curve associated with swap
    // pub curve: CurveInput,
}
