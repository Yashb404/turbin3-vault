
use anchor_lang::{
    prelude::*, system_program::{Transfer, transfer},
};

use crate::{VaultState, VAULT_SEED,error::CustomError};

#[derive(Accounts)]
pub struct Close<'info> {
    // Only the recorded authority can close the vault.
    #[account(mut)]
    pub authority: Signer<'info>,

    // Anchor will close this state account at the end of the instruction and
    // send its lamports back to the authority.
    #[account(
      mut, 
      has_one = authority @ CustomError::InvalidAuthority,
      seeds = [VAULT_SEED.as_bytes(),vault_state.authority.key().as_ref()],
      bump = vault_state.state_bump,
      close = authority
    )]
    pub vault_state: Account<'info, VaultState>,
    
    // The vault PDA is drained manually before the state account closes.
    #[account(
        mut,
        seeds = [VAULT_SEED.as_bytes(),vault_state.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>
}

impl <'info> Close <'info> {
    pub fn close (&mut self) -> Result<()> {
        let vault_info = self.vault.to_account_info();
      // Move whatever is left in the vault PDA back to the authority first.
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
        transfer(cpi_ctx, self.vault.lamports())?;
        Ok(())
    }
}