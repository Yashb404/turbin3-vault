use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{VaultState, VAULT_SEED};

#[derive(Accounts)]
pub struct Deposit<'info> {
    // Anyone can deposit, but the source account still has to sign because the
    // system program moves lamports out of that wallet.
    #[account(mut)]
    pub authority: Signer<'info>,

    // Re-derive the vault state PDA from the stored authority so deposits can
    // only target the vault created for that owner.
    #[account(
      mut, 
      seeds = [VAULT_SEED.as_bytes(),vault_state.authority.key().as_ref()],
      bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,
    
    // The actual lamport bucket for the vault. It is derived from the state
    // account so the two accounts stay linked.
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
        // This uses a CPI to the system program instead of manually editing
        // lamports, which keeps the transfer semantics consistent with Solana.
        let accounts = Transfer {
            from: self.authority.to_account_info(),
            to: self.vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(system_program::id(), accounts);
        transfer(cpi_ctx, amount)?;
        Ok(())
    }
}
