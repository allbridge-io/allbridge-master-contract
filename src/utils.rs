use crate::solana_program::{pubkey::Pubkey,
                            program_error::ProgramError,
                            account_info::AccountInfo,
                            rent::Rent,
                            entrypoint::ProgramResult,
                            program::invoke_signed,
                            system_instruction,
};

pub fn validate_bridge_authority_and_get_bump_seed(
    program_id: &Pubkey,
    bridge_account: &Pubkey,
    authority_account: &Pubkey,
) -> Result<u8, ProgramError> {
    validate_authority_and_get_bump_seed(program_id, bridge_account.as_ref(), authority_account)
}

pub fn validate_user_address_authority_and_get_bump_seed(
    program_id: &Pubkey,
    user_address: [u8; 32],
    authority_account: &Pubkey,
) -> Result<u8, ProgramError> {
    validate_authority_and_get_bump_seed(program_id, user_address.as_ref(), authority_account)
}

pub fn check_and_get_blockchain_account_seed(
    program_id: &Pubkey,
    blockchain_id: [u8; 4],
    bridge_authority: &Pubkey,
    blockchain_account: &Pubkey,
) -> Result<String, ProgramError> {
    let seed = format!("blockchain_{}", chain_id_to_str(&blockchain_id)?);
    check_and_get_account_seed(program_id, seed, bridge_authority, blockchain_account)
}

pub fn check_and_get_validator_account_seed(
    program_id: &Pubkey,
    blockchain_id: [u8; 4],
    index: u64,
    bridge_authority: &Pubkey,
    validator_account: &Pubkey,
) -> Result<String, ProgramError> {
    let seed = format!("validator_{}_{}", chain_id_to_str(&blockchain_id)?, index);
    check_and_get_account_seed(program_id, seed, bridge_authority, validator_account)
}

pub fn check_and_get_lock_account_seed(
    program_id: &Pubkey,
    source: [u8; 4],
    lock_id: u64,
    revert: bool,
    bridge_authority: &Pubkey,
    lock_account: &Pubkey,
) -> Result<String, ProgramError> {
    let seed = format!("{}_{}_{}", (if revert {"revert"} else {"lock"}), chain_id_to_str(&source)?, lock_id);
    check_and_get_account_seed(program_id, seed, bridge_authority, lock_account)
}

pub fn check_and_get_signature_account_seed(
    program_id: &Pubkey,
    source: [u8; 4],
    lock_id: u64,
    validator_id: u64,
    revert: bool,
    bridge_authority: &Pubkey,
    signature_account: &Pubkey,
) -> Result<String, ProgramError> {
    let seed = format!("signature_{}_{}_{}_{}", (if revert {"revert"} else { "lock" }), chain_id_to_str(&source)?, lock_id, validator_id);
    check_and_get_account_seed(program_id, seed, bridge_authority, signature_account)

}

pub fn check_and_get_user_account_seed(
    program_id: &Pubkey,
    blockchain_id: [u8; 4],
    user_authority: &Pubkey,
    user_account: &Pubkey,
) -> Result<String, ProgramError> {
    let seed = format!("user_{}", chain_id_to_str(&blockchain_id)?);
    check_and_get_account_seed(program_id, seed, user_authority, user_account)

}

pub fn check_and_get_sent_lock_account_seed(
    program_id: &Pubkey,
    blockchain_id: [u8; 4],
    user_authority: &Pubkey,
    index: u64,
    sent_lock_account: &Pubkey,
) -> Result<String, ProgramError> {
    let seed = format!("sent_{}_{}", chain_id_to_str(&blockchain_id)?, index);
    check_and_get_account_seed(program_id, seed, user_authority, sent_lock_account)
}

pub fn check_and_get_received_lock_account_seed(
    program_id: &Pubkey,
    blockchain_id: [u8; 4],
    user_authority: &Pubkey,
    index: u64,
    received_lock_account: &Pubkey,
) -> Result<String, ProgramError> {
    let seed = format!("received_{}_{}", chain_id_to_str(&blockchain_id)?, index);
    check_and_get_account_seed(program_id, seed, user_authority, received_lock_account)
}

pub fn check_and_get_lock_tx_account_seed(
    program_id: &Pubkey,
    blockchain_id: [u8; 4],
    index: u64,
    tx_type: &str,
    user_authority: &Pubkey,
    sent_lock_account: &Pubkey,
) -> Result<String, ProgramError> {
    let seed = format!("{}_{}_{}", tx_type, chain_id_to_str(&blockchain_id)?, index);
    check_and_get_account_seed(program_id, seed, user_authority, sent_lock_account)
}

pub fn str_to_chain_id(str: &str) -> [u8; 4] {
    let str_len = str.len();
    let mut result = [0; 4];
    result[..str_len].copy_from_slice(str.as_bytes());
    result
}

pub fn chain_id_to_str(chain_id: &[u8; 4]) -> Result<&str, ProgramError> {
    std::str::from_utf8(chain_id)
        .map_err(|_| ProgramError::InvalidArgument)
        .map(|s| s.trim_matches(0 as char))
}


pub fn validate_authority_and_get_bump_seed(
    program_id: &Pubkey,
    seed: &[u8],
    authority_account: &Pubkey,
) -> Result<u8, ProgramError> {
    let signer_seeds = &[seed];
    let (expected_authority_account, bump_seed) =
        Pubkey::find_program_address(signer_seeds, program_id);
    if expected_authority_account != *authority_account {
        return Err(ProgramError::InvalidSeeds);
    }
    Ok(bump_seed)
}

pub fn check_and_get_account_seed(
    program_id: &Pubkey,
    seed: String,
    authority: &Pubkey,
    account: &Pubkey,
) -> Result<String, ProgramError> {
    let expected_account =
        Pubkey::create_with_seed(authority, seed.as_str(), program_id)?;
    if expected_account != *account {
        return Err(ProgramError::InvalidSeeds);
    }
    Ok(seed)
}

pub fn create_account_with_seed<'a>(
    payer_info: &AccountInfo<'a>,
    new_account: &AccountInfo<'a>,
    authority_info: &AccountInfo<'a>,
    seed: String,
    data_size: usize,
    rent: &Rent,
    program_id: &Pubkey,
    signer_seed: &[u8],
    bump_seed: u8,
) -> ProgramResult {
    invoke_signed(
        &system_instruction::create_account_with_seed(
            &payer_info.key,
            &new_account.key,
            &authority_info.key,
            seed.as_str(),
            rent.minimum_balance(data_size),
            data_size as u64,
            &program_id,
        ),
        &[
            payer_info.clone(),
            new_account.clone(),
            authority_info.clone(),
        ],
        &[&[
            signer_seed,
            &[bump_seed],
        ]],
    )
}
