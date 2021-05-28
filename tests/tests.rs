#![cfg(feature = "test-bpf")]

use solbridge_master_contract::state::{Bridge, Blockchain, Validator};
use solbridge_master_contract::*;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_pack::Pack, pubkey::Pubkey, system_instruction};
use solana_program_test::*;
use solana_sdk::account::Account;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

pub fn program_test() -> ProgramTest {
    ProgramTest::new(
        "solbridge_master_contract",
        id(),
        processor!(processor::Processor::process_instruction),
    )
}

pub async fn get_account(program_context: &mut ProgramTestContext, pubkey: &Pubkey) -> Account {
    program_context
        .banks_client
        .get_account(*pubkey)
        .await
        .expect("account not found")
        .expect("account empty")
}

pub async fn transfer_token(
    program_context: &mut ProgramTestContext,
    from: &Pubkey,
    to: &Pubkey,
    amount: u64,
    payer: Option<&Keypair>,
) {
    let payer = payer.unwrap_or(&program_context.payer);

    let instructions = vec![spl_token::instruction::transfer(
        &spl_token::id(),
        from,
        to,
        &payer.pubkey(),
        &[],
        amount,
    )
    .unwrap()];

    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));

    transaction.sign(&[payer], program_context.last_blockhash);

    program_context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
}

#[derive(Debug)]
struct BridgeContext {
    bridge: Keypair,
    bridge_authority: Pubkey,
}

impl BridgeContext {
    pub async fn init(program_context: &mut ProgramTestContext) -> BridgeContext {
        let bridge_key = Keypair::new();
        let (bridge_authority_pubkey, _) = Pubkey::find_program_address(
            &[bridge_key.pubkey().as_ref()],
            &id(),
        );

        let rent = program_context.banks_client.get_rent().await.unwrap();

        let mut transaction = Transaction::new_with_payer(
            &[
                system_instruction::create_account(
                    &program_context.payer.pubkey(),
                    &bridge_key.pubkey(),
                    rent.minimum_balance(Bridge::LEN),
                    Bridge::LEN as u64,
                    &id(),
                ),
                instruction::init_bridge(
                    &id(),
                    &bridge_key.pubkey(),
                    &program_context.payer.pubkey()
                )
                .unwrap(),
            ],
            Some(&program_context.payer.pubkey()),
        );

        transaction.sign(
            &[&program_context.payer, &bridge_key],
            program_context.last_blockhash,
        );
        program_context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();

        BridgeContext {
            bridge: bridge_key,
            bridge_authority: bridge_authority_pubkey
        }
    }

    pub async fn add_blockchain(&self, program_context: &mut ProgramTestContext, blockchain_id_str: String, contract_address: [u8; 32]) -> Pubkey {
        let blockchain_pubkey =
            Pubkey::create_with_seed(&self.bridge_authority, format!("blockchain_{}", blockchain_id_str).as_str(), &id()).unwrap();
        let mut transaction = Transaction::new_with_payer(
            &[
                instruction::add_blockchain(
                    &id(),
                    &self.bridge.pubkey(),
                    &blockchain_pubkey,
                    &program_context.payer.pubkey(),
                    &self.bridge_authority,
                    blockchain_id_str,
                    contract_address
                )
                    .unwrap(),
            ],
            Some(&program_context.payer.pubkey()),
        );

        transaction.sign(
            &[&program_context.payer],
            program_context.last_blockhash,
        );
        program_context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();

        blockchain_pubkey
    }

    pub async fn add_validator(&self, program_context: &mut ProgramTestContext, blockchain_id_str: String, pubkey: [u8; 32]) -> Pubkey {
        let blockchain_pubkey =
            Pubkey::create_with_seed(&self.bridge_authority, format!("blockchain_{}", blockchain_id_str).as_str(), &id()).unwrap();
        let blockchain_account = get_account(program_context, &blockchain_pubkey).await;
        let blockchain_data: Blockchain = Blockchain::try_from_slice(&blockchain_account.data).unwrap();
        let validator_account =
            Pubkey::create_with_seed(&self.bridge_authority, format!("validator_{}_{}", blockchain_id_str, blockchain_data.validators).as_str(), &id()).unwrap();
        let mut transaction = Transaction::new_with_payer(
            &[
                instruction::add_validator(
                    &id(),
                    &self.bridge.pubkey(),
                    &blockchain_pubkey,
                    &validator_account,
                    &program_context.payer.pubkey(),
                    &self.bridge_authority,
                    blockchain_id_str,
                    pubkey,

                )
                    .unwrap(),
            ],
            Some(&program_context.payer.pubkey()),
        );

        transaction.sign(
            &[&program_context.payer],
            program_context.last_blockhash,
        );
        program_context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();

        validator_account
    }

