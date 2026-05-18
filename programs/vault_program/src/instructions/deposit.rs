use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{VaultState, VAULT_SEED};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
      mut, 
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

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let accounts = Transfer {
            from: self.authority.to_account_info(),
            to: self.vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(system_program::id(), accounts);
        transfer(cpi_ctx, amount)?;
        Ok(())
    }
}
