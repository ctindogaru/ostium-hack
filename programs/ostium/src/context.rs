use crate::account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};
use std::mem::size_of;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [
            b"ostium".as_ref()
        ],
        bump,
        payer = signer,
        space = 8 + size_of::<State>()
    )]
    pub state: Account<'info, State>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
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
pub struct CollectFees<'info> {
    #[account(
        seeds = [b"ostium".as_ref()],
        bump = state.ostium_bump,
        constraint = state.to_account_info().owner == program_id,
    )]
    pub state: Account<'info, State>,
    #[account(mut,
        constraint = &transfer_from.owner == state.to_account_info().key
    )]
    pub transfer_from: Account<'info, TokenAccount>,
    #[account(mut,
        constraint = transfer_to.owner == state.admin,
        constraint = &transfer_to.owner == signer.key,
    )]
    pub transfer_to: Account<'info, TokenAccount>,
    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> CollectFees<'info> {
    pub fn into_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.transfer_from.to_account_info(),
            to: self.transfer_to.to_account_info(),
            authority: self.state.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct DepositCollateral<'info> {
    #[account(mut)]
    pub position: Account<'info, Position>,
    #[account(mut)]
    pub transfer_from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub transfer_to: Account<'info, TokenAccount>,
    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> DepositCollateral<'info> {
    pub fn into_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.transfer_from.to_account_info(),
            to: self.transfer_to.to_account_info(),
            authority: self.signer.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct WithdrawCollateral<'info> {
    #[account(mut)]
    pub position: Account<'info, Position>,
    #[account(
        seeds = [b"ostium".as_ref()],
        bump = state.ostium_bump,
        constraint = state.to_account_info().owner == program_id,
    )]
    pub state: Account<'info, State>,
    #[account(mut)]
    pub transfer_from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub transfer_to: Account<'info, TokenAccount>,
    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> WithdrawCollateral<'info> {
    pub fn into_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.transfer_from.to_account_info(),
            to: self.transfer_to.to_account_info(),
            authority: self.state.to_account_info(),
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
            &position_manager.no_of_positions.to_le_bytes()
        ],
        bump,
        payer = signer,
        space = 8 + size_of::<Position>()
    )]
    pub position: Account<'info, Position>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub price_account_info: AccountInfo<'info>,
    #[account(mut)]
    pub fee_collector: Account<'info, TokenAccount>,
    #[account(mut)]
    pub transfer_from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub transfer_to: Account<'info, TokenAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    #[account(mut)]
    pub position: Account<'info, Position>,
    #[account(
        seeds = [b"ostium".as_ref()],
        bump = state.ostium_bump,
        constraint = state.to_account_info().owner == program_id,
    )]
    pub state: Account<'info, State>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub price_account_info: AccountInfo<'info>,
    #[account(mut)]
    pub transfer_from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub transfer_to: Account<'info, TokenAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> ClosePosition<'info> {
    pub fn into_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.transfer_from.to_account_info(),
            to: self.transfer_to.to_account_info(),
            authority: self.state.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct LiquidatePosition<'info> {
    #[account(mut)]
    pub position: Account<'info, Position>,
    #[account(
        seeds = [b"ostium".as_ref()],
        bump = state.ostium_bump,
        constraint = state.to_account_info().owner == program_id,
    )]
    pub state: Account<'info, State>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub price_account_info: AccountInfo<'info>,
    #[account(mut)]
    pub transfer_from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub transfer_to: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> LiquidatePosition<'info> {
    pub fn into_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.transfer_from.to_account_info(),
            to: self.transfer_to.to_account_info(),
            authority: self.state.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}
