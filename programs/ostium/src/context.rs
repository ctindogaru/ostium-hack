use crate::account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};
use std::mem::size_of;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = signer, space = 8 + size_of::<State>())]
    pub state: Account<'info, State>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: safe
    pub owner: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct InitializePositionManager<'info> {
    #[account(
        init,
        seeds = [
            b"position-manager".as_ref(),
            signer.key().as_ref()
        ],
        bump,
        payer = signer,
        space = 8 + size_of::<PositionManager>()
    )]
    pub position_manager: Account<'info, PositionManager>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub position_manager: Account<'info, PositionManager>,
    #[account(mut)]
    pub transfer_from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub transfer_to: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Deposit<'info> {
    pub fn into_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.transfer_from.to_account_info(),
            to: self.transfer_to.to_account_info(),
            authority: self.authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub position_manager: Account<'info, PositionManager>,
    pub state: Account<'info, State>,
    #[account(mut)]
    pub transfer_from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub transfer_to: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Withdraw<'info> {
    pub fn into_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.transfer_from.to_account_info(),
            to: self.transfer_to.to_account_info(),
            authority: self.authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct OpenPosition<'info> {
    #[account(mut)]
    pub position_manager: Account<'info, PositionManager>,
    #[account(
        init,
        seeds = [
            b"position".as_ref(),
            signer.key().as_ref(),
            &position_manager.no_of_positions.to_le_bytes(),
        ],
        bump,
        payer = signer,
        space = 8 + size_of::<Position>()
    )]
    pub position: Account<'info, Position>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    #[account(mut)]
    pub position_manager: Account<'info, PositionManager>,
    #[account(mut)]
    pub position: Account<'info, Position>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub price_account_info: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct LiquidatePosition {}