    pub async fn add_signature(&self, program_context: &mut ProgramTestContext, blockchain_pubkey: &Pubkey, validator_pubkey: &Pubkey) -> Pubkey {
        let blockchain_pubkey =
            Pubkey::create_with_seed(&self.bridge_authority, format!("blockchain_{}", blockchain_id_str).as_str(), &id()).unwrap();
        let blockchain_account = get_account(program_context, &blockchain_pubkey).await;
        let blockchain_data: Blockchain = Blockchain::try_from_slice(&blockchain_account.data).unwrap();
        let validator_account =
            Pubkey::create_with_seed(&self.bridge_authority, format!("validator_{}_{}", blockchain_id_str, blockchain_data.validators).as_str(), &id()).unwrap();


        let mut transaction = Transaction::new_with_payer(
            &[
                instruction::add_signature(
                    &id(),
                    &self.bridge.pubkey(),
                    &blockchain_pubkey,
                    &validator_pubkey,
                    &validator_account,
                    &program_context.payer.pubkey(),
                    &self.bridge_authority,
                    blockchain_id_str,
                    pubkey,

                )
                    .unwrap(),
            ],
            Some(&program_context.payer.pubkey()),
        );

        transaction.sign(
            &[&program_context.payer],
            program_context.last_blockhash,
        );
        program_context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();

        validator_account
    }
}

#[tokio::test]
async fn init_bridge_test() {
    let mut program_context = program_test().start_with_context().await;
    let bridge_context = BridgeContext::init(&mut program_context).await;

    let bridge_account = get_account(&mut program_context, &bridge_context.bridge.pubkey()).await;
    let bridge_data: Bridge = Bridge::try_from_slice(&bridge_account.data).unwrap();
    println!("{:?}", bridge_data);
    assert_eq!(bridge_data.locks, 0);
    assert_eq!(bridge_data.owner, program_context.payer.pubkey());
    assert_eq!(bridge_data.version, 1);
}

#[tokio::test]
async fn add_blockchain_test() {
    let mut program_context = program_test().start_with_context().await;
    let bridge_context = BridgeContext::init(&mut program_context).await;
    let blockchain_pubkey = bridge_context.add_blockchain(&mut program_context, String::from("ETH"), [1; 32]).await;

    let blockchain_account = get_account(&mut program_context, &blockchain_pubkey).await;
    let blockchain_data: Blockchain = Blockchain::try_from_slice(&blockchain_account.data).unwrap();
    println!("{:?}", blockchain_data);
    assert_eq!(blockchain_data.version, 1);
    assert_eq!(blockchain_data.bridge, bridge_context.bridge.pubkey());
    assert_eq!(blockchain_data.blockchain_id, [0x45, 0x54, 0x48, 0x0]);
    assert_eq!(blockchain_data.validators, 0);
    assert_eq!(blockchain_data.contract_address, [1;32]);
}

#[tokio::test]
async fn add_validator_test() {
    let mut program_context = program_test().start_with_context().await;
    let bridge_context = BridgeContext::init(&mut program_context).await;
    bridge_context.add_blockchain(&mut program_context, String::from("ETH"), [1; 32]).await;
    let validator_pubkey = bridge_context.add_validator(&mut program_context, String::from("ETH"), [2; 32]).await;

    let validator_account = get_account(&mut program_context, &validator_pubkey).await;
    let validator_data: Validator = Validator::try_from_slice(&validator_account.data).unwrap();
    println!("{:?}", validator_data);
    assert_eq!(validator_data.version, 1);
    assert_eq!(validator_data.blockchain_id, [0x45, 0x54, 0x48, 0x0]);
    assert_eq!(validator_data.index, 0);
    assert_eq!(validator_data.pub_key, [2;32]);
}

#[tokio::test]
async fn add_signature_test() {
    let mut program_context = program_test().start_with_context().await;
    let bridge_context = BridgeContext::init(&mut program_context).await;
    bridge_context.add_blockchain(&mut program_context, String::from("ETH"), [1; 32]).await;
    let validator_pubkey = bridge_context.add_validator(&mut program_context, String::from("ETH"), [2; 32]).await;

    let blockchain_pubkey =
        Pubkey::create_with_seed(&self.bridge_authority, format!("blockchain_{}", blockchain_id_str).as_str(), &id()).unwrap();

    let validator_account = get_account(&mut program_context, &validator_pubkey).await;
    let validator_data: Validator = Validator::try_from_slice(&validator_account.data).unwrap();
    println!("{:?}", validator_data);
    assert_eq!(validator_data.version, 1);
    assert_eq!(validator_data.blockchain_id, [0x45, 0x54, 0x48, 0x0]);
    assert_eq!(validator_data.index, 0);
    assert_eq!(validator_data.pub_key, [2;32]);
}
