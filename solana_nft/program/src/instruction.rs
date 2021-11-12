//! Instruction types

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum NftInstruction {
    /// Stakes a Master Edition in order to let other users purchase editions of the NFT.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[]` The master edition Mint
    ///   1. `[writable]` The master edition user token account
    ///   2. `[]` The Relayer PDA
    ///   3. `[signer]` The Update Authority
    ///   4. `[]` Metadata account (pda of ['metadata', program id, mint id])
    ///   5. `[]` Master Record Edition V2 (pda of ['metadata', program id, master metadata mint id, 'edition'])
    ///   6. `[]` Token program
    StakeMasterEdition,

    /// Unstakes a Master Edition from the marketplace.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[]` The master edition Mint
    ///   1. `[writable]` The master edition user token account
    ///   2. `[]` The Relayer PDA
    ///   3. `[signer]` The Update Authority
    ///   4. `[]` Metadata account (pda of ['metadata', program id, mint id])
    ///   5. `[]` Master Record Edition V2 (pda of ['metadata', program id, master metadata mint id, 'edition'])
    ///   6. `[]` Token program
    UnstakeMasterEdition,

    /// Mint a new Edition from a Master Edition on the marketplace.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` New Metadata key (pda of ['metadata', program id, mint id])
    ///   1. `[writable]` New Edition (pda of ['metadata', program id, mint id, 'edition'])
    ///   2. `[writable]` Master Record Edition V2 (pda of ['metadata', program id, master metadata mint id, 'edition'])
    ///   3. `[writable]` Mint of new token
    ///   4. `[signer]` Mint authority of new mint that is also Update authority info for new metadata
    ///   5. `[signer]` owner of token account containing master token (The Relayer PDA)
    ///   6. `[]` token account containing token from master metadata mint
    ///   7. `[]` Master record metadata account
    ///   8. `[]` Master record mint account
    ///   9. `[]` Token program
    ///   10. `[]` System program
    ///   11. `[]` Metaplex program
    ///   12. `[]` Rent info
    MintNewEdition(
        // Edition number for this new mint
        u64,
    ),
}

/// Create `StakeMasterEdition` instruction
pub fn stake_master_edition(
    program_id: &Pubkey,
    mint_info: &Pubkey,
    user_associated_token_account: &Pubkey,
    pda_relayer_account: &Pubkey,
    update_authority: &Pubkey,
    metadata_info: &Pubkey,
    master_edition_info: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let init_data = NftInstruction::StakeMasterEdition;
    let data = init_data
        .try_to_vec()
        .or(Err(ProgramError::InvalidArgument))?;
    let accounts = vec![
        AccountMeta::new_readonly(*mint_info, false),
        AccountMeta::new(*user_associated_token_account, false),
        AccountMeta::new_readonly(*pda_relayer_account, false),
        AccountMeta::new_readonly(*update_authority, true),
        AccountMeta::new_readonly(*metadata_info, false),
        AccountMeta::new_readonly(*master_edition_info, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Create `UnstakeMasterEdition` instruction
pub fn unstake_master_edition(
    program_id: &Pubkey,
    mint_info: &Pubkey,
    user_associated_token_account: &Pubkey,
    pda_relayer_account: &Pubkey,
    update_authority: &Pubkey,
    metadata_info: &Pubkey,
    master_edition_info: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let init_data = NftInstruction::StakeMasterEdition;
    let data = init_data
        .try_to_vec()
        .or(Err(ProgramError::InvalidArgument))?;
    let accounts = vec![
        AccountMeta::new_readonly(*mint_info, false),
        AccountMeta::new(*user_associated_token_account, false),
        AccountMeta::new_readonly(*pda_relayer_account, false),
        AccountMeta::new_readonly(*update_authority, true),
        AccountMeta::new_readonly(*metadata_info, false),
        AccountMeta::new_readonly(*master_edition_info, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}