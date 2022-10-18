use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, MintTo, TokenAccount, Transfer};
use curve::base::CurveType;
use std::convert::TryFrom;

pub mod curve;
use crate::curve::{
    base::SwapCurve,
    calculator::{CurveCalculator, RoundDirection, TradeDirection},
    fees::Fees,
};
use crate::curve::{
    constant_price::ConstantPriceCurve, constant_product::ConstantProductCurve,
    offset::OffsetCurve, stable::StableCurve,
};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        fees_input: FeesInput,
        curve_input: CurveInput,
    ) -> ProgramResult {

        

        // find_program_address iteratively calls create_program_address until valid
        // basically create swap_authority acccount
        // bump seed lets you later reference this program
        let (swap_authority, bump_seed) = Pubkey::find_program_address(
            &[&ctx.accounts.amm.to_account_info().key.to_bytes()],
            ctx.program_id,
        );
        let seeds = &[
            &ctx.accounts.amm.to_account_info().key.to_bytes(),
            &[bump_seed][..],
        ];

        // build the curve
        let curve = build_curve(&curve_input).unwrap();
        curve
            .calculator
            .validate_supply(ctx.accounts.token_a.amount, ctx.accounts.token_b.amount)?;
    
        // build fees
        let fees = build_fees(&fees_input).unwrap();
        fees.validate()?;
        curve.calculator.validate()?;

        // so calculate neew pool supply 
        let initial_amount = curve.calculator.new_pool_supply();
        
        // mint to owner's pool token account 
        token::mint_to(
            ctx.accounts
                .into_mint_to_context()
                .with_signer(&[&seeds[..]]),
            u64::try_from(initial_amount).unwrap(),
        )?;
        // ^ this is basically below but with context 
        // token::instruction::mint_to(
        //     ctx.accounts.token_program.key, 
        //     ctx.accounts.pool_mint.to_account_info().key, 
        //     ctx.accounts.destination.to_account_info().key, 
        //     ctx.accounts.authority,
        //     &[&seeds[..]],
        //     u64::try_from(initial_amount).unwrap()
        // );

        let amm = &mut ctx.accounts.amm;
        // initialized  = true
        amm.is_initialized = true;
        // the bump seed is the thing u need to derive the swap_authority 
        amm.bump_seed = bump_seed; 
        // the regular token program id
        amm.token_program_id = *ctx.accounts.token_program.key;
        // pubkey of token a account
        amm.token_a_account = *ctx.accounts.token_a.to_account_info().key;
        // pubkey of token b account
        amm.token_b_account = *ctx.accounts.token_b.to_account_info().key;
        // just keeping track of all this info we passed into initialize basically
        amm.pool_mint = *ctx.accounts.pool_mint.to_account_info().key;
        amm.token_a_mint = ctx.accounts.token_a.mint;
        amm.token_b_mint = ctx.accounts.token_b.mint;
        amm.pool_fee_account = *ctx.accounts.fee_account.to_account_info().key;
        amm.fees = fees_input;
        amm.curve = curve_input;

        Ok(())
    }

    pub fn swap(ctx: Context<Swap>, amount_in: u64, _minimum_amount_out: u64) -> ProgramResult {
        let amm = &mut ctx.accounts.amm;

        // if the key is the token A account key, then A to B trade, else B to A
        let trade_direction =
            if *ctx.accounts.swap_source.to_account_info().key == amm.token_a_account {
                TradeDirection::AtoB
            } else {
                TradeDirection::BtoA
            };

        // curve and fees
        let curve = build_curve(&amm.curve).unwrap();
        let fees = build_fees(&amm.fees).unwrap();
        
        //.swap takes in (sourceamount, swapsourceamount, swapdestinationamount) results a swap result
        let result = curve
            .swap(
                u128::try_from(amount_in).unwrap(),
                u128::try_from(ctx.accounts.swap_source.amount).unwrap(),
                u128::try_from(ctx.accounts.swap_destination.amount).unwrap(),
                trade_direction,
                &fees,
            )
            .ok_or(SwapError::ZeroTradingTokens)?;
        
        let seeds = &[&amm.to_account_info().key.to_bytes(), &[amm.bump_seed][..]];

        //tranfer using the contex syntax again with seeds and stuff
        // pub fn transfer(
        //     token_program_id
        //     from (source_info - useraccountA),
        //     to (swap source - tokenaccountA),
        //     user swap authority, owns useraccA (wait question how does this have authoirty over token acc b)
        //     signer_pubkeys: &[&Pubkey], 
        //     amount: u64
        // ) -> Result<Instruction, ProgramError>

        // n.b If an account is not owned by a program, the program is only permitted to read its data and credit the account.

        token::transfer(
            ctx.accounts
                .into_transfer_to_swap_source_context()
                .with_signer(&[&seeds[..]]),
            u64::try_from(result.source_amount_swapped).unwrap(),
        )?;

        //finally transfer from token account B (owned by authority) to user acc B
        token::transfer(
            ctx.accounts
                .into_transfer_to_destination_context()
                .with_signer(&[&seeds[..]]),
            u64::try_from(result.destination_amount_swapped).unwrap(),
        )?;

        Ok(())
    }

    pub fn deposit_liquidity(
        ctx: Context<DepositLiquidity>,
        pool_token_amount: u64,
        _maximum_token_a_amount: u64,
        _maximum_token_b_amount: u64,
    ) -> ProgramResult {
        let amm = &mut ctx.accounts.amm;

        let curve = build_curve(&amm.curve).unwrap();
        let calculator = curve.calculator;
        if !calculator.allows_deposits() {
            return Err(SwapError::UnsupportedCurveOperation.into());
        }

        let current_pool_mint_supply = u128::try_from(ctx.accounts.pool_mint.supply).unwrap();
        let (pool_token_amount, pool_mint_supply) = if current_pool_mint_supply > 0 {
            (
                u128::try_from(pool_token_amount).unwrap(),
                current_pool_mint_supply,
            )
        } else {
            (calculator.new_pool_supply(), calculator.new_pool_supply())
        };

        let results = calculator
            .pool_tokens_to_trading_tokens(
                pool_token_amount,
                pool_mint_supply,
                u128::try_from(ctx.accounts.token_a.amount).unwrap(),
                u128::try_from(ctx.accounts.token_b.amount).unwrap(),
                RoundDirection::Ceiling,
            )
            .ok_or(SwapError::ZeroTradingTokens)?;
        let token_a_amount = u64::try_from(results.token_a_amount).unwrap();
        let token_b_amount = u64::try_from(results.token_b_amount).unwrap();
       

        let pool_token_amount = u64::try_from(pool_token_amount).unwrap();

        let seeds = &[&amm.to_account_info().key.to_bytes(), &[amm.bump_seed][..]];

        token::transfer(
            ctx.accounts
                .into_transfer_to_token_a_context()
                .with_signer(&[&seeds[..]]),
            token_a_amount,
        )?;

        token::transfer(
            ctx.accounts
                .into_transfer_to_token_b_context()
                .with_signer(&[&seeds[..]]),
            token_b_amount,
        )?;

        token::mint_to(
            ctx.accounts
                .into_mint_to_context()
                .with_signer(&[&seeds[..]]),
            u64::try_from(pool_token_amount).unwrap(),
        )?;

        Ok(())
    }
    pub fn withdraw_liquidity(
        ctx: Context<WithdrawLiquidity>,
        pool_token_amount: u64,
        _minimum_token_a_amount: u64,
        _minimum_token_b_amount: u64,
    ) -> ProgramResult {
        let amm = &mut ctx.accounts.amm;

        let curve = build_curve(&amm.curve).unwrap();
        let fees = build_fees(&amm.fees).unwrap();

        let calculator = curve.calculator;
        if !calculator.allows_deposits() {
            return Err(SwapError::UnsupportedCurveOperation.into());
        }


        let withdraw_fee: u128 = if *ctx.accounts.fee_account.key == *ctx.accounts.source_info.key {
            // withdrawing from the fee account, don't assess withdraw fee
            0
        } else {
            fees.owner_withdraw_fee(u128::try_from(pool_token_amount).unwrap())
                .ok_or(SwapError::FeeCalculationFailure)?
        };
        let pool_token_amount = u128::try_from(pool_token_amount)
            .unwrap()
            .checked_sub(withdraw_fee)
            .ok_or(SwapError::CalculationFailure)?;

        let results = calculator
            .pool_tokens_to_trading_tokens(
                pool_token_amount,
                u128::try_from(ctx.accounts.pool_mint.supply).unwrap(),
                u128::try_from(ctx.accounts.token_a.amount).unwrap(),
                u128::try_from(ctx.accounts.token_b.amount).unwrap(),
                RoundDirection::Floor,
            )
            .ok_or(SwapError::ZeroTradingTokens)?;

        let token_a_amount = u64::try_from(results.token_a_amount).unwrap();
        let token_a_amount = std::cmp::min(ctx.accounts.token_a.amount, token_a_amount);
       
        let token_b_amount = u64::try_from(results.token_b_amount).unwrap();
        let token_b_amount = std::cmp::min(ctx.accounts.token_b.amount, token_b_amount);

        let seeds = &[&amm.to_account_info().key.to_bytes(), &[amm.bump_seed][..]];

        if withdraw_fee > 0 {
            token::transfer(
                ctx.accounts.into_transfer_to_fee_account_context(),
                u64::try_from(withdraw_fee).unwrap(),
            )?;
        }
        token::burn(
            ctx.accounts.into_burn_context(),
            u64::try_from(pool_token_amount).unwrap(),
        )?;

        if token_a_amount > 0 {
            token::transfer(
                ctx.accounts
                    .into_transfer_to_token_a_context()
                    .with_signer(&[&seeds[..]]),
                token_a_amount,
            )?;
        }
        if token_b_amount > 0 {
            token::transfer(
                ctx.accounts
                    .into_transfer_to_token_b_context()
                    .with_signer(&[&seeds[..]]),
                token_a_amount,
            )?;
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    pub authority: AccountInfo<'info>,
    #[account(signer, zero)]
    pub amm: ProgramAccount<'info, AMM>,
    #[account(mut)]
    pub pool_mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub fee_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub destination: Account<'info, TokenAccount>,
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    pub authority: AccountInfo<'info>,
    pub amm: ProgramAccount<'info, AMM>,
    #[account(signer)]
    pub user_transfer_authority: AccountInfo<'info>,
    #[account(mut)]
    pub source_info: AccountInfo<'info>,
    #[account(mut)]
    pub destination_info: AccountInfo<'info>,
    #[account(mut)]
    pub swap_source: Account<'info, TokenAccount>,
    #[account(mut)]
    pub swap_destination: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_mint: Account<'info, Mint>,
    #[account(mut)]
    pub fee_account: Account<'info, TokenAccount>,
    pub token_program: AccountInfo<'info>,
    pub host_fee_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct DepositLiquidity<'info> {
    pub amm: ProgramAccount<'info, AMM>,
    pub authority: AccountInfo<'info>,
    #[account(signer)]
    pub user_transfer_authority_info: AccountInfo<'info>,
    #[account(mut)]
    pub source_a_info: AccountInfo<'info>,
    #[account(mut)]
    pub source_b_info: AccountInfo<'info>,
    #[account(mut)]
    pub token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_mint: Account<'info, Mint>,
    #[account(mut)]
    pub destination: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct WithdrawLiquidity<'info> {
    pub amm: ProgramAccount<'info, AMM>,
    pub authority: AccountInfo<'info>,
    #[account(signer)]
    pub user_transfer_authority_info: AccountInfo<'info>,
    #[account(mut)]
    pub source_info: AccountInfo<'info>,
    #[account(mut)]
    pub token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_mint: Account<'info, Mint>,
    #[account(mut)]
    pub dest_token_a_info: AccountInfo<'info>,
    #[account(mut)]
    pub dest_token_b_info: AccountInfo<'info>,
    #[account(mut)]
    pub fee_account: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}



#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct FeesInput {
    pub trade_fee_numerator: u64,
    pub trade_fee_denominator: u64,
    pub owner_trade_fee_numerator: u64,
    pub owner_trade_fee_denominator: u64,
    pub owner_withdraw_fee_numerator: u64,
    pub owner_withdraw_fee_denominator: u64,
    pub host_fee_numerator: u64,
    pub host_fee_denominator: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct CurveInput {
    pub curve_type: u8,
    pub curve_parameters: u64,
}

#[account]
pub struct AMM {
    /// Is the swap initialized, with data written to it
    pub is_initialized: bool,
    /// Bump seed used to generate the program address / authority
    pub bump_seed: u8,
    /// Token program ID associated with the swap
    pub token_program_id: Pubkey,
    /// Address of token A liquidity account
    pub token_a_account: Pubkey,
    /// Address of token B liquidity account
    pub token_b_account: Pubkey,
    /// Address of pool token mint
    pub pool_mint: Pubkey,
    /// Address of token A mint
    pub token_a_mint: Pubkey,
    /// Address of token B mint
    pub token_b_mint: Pubkey,
    /// Address of pool fee account
    pub pool_fee_account: Pubkey,
    /// Fees associated with swap
    pub fees: FeesInput,
    /// Curve associated with swap
    pub curve: CurveInput,
}

#[error]
pub enum SwapError {
    // 0.
    // The account cannot be initialized because it is already being used.
    #[msg("Swap account already in use")]
    AlreadyInUse,
    // The program address provided doesn't match the value generated by the program.
    #[msg("Invalid program address generated from bump seed and key")]
    InvalidProgramAddress,
    // The owner of the input isn't set to the program address generated by the program.
    #[msg("Input account owner is not the program address")]
    InvalidOwner,
    // The owner of the pool token output is set to the program address generated by the program.
    #[msg("Output pool account owner cannot be the program address")]
    InvalidOutputOwner,
    // The deserialization of the account returned something besides State::Mint.
    #[msg("Deserialized account is not an SPL Token mint")]
    ExpectedMint,

    // 5.
    // The deserialization of the account returned something besides State::Account.
    #[msg("Deserialized account is not an SPL Token account")]
    ExpectedAccount,
    // The input token account is empty.
    #[msg("Input token account empty")]
    EmptySupply,
    // The pool token mint has a non-zero supply.
    #[msg("Pool token mint has a non-zero supply")]
    InvalidSupply,
    // The provided token account has a delegate.
    #[msg("Token account has a delegate")]
    InvalidDelegate,
    // The input token is invalid for swap.
    #[msg("InvalidInput")]
    InvalidInput,

    // 10.
    // Address of the provided swap token account is incorrect.
    #[msg("Address of the provided swap token account is incorrect")]
    IncorrectSwapAccount,
    // Address of the provided pool token mint is incorrect
    #[msg("Address of the provided pool token mint is incorrect")]
    IncorrectPoolMint,
    // The output token is invalid for swap.
    #[msg("InvalidOutput")]
    InvalidOutput,
    // General calculation failure due to overflow or underflow
    #[msg("General calculation failure due to overflow or underflow")]
    CalculationFailure,
    // Invalid instruction number passed in.
    #[msg("Invalid instruction")]
    InvalidInstruction,

    // 15.
    // Swap input token accounts have the same mint
    #[msg("Swap input token accounts have the same mint")]
    RepeatedMint,
    // Swap instruction exceeds desired slippage limit
    #[msg("Swap instruction exceeds desired slippage limit")]
    ExceededSlippage,
    // The provided token account has a close authority.
    #[msg("Token account has a close authority")]
    InvalidCloseAuthority,
    // The pool token mint has a freeze authority.
    #[msg("Pool token mint has a freeze authority")]
    InvalidFreezeAuthority,
    // The pool fee token account is incorrect
    #[msg("Pool fee token account incorrect")]
    IncorrectFeeAccount,

    // 20.
    // Given pool token amount results in zero trading tokens
    #[msg("Given pool token amount results in zero trading tokens")]
    ZeroTradingTokens,
    // The fee calculation failed due to overflow, underflow, or unexpected 0
    #[msg("Fee calculation failed due to overflow, underflow, or unexpected 0")]
    FeeCalculationFailure,
    // ConversionFailure
    #[msg("Conversion to u64 failed with an overflow or underflow")]
    ConversionFailure,
    // The provided fee does not match the program owner's constraints
    #[msg("The provided fee does not match the program owner's constraints")]
    InvalidFee,
    // The provided token program does not match the token program expected by the swap
    #[msg("The provided token program does not match the token program expected by the swap")]
    IncorrectTokenProgramId,

    // 25.
    // The provided curve type is not supported by the program owner
    #[msg("The provided curve type is not supported by the program owner")]
    UnsupportedCurveType,
    // The provided curve parameters are invalid
    #[msg("The provided curve parameters are invalid")]
    InvalidCurve,
    // The operation cannot be performed on the given curve
    #[msg("The operation cannot be performed on the given curve")]
    UnsupportedCurveOperation,
}

pub struct SwapConstraints<'a> {
    /// Owner of the program
    pub owner_key: &'a str,
    /// Valid curve types
    pub valid_curve_types: &'a [CurveType],
    /// Valid fees
    pub fees: &'a Fees,
}

