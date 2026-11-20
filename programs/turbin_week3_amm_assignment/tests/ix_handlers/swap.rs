use anchor_lang::{
    solana_program::instruction::Instruction,system_program::ID as SYSTEM_PROGRAM_ID,
InstructionData, ToAccountMetas,    
};
use anchor_spl::associated_token::{self, ID as ASSOCIATED_TOKEN_PROGRAM_ID};
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use litesvm_token::spl_token::ID as TOKEN_PROGRAM_ID;
pub fn create_swap_ix(
    payer: &Keypair,
    mint_x: Pubkey,
    mint_y: Pubkey,
    mint_lp: Pubkey,
    config: Pubkey,
    vault_x: Pubkey,
    vault_y: Pubkey,
    ) -> Instruction {
    let user = payer.pubkey();
    let user_x = associated_token::get_associated_token_address(&user, &mint_x);
    let user_y = associated_token::get_associated_token_address(&user, &mint_y);
    
    Instruction::new_with_bytes(
        turbin_week3_amm_assignment::id(),
        &turbin_week3_amm_assignment::instruction::Swap { is_x: true, amount: 10_000_000, min: 0 }.data(),
        turbin_week3_amm_assignment::accounts::Swap{
            signer: user,
            mint_x,
            mint_y,
            config,
            vault_x,
            vault_y,
            user_x,
            user_y,
            lp_mint: mint_lp,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}