use anchor_lang::prelude::*;

#[account]
pub struct Exchange {
    /// Bump seed used to generate the program address / authority
    pub bump_seed: u8,
}
