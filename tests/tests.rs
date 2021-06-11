#![cfg(feature = "test-bpf")]

use borsh::{BorshDeserialize};
use solana_program::{pubkey::Pubkey, system_instruction};
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_sdk::account::Account;

use solbridge_master_contract::*;
use solbridge_master_contract::state::{Blockchain, Bridge, Lock, Validator, Signature, User, LockTx};

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

    pub async fn add_signature(&self, program_context: &mut ProgramTestContext,
                               signature: [u8; 65],
                               token_source: String,
                               token_source_address: [u8; 32],
                               source: String,
                               tx_id: [u8; 64],
                               lock_id: u64,
                               destination: String,
                               sender: [u8; 32],
                               recipient: [u8; 32],
                               amount: u64,
                               validator_index: u64) -> (Pubkey, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey) {

        let lock_pubkey =
            Pubkey::create_with_seed(&self.bridge_authority, format!("lock_{}_{}", source, lock_id).as_str(), &id()).unwrap();
        let lock_account = program_context
            .banks_client
            .get_account(lock_pubkey)
            .await
            .expect("account not found");
        let signature_index: u64 = match lock_account {
            Some(l) => Lock::try_from_slice(&l.data).unwrap().signatures,
            None => 0
        };

        let blockchain_pubkey =
            Pubkey::create_with_seed(&self.bridge_authority, format!("blockchain_{}", source).as_str(), &id()).unwrap();

        let signature_pubkey =
            Pubkey::create_with_seed(&self.bridge_authority, format!("signature_lock_{}_{}_{}", source, lock_id, signature_index).as_str(), &id()).unwrap();

        let validator_pubkey =
            Pubkey::create_with_seed(&self.bridge_authority, format!("validator_{}_{}", source, validator_index).as_str(), &id()).unwrap();

        let (sender_authority, _) =
            Pubkey::find_program_address(&[sender.as_ref()], &id());

        let (recipient_authority, _) =
            Pubkey::find_program_address(&[recipient.as_ref()], &id());

        let sender_user_pubkey =
            Pubkey::create_with_seed(&sender_authority, format!("user_{}", source).as_str(), &id()).unwrap();

        let sender_user_account = program_context
            .banks_client
            .get_account(sender_user_pubkey)
            .await
            .expect("account not found");
        let sent_index: u64 = match sender_user_account {
            Some(l) => User::try_from_slice(&l.data).unwrap().sent,
            None => 0
        };

        let recipient_user_pubkey =
            Pubkey::create_with_seed(&recipient_authority, format!("user_{}", destination).as_str(), &id()).unwrap();

        let recipient_user_account = program_context
            .banks_client
            .get_account(recipient_user_pubkey)
            .await
            .expect("account not found");
        let received_index: u64 = match recipient_user_account {
            Some(l) => User::try_from_slice(&l.data).unwrap().received,
            None => 0
        };

        let sent_lock_pubkey =
            Pubkey::create_with_seed(&sender_authority, format!("sent_{}_{}", source, sent_index).as_str(), &id()).unwrap();

        let received_lock_pubkey =
            Pubkey::create_with_seed(&recipient_authority, format!("received_{}_{}", destination, received_index).as_str(), &id()).unwrap();


        let mut transaction = Transaction::new_with_payer(
            &[
                instruction::add_signature(
                    &id(),
                    &self.bridge.pubkey(),
                    &blockchain_pubkey,
                    &validator_pubkey,
                    &lock_pubkey,
                    &signature_pubkey,
                    &self.bridge_authority,
                    &sender_user_pubkey,
                    &sender_authority,
                    &recipient_user_pubkey,
                    &recipient_authority,
                    &sent_lock_pubkey,
                    &received_lock_pubkey,
                    &program_context.payer.pubkey(),
                    signature,
                    token_source,
                    token_source_address,
                    source,
                    tx_id,
                    lock_id,
                    destination,
                    sender,
                    recipient,
                    amount,
                    false
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

        (lock_pubkey, signature_pubkey, sender_user_pubkey, recipient_user_pubkey, sent_lock_pubkey, received_lock_pubkey)
    }
}

#[tokio::test]
async fn init_bridge_test() {
    let mut program_context = program_test().start_with_context().await;
    let bridge_context = BridgeContext::init(&mut program_context).await;

    let bridge_account = get_account(&mut program_context, &bridge_context.bridge.pubkey()).await;
    let bridge_data: Bridge = Bridge::try_from_slice(&bridge_account.data).unwrap();
    println!("{:?}", bridge_data);
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
    let blockchain_pubkey = bridge_context.add_blockchain(&mut program_context, String::from("ETH"), [1; 32]).await;
    let validator_pubkey = bridge_context.add_validator(&mut program_context, String::from("ETH"), [2; 32]).await;

    let validator_account = get_account(&mut program_context, &validator_pubkey).await;
    let validator_data: Validator = Validator::try_from_slice(&validator_account.data).unwrap();
    println!("{:?}", validator_data);
    assert_eq!(validator_data.version, 1);
    assert_eq!(validator_data.blockchain_id, [0x45, 0x54, 0x48, 0x0]);
    assert_eq!(validator_data.index, 0);
    assert_eq!(validator_data.pub_key, [2;32]);

    let blockchain_account = get_account(&mut program_context, &blockchain_pubkey).await;
    let blockchain_data: Blockchain = Blockchain::try_from_slice(&blockchain_account.data).unwrap();
    assert_eq!(blockchain_data.validators, 1);
}

#[tokio::test]
async fn add_signature_test() {
    let mut program_context = program_test().start_with_context().await;
    let bridge_context = BridgeContext::init(&mut program_context).await;
    bridge_context.add_blockchain(&mut program_context, String::from("ETH"), [1; 32]).await;
    let validator_pubkey = bridge_context.add_validator(&mut program_context, String::from("ETH"), [2; 32]).await;

    let (
        lock_pubkey,
        signature_pubkey,
        sender_pubkey,
        recipient_pubkey,
        sent_lock_pubkey,
        received_lock_pubkey
    ) = bridge_context.add_signature(
        &mut program_context,
        [7; 65],
        String::from("ETH"),
        [3; 32],
        String::from("ETH"),
        [9; 64],
        1,
        String::from("BSC"),
        [2; 32],
        [4; 32],
        10000,
        0
    ).await;


    let lock_account = get_account(&mut program_context, &lock_pubkey).await;
    let lock_data: Lock = Lock::try_from_slice(&lock_account.data).unwrap();
    println!("{:?}", lock_data);
    assert_eq!(lock_data.version, 1);
    assert_eq!(lock_data.index, 0);
    assert_eq!(lock_data.lock_id, 1);
    assert_eq!(lock_data.bridge, bridge_context.bridge.pubkey());
    assert_eq!(lock_data.token_source_address, [3; 32]);
    assert_eq!(lock_data.token_source, [0x45, 0x54, 0x48, 0x0]);
    assert_eq!(lock_data.source, [0x45, 0x54, 0x48, 0x0]);
    assert_eq!(lock_data.recipient, [4; 32]);
    assert_eq!(lock_data.destination, [0x42, 0x53, 0x43, 0x0]);
    assert_eq!(lock_data.amount, 10000);
    assert_eq!(lock_data.signatures, 1);

    let signature_account = get_account(&mut program_context, &signature_pubkey).await;
    let signature_data: Signature = Signature::try_from_slice(&signature_account.data).unwrap();
    println!("{:?}", signature_data);
    assert_eq!(signature_data.version, 1);
    assert_eq!(signature_data.source, [0x45, 0x54, 0x48, 0x0]);
    assert_eq!(signature_data.lock_id, 1);
    assert_eq!(signature_data.bridge, bridge_context.bridge.pubkey());
    assert_eq!(signature_data.signature, [7; 65]);
    assert_eq!(signature_data.validator, validator_pubkey);
    assert_eq!(signature_data.validator_index, 0);

    let sender_account = get_account(&mut program_context, &sender_pubkey).await;
    let sender_data: User = User::try_from_slice(&sender_account.data).unwrap();
    println!("{:?}", sender_data);
    assert_eq!(sender_data.version, 1);
    assert_eq!(sender_data.blockchain_id, [0x45, 0x54, 0x48, 0x0]);
    assert_eq!(sender_data.address, [2; 32]);
    assert_eq!(sender_data.sent, 1);
    assert_eq!(sender_data.received, 0);

    let recipient_account = get_account(&mut program_context, &recipient_pubkey).await;
    let recipient_data: User = User::try_from_slice(&recipient_account.data).unwrap();
    println!("{:?}", recipient_data);
    assert_eq!(recipient_data.version, 1);
    assert_eq!(recipient_data.blockchain_id, [0x42, 0x53, 0x43, 0x0]);
    assert_eq!(recipient_data.address, [4; 32]);
    assert_eq!(recipient_data.sent, 0);
    assert_eq!(recipient_data.received, 1);

    let sent_lock_account = get_account(&mut program_context, &sent_lock_pubkey).await;
    let sent_lock_data: LockTx = LockTx::try_from_slice(&sent_lock_account.data).unwrap();
    println!("{:?}", sent_lock_data);
    assert_eq!(sent_lock_data.version, 1);
    assert_eq!(sent_lock_data.tx_id, [9; 64]);

    let received_lock_account = get_account(&mut program_context, &received_lock_pubkey).await;
    let received_lock_data: LockTx = LockTx::try_from_slice(&received_lock_account.data).unwrap();
    println!("{:?}", received_lock_data);
    assert_eq!(received_lock_data.version, 1);
    assert_eq!(received_lock_data.tx_id, [9; 64]);


    let (
        lock_pubkey,
        signature_pubkey,
        sender_pubkey,
        recipient_pubkey,
        sent_lock_pubkey,
        received_lock_pubkey
    ) = bridge_context.add_signature(
        &mut program_context,
        [17; 65],
        String::from("ETH"),
        [3; 32],
        String::from("ETH"),
        [9; 64],
        1,
        String::from("BSC"),
        [2; 32],
        [4; 32],
        10000,
        0
    ).await;

    let lock_account = get_account(&mut program_context, &lock_pubkey).await;
    let lock_data: Lock = Lock::try_from_slice(&lock_account.data).unwrap();
    println!("{:?}", lock_data);
    assert_eq!(lock_data.version, 1);
    assert_eq!(lock_data.index, 0);
    assert_eq!(lock_data.lock_id, 1);
    assert_eq!(lock_data.bridge, bridge_context.bridge.pubkey());
    assert_eq!(lock_data.token_source_address, [3; 32]);
    assert_eq!(lock_data.token_source, [0x45, 0x54, 0x48, 0x0]);
    assert_eq!(lock_data.source, [0x45, 0x54, 0x48, 0x0]);
    assert_eq!(lock_data.recipient, [4; 32]);
    assert_eq!(lock_data.destination, [0x42, 0x53, 0x43, 0x0]);
    assert_eq!(lock_data.amount, 10000);
    assert_eq!(lock_data.signatures, 2);

    let signature_account = get_account(&mut program_context, &signature_pubkey).await;
    let signature_data: Signature = Signature::try_from_slice(&signature_account.data).unwrap();
    println!("{:?}", signature_data);
    assert_eq!(signature_data.version, 1);
    assert_eq!(signature_data.source, [0x45, 0x54, 0x48, 0x0]);
    assert_eq!(signature_data.lock_id, 1);
    assert_eq!(signature_data.bridge, bridge_context.bridge.pubkey());
    assert_eq!(signature_data.signature, [17; 65]);
    assert_eq!(signature_data.validator, validator_pubkey);
    assert_eq!(signature_data.validator_index, 0);

    let sender_account = get_account(&mut program_context, &sender_pubkey).await;
    let sender_data: User = User::try_from_slice(&sender_account.data).unwrap();
    println!("{:?}", sender_data);
    assert_eq!(sender_data.version, 1);
    assert_eq!(sender_data.blockchain_id, [0x45, 0x54, 0x48, 0x0]);
    assert_eq!(sender_data.address, [2; 32]);
    assert_eq!(sender_data.sent, 1);
    assert_eq!(sender_data.received, 0);

    let recipient_account = get_account(&mut program_context, &recipient_pubkey).await;
    let recipient_data: User = User::try_from_slice(&recipient_account.data).unwrap();
    println!("{:?}", recipient_data);
    assert_eq!(recipient_data.version, 1);
    assert_eq!(recipient_data.blockchain_id, [0x42, 0x53, 0x43, 0x0]);
    assert_eq!(recipient_data.address, [4; 32]);
    assert_eq!(recipient_data.sent, 0);
    assert_eq!(recipient_data.received, 1);

    assert_eq!(program_context
                   .banks_client
                   .get_account(sent_lock_pubkey)
                   .await
                   .expect("account not found"), None);

    assert_eq!(program_context
                   .banks_client
                   .get_account(received_lock_pubkey)
                   .await
                   .expect("account not found"), None);

}
