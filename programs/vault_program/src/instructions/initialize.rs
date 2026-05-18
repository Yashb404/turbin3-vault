use crate::{
    constants::{ANCHOR_DISCRIMINATOR_SIZE, VAULT_SEED},
    VaultState,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer=authority,
        seeds = [VAULT_SEED.as_bytes(),authority.key().as_ref()],
        bump,
        space = ANCHOR_DISCRIMINATOR_SIZE + VaultState::INIT_SPACE
    )]
    pub vault_state: Account<'info, VaultState>,
    
    #[account(
        seeds = [VAULT_SEED.as_bytes(),vault_state.key().as_ref()],
        bump, 
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        self.vault_state.authority = self.authority.key();
        self.vault_state.state_bump = bumps.vault_state;
        self.vault_state.vault_bump = bumps.vault;
        Ok(())
    }
}
