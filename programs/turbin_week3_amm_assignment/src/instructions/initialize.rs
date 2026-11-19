use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, 
    token::{Mint, Token, TokenAccount}
};

use crate::Config;


#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Initialize<'info> {
    // signer
    #[account(mut)]
    pub signer: Signer<'info>,
    // config pda
    #[account(
        init,
        payer = signer,
        seeds = [b"config", seed.to_le_bytes().as_ref()],
        bump,
        space = Config::DISCRIMINATOR.len() + Config::INIT_SPACE
    )]
    pub config: Box<Account<'info, Config>>,
    // lp mint account pda
    #[account(
        init,
        payer = signer,
        seeds = [b"lp", config.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = config
    )]
    pub lp_mint: Box<Account<'info, Mint>>,
    // mint x account
    pub mint_x: Box<Account<'info, Mint>>,
    // mint y account
    pub mint_y: Box<Account<'info, Mint>>,
    // vault x account pda
    #[account(
        init,
        payer = signer,
        associated_token::mint = mint_x,
        associated_token::authority = config
    )]
    pub vault_x: Box<Account<'info, TokenAccount>>,
    // vault y account pda
    #[account(
        init,
        payer = signer,
        associated_token::mint = mint_y,
        associated_token::authority = config
    )]
    pub vault_y: Box<Account<'info, TokenAccount>>,
    // System program
    pub system_program: Program<'info, System>,
    // Token program
    pub token_program: Program<'info, Token>,
    // Associated token program
    pub associated_token_program: Program<'info, AssociatedToken>
}

pub fn initialize(ctx: Context<Initialize>, fee: u64, seed: u64, authority: Option<Pubkey>) -> Result<()> {
    ctx.accounts.config.set_inner(Config {
        seed,
        fee,
        authority,
        is_locked: false,
        config_bump: ctx.bumps.config,
        lp_bump: ctx.bumps.lp_mint,
        mint_x: ctx.accounts.mint_x.key(),
        mint_y: ctx.accounts.mint_y.key(),
    });
    
    Ok(())
}
