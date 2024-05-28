// #![cfg(feature = "test-bpf")]

use std::assert_eq;

use borsh::BorshDeserialize;
use create_pda_example::UserStake;
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::Signer,
    signer::keypair::Keypair,
    system_program, system_transaction,
    transaction::Transaction,
};
use solana_validator::test_validator::*;

#[test]
fn test_validator_transaction() {
    solana_logger::setup_with_default("solana_program_runtime=debug");
    let program_id = Pubkey::new_unique();

    let (test_validator, payer) = TestValidatorGenesis::default()
        .add_program("create_pda_example", program_id)
        .start();
    let rpc_client = test_validator.get_rpc_client();

    let blockhash = rpc_client.get_latest_blockhash().unwrap();

    let alice = Keypair::new();
    // airdrop 1 SOL
    const ALICE_INIT_BALANCE: u64 = 1_000_000_000;
    {
        let tx =
            system_transaction::transfer(&payer, &alice.pubkey(), ALICE_INIT_BALANCE, blockhash);
        rpc_client.send_and_confirm_transaction(&tx).unwrap();
    }

    let (alice_pda, _) = Pubkey::find_program_address(&[alice.pubkey().as_ref()], &program_id);

    let instruction_data: Vec<u8> = Vec::new();
    let mut transaction = Transaction::new_with_payer(
        &[Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(alice.pubkey(), true),
                AccountMeta::new(alice_pda, false),
                AccountMeta::new(system_program::id(), false),
            ],
            data: instruction_data,
        }],
        Some(&alice.pubkey()),
    );
    transaction.sign(&[&alice], blockhash);

    rpc_client
        .send_and_confirm_transaction(&transaction)
        .unwrap();

    let account_data = rpc_client.get_account_data(&alice_pda).unwrap();
    let user_stake = UserStake::try_from_slice(&account_data).unwrap();

    assert!(user_stake.is_initialized);
    assert_eq!(user_stake.lamports, 42);
}
