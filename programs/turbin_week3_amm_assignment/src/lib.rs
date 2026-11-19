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
        initialize::initialize(ctx,  fee, seed, authority)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64, max_x: u64, max_y: u64) -> Result<()> {
        deposit::deposit(ctx, amount, max_x, max_y)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64, min_x: u64, min_y: u64) -> Result<()> {
        withdraw::withdraw(ctx, amount, min_x, min_y)
    }

    pub fn swap(ctx: Context<Swap>, is_x: bool, amount: u64, min: u64) -> Result<()> {
        swap::swap(ctx, is_x, amount, min)
    }
}
