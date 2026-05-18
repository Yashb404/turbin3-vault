use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Can't withdraw given amount due to insufficient balance")]
    InsufficientBalance,

    #[msg("You do not have authority to perform actions on this account")]
    InvalidAuthority,
}
