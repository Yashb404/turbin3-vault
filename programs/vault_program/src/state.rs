use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account]
pub struct VaultState {
    // The wallet that is allowed to withdraw from and close this vault.
    pub authority: Pubkey,

    // Stored so later instructions can re-derive the exact PDA used for this
    // state account.
    pub state_bump: u8,

    // Stored so later instructions can re-derive the vault PDA derived from
    // this state account.
    pub vault_bump: u8,
}
