use crate::account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct Deposit<'info> {
    pub exchange: Account<'info, Exchange>,
    #[account(mut)]
    pub transfer_from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub transfer_to: Account<'info, TokenAccount>,
    /// CHECK: TBD
    #[account(signer)]
    pub transfer_authority: AccountInfo<'info>,
    /// CHECK: TBD
    #[account(address = token::ID)]
    pub token_program: AccountInfo<'info>,
}

impl<'info> Deposit<'info> {
    pub fn into_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.transfer_from.to_account_info(),
            to: self.transfer_to.to_account_info(),
            authority: self.transfer_authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct Withdraw {}

#[derive(Accounts)]
pub struct OpenPosition {}

#[derive(Accounts)]
pub struct ClosePosition {}

#[derive(Accounts)]
pub struct LiquidatePosition {}
