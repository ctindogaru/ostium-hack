use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct Deposit {}

#[derive(Accounts)]
pub struct Withdraw {}

#[derive(Accounts)]
pub struct OpenPosition {}

#[derive(Accounts)]
pub struct ClosePosition {}

#[derive(Accounts)]
pub struct LiquidatePosition {}
