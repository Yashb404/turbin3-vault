pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("5yfhPSsqiQC3Sqvvf6JaG6sGVNj4CUEmnoWJdCzWfWWA");

#[program]
pub mod vault_program {
    use super::*;

    // Keep the entrypoints thin: the account structs hold the validation rules,
    // and the helper methods below hold the business logic.

    // Create the vault state PDA and remember which wallet owns it.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)
    }

    // Move lamports from any signer into the vault PDA.
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    // Withdraw lamports back to the recorded authority, but only if the vault
    // keeps enough lamports to remain rent exempt.
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    // Drain the vault and close the state account, returning the remaining
    // lamports to the authority.
    pub fn close(ctx: Context<Close>) -> Result<()> {
        ctx.accounts.close()
    }
}
