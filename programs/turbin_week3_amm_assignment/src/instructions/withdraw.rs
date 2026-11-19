use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, 
    token::{transfer, Burn, Mint, Token, TokenAccount, Transfer, burn}
};
use crate::ConstantProduct;
use crate::Config;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    // signer
    #[account(mut)]
    pub signer: Signer<'info>,
    // lp mint account
    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump
    )]
    pub lp_mint: Box<Account<'info, Mint>>,
    // vault x account
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config
    )]
    pub vault_x: Box<Account<'info, TokenAccount>>,
    // vault y account
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config
    )]
    pub vault_y: Box<Account<'info, TokenAccount>>,
    // config account pda
    #[account(
        has_one = mint_x,
        has_one = mint_y,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump
    )]
    pub config: Box<Account<'info, Config>>,
    // mint x account
    pub mint_x: Box<Account<'info, Mint>>,
    // mint y account
    pub mint_y: Box<Account<'info, Mint>>,
    // user x account
    pub user_x: Box<Account<'info, TokenAccount>>, 
    // user y account
    pub user_y: Box<Account<'info, TokenAccount>>,
    // user lp account
    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = signer
    )]
    pub user_lp: Box<Account<'info, TokenAccount>>,
    // system account
    pub system_program: Program<'info, System>,
    // token account
    pub token_program: Program<'info, Token>,
    // associated token account 
    pub associated_token_program: Program<'info, AssociatedToken>
}

pub fn withdraw(ctx: Context<Withdraw>, amount: u64, min_x: u64, min_y: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(min_x > 0, ErrorCode::InvalidAmount);
    require!(min_y > 0, ErrorCode::InvalidAmount);

    let  amounts = ConstantProduct::xy_withdraw_amounts_from_l(ctx.accounts.vault_x.amount, ctx.accounts.vault_y.amount, ctx.accounts.lp_mint.supply, amount, 6).unwrap();
    require!(amounts.x >= min_x && amounts.y >= min_y, ErrorCode::SlippageExceeded);
    transfer_token(&ctx, true, amounts.x)?;
    transfer_token(&ctx, false, amounts.y)?;
    burn_token(&ctx, amount)?;
    Ok(())
}

fn transfer_token(ctx: &Context<Withdraw>, is_x: bool, amount: u64) -> Result<()> {
    let (from, to) = match is_x {
        true => {
            let from = ctx.accounts.vault_x.to_account_info();
            let to = ctx.accounts.user_x.to_account_info();
            (from, to)
        },
        false => {
            let from = ctx.accounts.vault_y.to_account_info();
            let to = ctx.accounts.user_y.to_account_info();
            (from, to)   
        }
    };
    let cpi_program = ctx.accounts.token_program.key();
    let cpi_accounts = Transfer {
        from,
        to,
        authority: ctx.accounts.config.to_account_info()
    };
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"config",
        &ctx.accounts.config.seed.to_le_bytes(),
        &[ctx.accounts.config.config_bump]
    ]];
    let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    transfer(cpi_context, amount)
}

fn burn_token(ctx: &Context<Withdraw>, amount: u64) -> Result<()> {
    let cpi_program = ctx.accounts.token_program.key();
    let cpi_accounts = Burn {
        mint: ctx.accounts.lp_mint.to_account_info(),
        from: ctx.accounts.user_lp.to_account_info(),
        authority: ctx.accounts.config.to_account_info()
    };
    let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
    burn(cpi_context, amount)
}