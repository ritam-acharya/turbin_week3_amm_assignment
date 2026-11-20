use anchor_lang::prelude::*;
use anchor_spl::{ 
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
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = signer,
    )]
    pub user_x: Box<Account<'info, TokenAccount>>, 
    // user y account
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = signer,
    )]
    pub user_y: Box<Account<'info, TokenAccount>>,
    // user lp account
    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = signer
    )]
    pub user_lp: Box<Account<'info, TokenAccount>>,
    // token account
    pub token_program: Program<'info, Token>
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64, min_x: u64, min_y: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount);
    
        let  amounts = ConstantProduct::xy_withdraw_amounts_from_l(self.vault_x.amount, self.vault_y.amount, self.lp_mint.supply, amount, 6).unwrap();
        require!(amounts.x >= min_x && amounts.y >= min_y, ErrorCode::SlippageExceeded);
        self.transfer_token(true, amounts.x)?;
        self.transfer_token(false, amounts.y)?;
        self.burn_token(amount)?;
        Ok(())
    }
    
    fn transfer_token(&mut self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => {
                let from = self.vault_x.to_account_info();
                let to = self.user_x.to_account_info();
                (from, to)
            },
            false => {
                let from = self.vault_y.to_account_info();
                let to = self.user_y.to_account_info();
                (from, to)   
            }
        };
        let cpi_program = self.token_program.key();
        let cpi_accounts = Transfer {
            from,
            to,
            authority: self.config.to_account_info()
        };
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"config",
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump]
        ]];
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        transfer(cpi_context, amount)
    }
    
    fn burn_token(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.key();
        let cpi_accounts = Burn {
            mint: self.lp_mint.to_account_info(),
            from: self.user_lp.to_account_info(),
            authority: self.signer.to_account_info()
        };
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        burn(cpi_context, amount)
    }
}