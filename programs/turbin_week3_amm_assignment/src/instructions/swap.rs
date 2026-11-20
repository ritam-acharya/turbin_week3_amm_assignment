use crate::error::ErrorCode;
use crate::Config;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use crate::ConstantProduct;
use crate::LiquidityPair;

#[derive(Accounts)]
pub struct Swap<'info> {
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
        init_if_needed,
        payer = signer,
        associated_token::mint = mint_x,
        associated_token::authority = signer,
    )]
    pub user_x: Box<Account<'info, TokenAccount>>,
    // user y account
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint_y,
        associated_token::authority = signer,
    )]
    pub user_y: Box<Account<'info, TokenAccount>>,
    // system account
    pub system_program: Program<'info, System>,
    // token account
    pub token_program: Program<'info, Token>,
    // associated token account
    pub associated_token_program: Program<'info, AssociatedToken>,
}


impl<'info> Swap<'info> {
    pub fn swap(&mut self, is_x: bool, amount: u64, min: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount);
        let mut curve = ConstantProduct::init(
            self.vault_x.amount,
            self.vault_y.amount,
            self.lp_mint.supply,
            self.config.fee,
            Some(6),
        )
        .unwrap();
        
        let p = match is_x {
            true => LiquidityPair::X,
            false => LiquidityPair::Y,
        };
        
        let swap_result = curve.swap(p, amount, min)?;
    
        self.deposit_token(swap_result.deposit, is_x)?;
        self.withdraw_token(swap_result.withdraw, is_x)?;
    
        Ok(())
    }
    
    fn deposit_token(&mut self, amount: u64, is_x: bool) -> Result<()> {
        let cpi_program = self.token_program.key();
        let (from, to) = match is_x {
            true => {
                let deposit_from = self.user_x.to_account_info();
                let deposit_to = self.vault_x.to_account_info();
                (deposit_from, deposit_to)
            },
            false => {
                let deposit_from = self.user_y.to_account_info();
                let deposit_to = self.vault_y.to_account_info();
                (deposit_from, deposit_to)
            }
        };
        let cpi_accounts = Transfer {
            from,
            to,
            authority: self.signer.to_account_info(),
        };
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_context, amount)
    }
    
    fn withdraw_token(&mut self, amount: u64, is_x: bool) -> Result<()> {
        let cpi_program = self.token_program.key();
        let (from, to) = match is_x {
            true => {
                let withdraw_from = self.vault_y.to_account_info();
                let withdraw_to = self.user_y.to_account_info();
                (withdraw_from, withdraw_to)
            },
            false => {
                let withdraw_from = self.vault_x.to_account_info();
                let withdraw_to = self.user_x.to_account_info();
                (withdraw_from, withdraw_to)
            }
        };
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

}