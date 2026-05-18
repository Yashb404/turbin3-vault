
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{VaultState, VAULT_SEED,error::CustomError};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
      mut, 
      has_one = authority @ CustomError::InvalidAuthority,
      seeds = [VAULT_SEED.as_bytes(),vault_state.authority.key().as_ref()],
      bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,
    
    #[account(
        mut,
        seeds = [VAULT_SEED.as_bytes(),vault_state.key().as_ref()],
        bump = vault_state.vault_bump, 
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
      
      let vault_info = self.vault.to_account_info();
      let rent_exempt = Rent::get()?.minimum_balance(vault_info.data_len());
      let balance_after_vault = self.vault.get_lamports().checked_sub(amount).ok_or(CustomError::InsufficientBalance)?;
      // Checking if balance is higher than the rent exempt after withdrawal
      require!(balance_after_vault >= rent_exempt, CustomError::InsufficientBalance);
      
      let accounts = Transfer {
            from: vault_info,
            to: self.authority.to_account_info(),
        };

    
        let vault_state_key = self.vault_state.key();
        let seeds = &[
            VAULT_SEED.as_bytes(),
            vault_state_key.as_ref(),
            &[self.vault_state.vault_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(system_program::id(), accounts,signer_seeds);
        transfer(cpi_ctx, amount)?;
        Ok(())
    }
}
