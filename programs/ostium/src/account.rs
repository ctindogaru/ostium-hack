use anchor_lang::prelude::*;

#[account]
pub struct State {
    pub admin: Pubkey,
    pub bump_seed: u8,
}