pub const SWAP_CONSTRAINTS: Option<SwapConstraints> = {
    #[cfg(feature = "production")]
    {
        Some(SwapConstraints {
            owner_key: OWNER_KEY,
            valid_curve_types: VALID_CURVE_TYPES,
            fees: FEES,
        })
    }
    #[cfg(not(feature = "production"))]
    {
        None
    }
};

impl<'a> SwapConstraints<'a> {
    /// Checks that the provided curve is valid for the given constraints
    pub fn validate_curve(&self, swap_curve: &SwapCurve) -> Result<()> {
        if self
            .valid_curve_types
            .iter()
            .any(|x| *x == swap_curve.curve_type)
        {
            Ok(())
        } else {
            Err(SwapError::UnsupportedCurveType.into())
        }
    }

    /// Checks that the provided curve is valid for the given constraints
    pub fn validate_fees(&self, fees: &Fees) -> Result<()> {
        if fees.trade_fee_numerator >= self.fees.trade_fee_numerator
            && fees.trade_fee_denominator == self.fees.trade_fee_denominator
            && fees.owner_trade_fee_numerator >= self.fees.owner_trade_fee_numerator
            && fees.owner_trade_fee_denominator == self.fees.owner_trade_fee_denominator
            && fees.owner_withdraw_fee_numerator >= self.fees.owner_withdraw_fee_numerator
            && fees.owner_withdraw_fee_denominator == self.fees.owner_withdraw_fee_denominator
            && fees.host_fee_numerator == self.fees.host_fee_numerator
            && fees.host_fee_denominator == self.fees.host_fee_denominator
        {
            Ok(())
        } else {
            Err(SwapError::InvalidFee.into())
        }
    }
}

