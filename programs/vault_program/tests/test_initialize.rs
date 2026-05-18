use {
    anchor_lang::{prelude::*, InstructionData, ToAccountMetas},
    litesvm::LiteSVM,
    solana_instruction::Instruction,
    solana_keypair::Keypair,
    solana_message::Message,
    solana_signer::Signer,
    solana_transaction::Transaction,
    vault_program::{
        self, accounts as vault_accounts, instruction as vault_ix, VaultState, VAULT_SEED,
    },
};

const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

fn sol(amount: u64) -> u64 {
    amount.checked_mul(LAMPORTS_PER_SOL).unwrap()
}

fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();

    let program_bytes = include_bytes!("../../../target/deploy/vault_program.so");
    svm.add_program(vault_program::id(), program_bytes).unwrap();

    let user = Keypair::new();

    svm.airdrop(&user.pubkey(), sol(10)).unwrap();

    (svm, user)
}

fn process_ix(svm: &mut LiteSVM, ix: Instruction, payer: &Keypair, signers: &[&Keypair]) {
    let latest_blockhash = svm.latest_blockhash();

    let tx = Transaction::new(
        signers,
        Message::new(&[ix], Some(&payer.pubkey())),
        latest_blockhash,
    );

    svm.send_transaction(tx).unwrap();
}

fn derive_accounts(authority: &Pubkey) -> (Pubkey, u8, Pubkey, u8) {
    let program_id = vault_program::id();

    let (vault_state, state_bump) =
        Pubkey::find_program_address(&[VAULT_SEED.as_bytes(), authority.as_ref()], &program_id);

    let (vault, vault_bump) =
        Pubkey::find_program_address(&[VAULT_SEED.as_bytes(), vault_state.as_ref()], &program_id);

    (vault_state, state_bump, vault, vault_bump)
}

#[test]
fn test_initialize_deposit_withdraw_close() {
    let (mut svm, user) = setup();

    let (vault_state, state_bump, vault, vault_bump) = derive_accounts(&user.pubkey());

    // Initialize
    let init_ix = Instruction {
        program_id: vault_program::id(),
        accounts: vault_accounts::Initialize {
            authority: user.pubkey(),
            vault_state,
            vault,
            system_program: system_program::id(),
        }
        .to_account_metas(None),
        data: vault_ix::Initialize {}.data(),
    };

    process_ix(&mut svm, init_ix, &user, &[&user]);

    let vault_state_account = svm.get_account(&vault_state).unwrap();

    let state = VaultState::try_deserialize(&mut vault_state_account.data.as_slice()).unwrap();

    assert_eq!(state.authority, user.pubkey());
    assert_eq!(state.state_bump, state_bump);
    assert_eq!(state.vault_bump, vault_bump);

    // Deposit
    const DEPOSIT_SOL: u64 = 5_000_000_000;

    let deposit_ix = Instruction {
        program_id: vault_program::id(),
        accounts: vault_accounts::Deposit {
            authority: user.pubkey(),
            vault_state,
            vault,
            system_program: system_program::id(),
        }
        .to_account_metas(None),
        data: vault_ix::Deposit {
            amount: DEPOSIT_SOL,
        }
        .data(),
    };

    process_ix(&mut svm, deposit_ix, &user, &[&user]);

    assert_eq!(svm.get_balance(&vault).unwrap(), DEPOSIT_SOL);

    // Withdraw
    const WITHDRAW_SOL: u64 = 4_000_000_000;

    let withdraw_ix = Instruction {
        program_id: vault_program::id(),
        accounts: vault_accounts::Withdraw {
            authority: user.pubkey(),
            vault_state,
            vault,
            system_program: system_program::id(),
        }
        .to_account_metas(None),
        data: vault_ix::Withdraw {
            amount: WITHDRAW_SOL,
        }
        .data(),
    };

    process_ix(&mut svm, withdraw_ix, &user, &[&user]);

    assert_eq!(svm.get_balance(&vault).unwrap(), DEPOSIT_SOL - WITHDRAW_SOL);

    // Close
    let user_balance_before = svm.get_balance(&user.pubkey()).unwrap();

    let close_ix = Instruction {
        program_id: vault_program::id(),
        accounts: vault_accounts::Close {
            authority: user.pubkey(),
            vault_state,
            vault,
            system_program: system_program::id(),
        }
        .to_account_metas(None),
        data: vault_ix::Close {}.data(),
    };

    process_ix(&mut svm, close_ix, &user, &[&user]);

    assert!(svm.get_account(&vault).is_none());
    assert!(svm.get_account(&vault_state).is_none());

    assert!(svm.get_balance(&user.pubkey()).unwrap() > user_balance_before);
}

