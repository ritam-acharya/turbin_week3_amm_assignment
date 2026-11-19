use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken,
    token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer},
};
use crate::ConstantProduct;

use crate::Config;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct Deposit<'info> {
    // signer
    #[account(mut)]
    pub signer: Signer<'info>,
    // mint x account
    pub mint_x: Box<Account<'info, Mint>>,
    // mint y account
    pub mint_y: Box<Account<'info, Mint>>,
    // config account pda
    #[account(
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
        has_one = mint_x,
        has_one = mint_y
    )]
    pub config: Box<Account<'info, Config>>,
    // lp mint account
    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump
    )]
    pub lp_mint: Box<Account<'info, Mint>>,
    // vault x account pda
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config
    )]
    pub vault_x: Box<Account<'info, TokenAccount>>,
    // vault y account pda
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config
    )]
    pub vault_y: Box<Account<'info, TokenAccount>>,
    // user lp account
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = lp_mint,
        associated_token::authority = signer
    )]
    pub user_lp: Box<Account<'info, TokenAccount>>,
    // user x ata account
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = signer
    )]
    pub user_x: Box<Account<'info, TokenAccount>>,
    // user y ata account
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = signer
    )]
    pub user_y: Box<Account<'info, TokenAccount>>,
    // system program
    pub system_program: Program<'info, System>,
    // token program
    pub token_program: Program<'info, Token>,
    // associated token program
    pub associated_token_program: Program<'info, AssociatedToken>
    
} 


pub fn deposit(ctx: Context<Deposit>, amount: u64, max_x: u64, max_y: u64) -> Result<()> {
    require!(ctx.accounts.config.is_locked, ErrorCode::PoolLocked);
    require_neq!(amount, 0, ErrorCode::InvalidAmount);
    require_neq!(max_x, 0, ErrorCode::InvalidAmount);
    require_neq!(max_y, 0, ErrorCode::InvalidAmount);
    
    let (x, y) = if ctx.accounts.lp_mint.supply == 0 && ctx.accounts.vault_x.amount == 0 && ctx.accounts.vault_y.amount == 0 {
        (max_x, max_y)
    }else {
        let amount = ConstantProduct::xy_deposit_amounts_from_l(ctx.accounts.user_x.amount, ctx.accounts.user_y.amount, ctx.accounts.lp_mint.supply, amount, 6).unwrap();
        require!(amount.x <= max_x && amount.y <= max_y, ErrorCode::SlippageExceeded);
        (amount.x, amount.y)
    };

    deposit_token(&ctx, true, x)?;
    deposit_token(&ctx, false, y)?;
    mint_lp_tokens(&ctx, amount)?;
    Ok(())
}

fn deposit_token(ctx: &Context<Deposit>, is_x: bool, amount: u64) -> Result<()> {
    let (from, to) = match is_x {
        true => {
            let from = ctx.accounts.user_x.to_account_info();
            let to = ctx.accounts.vault_x.to_account_info();
            (from, to)
        },
        false => {
            let from = ctx.accounts.user_y.to_account_info();
            let to = ctx.accounts.vault_y.to_account_info();
            (from, to)
        }
    };

    let cpi_program = ctx.accounts.token_program.key();
    let accounts = Transfer {
        from,
        to,
        authority: ctx.accounts.signer.to_account_info()
    };
    let cpi_context = CpiContext::new(cpi_program, accounts);

    transfer(cpi_context, amount)
}

fn mint_lp_tokens(ctx: &Context<Deposit>, amount: u64) -> Result<()> {
    let cpi_program = ctx.accounts.token_program.key();
    let cpi_accounts = MintTo {
        mint: ctx.accounts.lp_mint.to_account_info(),
        to: ctx.accounts.user_lp.to_account_info(),
        authority: ctx.accounts.config.to_account_info()
    };
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"config",
        &ctx.accounts.config.seed.to_be_bytes(),
        &[ctx.accounts.config.config_bump]
    ]];
    let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    mint_to(cpi_context, amount)
}