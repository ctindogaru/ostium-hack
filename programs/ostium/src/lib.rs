use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, MintTo, TokenAccount, Transfer};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod ostium {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        
        Ok(())
    }

    pub fn open_new_position(ctx: Context<OpenNewPosition>, amount_to_lock: NonZeroU64, asset: NonZeroU64, leverage: NonZeroU64, oracle_price: NonZeroU64 ) -> Result<()> {
        // Need current price from oracle
        // Leverage
        
        // Transfer amount_to_lock from user to program

        
        token::transfer(
            ctx.accounts.from_account,
            ctx.accounts.to_account, // PDA account owned by program (need to initialize on client side)
            amount_to_lock,
        )?;


        // Call pyth oracle to get most recent price


        // How do u represent an open position

        let open_order = &mut ctx.accounts.openorder;

        open_order.amount_collateral = amount_to_lock;
        open_order.user_pubkey = *ctx.accounts.user_pubkey;
        open_order.leverage = leverage;
        open_order.is_liquidated = false;

        // let account = &mut ctx.accounts.account;
        // account.name = name;
        // account.owner = *ctx.accounts.owner.key;
        // account.balance = 0;
        Ok(())
    }

    // Insert new function here (for example, liquidate or close)

    pub fn liquidate_position(ctx: Context<LiquidateAccount>) -> Result<()> {
        // Liquidate position
        // Transfer collateral to liquidator
        // Transfer asset to liquidator
        // Burn asset
        // Burn collateral
        
        // Check to make sure liquidate_position is a valid call
        
        // Check the openOrder account and make sure conditions are met

        let open_order = &mut ctx.accounts.openorder;

        if open_open.amount_collateral < 10 {
            // Close the open_order account
            open_order.is_liquidated = true;
        }

        
        Ok(())
    }

    pub fn withdraw() -> {

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
    pub openOrder: ProgramAccount<'info, OpenOrder>
    pub user_pubkey: AccountInfo<'info>,
    // Whatever else u need to access with ctx.accounts


    // pub authority: AccountInfo<'info>,
    // pub amm: ProgramAccount<'info, AMM>,
    // #[account(signer)]
    // pub user_transfer_authority: AccountInfo<'info>,
    // #[account(mut)]
    // pub source_info: AccountInfo<'info>,
    // #[account(mut)]
    // pub destination_info: AccountInfo<'info>,
    // #[account(mut)]
    // pub swap_source: Account<'info, TokenAccount>,
    // #[account(mut)]
    // pub swap_destination: Account<'info, TokenAccount>,
    // #[account(mut)]
    // pub pool_mint: Account<'info, Mint>,
    // #[account(mut)]
    // pub fee_account: Account<'info, TokenAccount>,
    // pub token_program: AccountInfo<'info>,
    // pub host_fee_account: AccountInfo<'info>,
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