#[test]
#[should_panic(expected = "InsufficientBalance")]
fn test_withdraw_more_than_balance() {
    let (mut svm, user) = setup();

    let (vault_state, _, vault, _) = derive_accounts(&user.pubkey());

    // Initialize
    let init_ix = Instruction {
        program_id: vault_program::id(),
        accounts: vault_accounts::Initialize {
            authority: user.pubkey(),
            vault_state,
            vault,
            system_program: system_program::id(),
        }
        .to_account_metas(None),
        data: vault_ix::Initialize {}.data(),
    };

    process_ix(&mut svm, init_ix, &user, &[&user]);

    // Deposit 1 SOL
    let deposit_ix = Instruction {
        program_id: vault_program::id(),
        accounts: vault_accounts::Deposit {
            authority: user.pubkey(),
            vault_state,
            vault,
            system_program: system_program::id(),
        }
        .to_account_metas(None),
        data: vault_ix::Deposit {
            amount: sol(1),
        }
        .data(),
    };

    process_ix(&mut svm, deposit_ix, &user, &[&user]);

    // Try withdrawing 5 SOL
    let withdraw_ix = Instruction {
        program_id: vault_program::id(),
        accounts: vault_accounts::Withdraw {
            authority: user.pubkey(),
            vault_state,
            vault,
            system_program: system_program::id(),
        }
        .to_account_metas(None),
        data: vault_ix::Withdraw {
            amount: sol(5),
        }
        .data(),
    };

    process_ix(&mut svm, withdraw_ix, &user, &[&user]);
}

#[test]
#[should_panic(expected = "InvalidAuthority")]
fn test_withdraw_from_different_authority() {
    let (mut svm, user) = setup();

    let attacker = Keypair::new();

    svm.airdrop(&attacker.pubkey(), sol(5))
        .unwrap();

    let (vault_state, _, vault, _) = derive_accounts(&user.pubkey());

    // Initialize
    let init_ix = Instruction {
        program_id: vault_program::id(),
        accounts: vault_accounts::Initialize {
            authority: user.pubkey(),
            vault_state,
            vault,
            system_program: system_program::id(),
        }
        .to_account_metas(None),
        data: vault_ix::Initialize {}.data(),
    };

    process_ix(&mut svm, init_ix, &user, &[&user]);

    // Attacker withdraw attempt
    let withdraw_ix = Instruction {
        program_id: vault_program::id(),
        accounts: vault_accounts::Withdraw {
            authority: attacker.pubkey(),
            vault_state,
            vault,
            system_program: system_program::id(),
        }
        .to_account_metas(None),
        data: vault_ix::Withdraw {
            amount: sol(1),
        }
        .data(),
    };

    process_ix(&mut svm, withdraw_ix, &attacker, &[&attacker]);
}

#[test]
#[should_panic(expected = "InvalidAuthority")]
fn test_close_from_different_authority() {
    let (mut svm, user) = setup();

    let attacker = Keypair::new();

    svm.airdrop(&attacker.pubkey(), sol(5))
        .unwrap();

    let (vault_state, _, vault, _) = derive_accounts(&user.pubkey());

    // Initialize
    let init_ix = Instruction {
        program_id: vault_program::id(),
        accounts: vault_accounts::Initialize {
            authority: user.pubkey(),
            vault_state,
            vault,
            system_program: system_program::id(),
        }
        .to_account_metas(None),
        data: vault_ix::Initialize {}.data(),
    };

    process_ix(&mut svm, init_ix, &user, &[&user]);

    let close_ix = Instruction {
        program_id: vault_program::id(),
        accounts: vault_accounts::Close {
            authority: attacker.pubkey(),
            vault_state,
            vault,
            system_program: system_program::id(),
        }
        .to_account_metas(None),
        data: vault_ix::Close {}.data(),
    };

    process_ix(&mut svm, close_ix, &attacker, &[&attacker]);
}

#[test]
fn test_deposit_from_different_authority_allowed() {
    let (mut svm, user) = setup();

    let depositor = Keypair::new();

    svm.airdrop(&depositor.pubkey(), sol(5))
        .unwrap();

    let (vault_state, _, vault, _) = derive_accounts(&user.pubkey());

    // Initialize
    let init_ix = Instruction {
        program_id: vault_program::id(),
        accounts: vault_accounts::Initialize {
            authority: user.pubkey(),
            vault_state,
            vault,
            system_program: system_program::id(),
        }
        .to_account_metas(None),
        data: vault_ix::Initialize {}.data(),
    };

    process_ix(&mut svm, init_ix, &user, &[&user]);

    // Different depositor deposits
    let deposit_ix = Instruction {
        program_id: vault_program::id(),
        accounts: vault_accounts::Deposit {
            authority: depositor.pubkey(),
            vault_state,
            vault,
            system_program: system_program::id(),
        }
        .to_account_metas(None),
        data: vault_ix::Deposit {
            amount: sol(2),
        }
        .data(),
    };

    process_ix(&mut svm, deposit_ix, &depositor, &[&depositor]);

    assert_eq!(svm.get_balance(&vault).unwrap(), sol(2));
}
