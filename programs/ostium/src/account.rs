use anchor_lang::prelude::*;

#[account]
pub struct State {
    pub bump_seed: u8,
}
