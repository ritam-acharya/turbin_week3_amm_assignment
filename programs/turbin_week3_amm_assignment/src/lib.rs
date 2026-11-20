pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("6Rk7ZKqJbfMY97yb13you6LorRg1Q1RESDSuERNgYoW6");

#[program]
pub mod turbin_week3_assignment {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>,  fee: u64, seed: u64, authority: Option<Pubkey>) -> Result<()> {
        ctx.accounts.initialize(fee, seed, authority, ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64, max_x: u64, max_y: u64) -> Result<()> {
        ctx.accounts.deposit(amount, max_x, max_y)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64, min_x: u64, min_y: u64) -> Result<()> {
        ctx.accounts.withdraw(amount, min_x, min_y)
    }

    pub fn swap(ctx: Context<Swap>, is_x: bool, amount: u64, min: u64) -> Result<()> {
        ctx.accounts.swap(is_x, amount, min)
    }
}
