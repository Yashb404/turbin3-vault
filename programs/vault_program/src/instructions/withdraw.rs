
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{VaultState, VAULT_SEED,error::CustomError};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    // Only the authority can authorize a withdrawal, so the caller must sign.
    #[account(mut)]
    pub authority: Signer<'info>,

    // Enforce that the recorded owner matches the signer before any lamports
    // leave the vault.
    #[account(
      mut, 
      has_one = authority @ CustomError::InvalidAuthority,
      seeds = [VAULT_SEED.as_bytes(),vault_state.authority.key().as_ref()],
      bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,
    
    // The vault PDA is the source of the withdrawal.
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
                // Keep enough lamports behind so the vault account stays rent exempt
                // after the withdrawal.
      let vault_info = self.vault.to_account_info();
      let rent_exempt = Rent::get()?.minimum_balance(vault_info.data_len());
      let balance_after_vault = self.vault.get_lamports().checked_sub(amount).ok_or(CustomError::InsufficientBalance)?;
      // Checking if balance is higher than the rent exempt after withdrawal
      require!(balance_after_vault >= rent_exempt, CustomError::InsufficientBalance);
      
            // Transfer from the vault PDA, so we need to sign with its seeds rather
            // than the authority's signature.
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