// Context

impl<'info> Initialize<'info> {
    fn into_mint_to_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: self.pool_mint.to_account_info().clone(),
            to: self.destination.to_account_info().clone(),
            authority: self.authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

impl<'info> DepositLiquidity<'info> {
    fn into_transfer_to_token_a_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.source_a_info.clone(),
            to: self.token_a.to_account_info().clone(),
            authority: self.user_transfer_authority_info.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_transfer_to_token_b_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.source_b_info.clone(),
            to: self.token_b.to_account_info().clone(),
            authority: self.user_transfer_authority_info.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_mint_to_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: self.pool_mint.to_account_info().clone(),
            to: self.destination.to_account_info().clone(),
            authority: self.authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}


impl<'info> WithdrawLiquidity<'info> {
    fn into_transfer_to_fee_account_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.source_info.clone(),
            to: self.fee_account.to_account_info().clone(),
            authority: self.user_transfer_authority_info.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_burn_context(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        let cpi_accounts = Burn {
            mint: self.pool_mint.to_account_info().clone(),
            to: self.source_info.clone(),
            authority: self.user_transfer_authority_info.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_transfer_to_token_a_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.token_a.to_account_info().clone(),
            to: self.dest_token_a_info.clone(),
            authority: self.authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_transfer_to_token_b_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.token_b.to_account_info().clone(),
            to: self.dest_token_b_info.clone(),
            authority: self.authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}


impl<'info> Swap<'info> {
    fn into_transfer_to_swap_source_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.source_info.clone(),
            to: self.swap_source.to_account_info().clone(),
            authority: self.user_transfer_authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_transfer_to_destination_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.swap_destination.to_account_info().clone(),
            to: self.destination_info.clone(),
            authority: self.authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

// Utils

/// calculates the authority id by generating a program address.
pub fn authority_id(program_id: &Pubkey, my_info: &Pubkey, bump_seed: u8) -> Result<Pubkey> {
    Pubkey::create_program_address(&[&my_info.to_bytes()[..32], &[bump_seed]], program_id)
        .or(Err(SwapError::InvalidProgramAddress.into()))
}

/// Build curve object and Fee object
pub fn build_curve(curve_input: &CurveInput) -> Result<SwapCurve> {
    let curve_type = CurveType::try_from(curve_input.curve_type).unwrap();
    let culculator: Box<dyn CurveCalculator> = match curve_type {
        CurveType::ConstantProduct => Box::new(ConstantProductCurve {}),
        CurveType::ConstantPrice => Box::new(ConstantPriceCurve {
            token_b_price: curve_input.curve_parameters,
        }),
        CurveType::Stable => Box::new(StableCurve {
            amp: curve_input.curve_parameters,
        }),
        CurveType::Offset => Box::new(OffsetCurve {
            token_b_offset: curve_input.curve_parameters,
        }),
    };
    let curve = SwapCurve {
        curve_type: curve_type,
        calculator: culculator,
    };
    Ok(curve)
}

pub fn build_fees(fees_input: &FeesInput) -> Result<Fees> {
    let fees = Fees {
        trade_fee_numerator: fees_input.trade_fee_numerator,
        trade_fee_denominator: fees_input.trade_fee_denominator,
        owner_trade_fee_numerator: fees_input.owner_trade_fee_numerator,
        owner_trade_fee_denominator: fees_input.owner_trade_fee_denominator,
        owner_withdraw_fee_numerator: fees_input.owner_withdraw_fee_numerator,
        owner_withdraw_fee_denominator: fees_input.owner_withdraw_fee_denominator,
        host_fee_numerator: fees_input.host_fee_numerator,
        host_fee_denominator: fees_input.host_fee_denominator,
    };
    Ok(fees)
}
