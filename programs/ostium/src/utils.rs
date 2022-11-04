use anchor_lang::prelude::*;
use pyth_sdk_solana::{load_price_feed_from_account_info, Price, PriceFeed};

pub const UNITS_IN_ONE_QUANTITY: u64 = 100_000_000; // it means 1 ounce of gold = 10^8
pub const MIN_QUANTITY: u64 = 10_000; // minimum quantity required for opening a position
const USDC_DECIMALS: u32 = 6;

// it always enforces a precision of 6 decimals
pub fn get_current_price(price_account_info: &AccountInfo) -> u64 {
    let price_feed: PriceFeed = load_price_feed_from_account_info(&price_account_info).unwrap();
    let current_price: Price = price_feed.get_current_price().unwrap();
    to_usdc_decimals(&current_price)
}

pub fn to_usdc_decimals(current_price: &Price) -> u64 {
    let expo = -current_price.expo as u32;
    if expo < USDC_DECIMALS {
        let diff = USDC_DECIMALS - expo;
        return current_price.price as u64 * 10u64.pow(diff);
    } else {
        let diff = expo - USDC_DECIMALS;
        return current_price.price as u64 / 10u64.pow(diff);
    }
}

pub fn should_be_liquidated(collateral: i64, pnl: i64) -> bool {
    let remaining_collateral = collateral + pnl;
    let ratio = remaining_collateral * 10 / collateral;
    // if the ratio is less than 20%, then liquidate the position
    ratio < 2
}

pub fn get_ostium_fee(quantity: u64) -> u64 {
    // it represents a 0.05% fee
    quantity * 5 / 10_000
}
