//! Program state processor

use crate::{
    error::NftError,
    instruction::{NftInstruction},
};
use borsh::{BorshDeserialize};//, BorshSerialize};
use solana_program::{
    program::{invoke, invoke_signed},
    account_info::next_account_info,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    // sysvar::{rent::Rent, Sysvar},
};

use spl_token;
use spl_associated_token_account;
use metaplex_token_metadata;
use solana_program::program_pack::Pack;

/// Program state handler.
pub struct Processor {}
impl Processor {
    /// Seed
    pub const RELAYER_SEED: &'static str = "relayer";
    pub const TOKEN_DATA_SEED: &'static str = "token_data";

    /// Stake Master Edition token
    pub fn process_stake_master_edition(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let mint_info = next_account_info(account_info_iter)?;
        let user_associated_token_account = next_account_info(account_info_iter)?;
        let pda_relayer_account = next_account_info(account_info_iter)?;
        let update_authority = next_account_info(account_info_iter)?;
        let metadata_info = next_account_info(account_info_iter)?;
        let master_edition_info = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        // validate
        let metadata = metaplex_token_metadata::state::Metadata::from_account_info(metadata_info)?;
        if !update_authority.is_signer || metadata.update_authority == *update_authority.key {
            return Err(ProgramError::MissingRequiredSignature);
        }
        if metadata.mint != *mint_info.key {
            return Err(NftError::MetadataMismatch.into());
        }

        let metaplex_pid = metaplex_token_metadata::id();
        let master_ed_seed_keys = &[
            &metaplex_token_metadata::state::PREFIX.as_bytes(),
            metaplex_pid.as_ref(),
            &mint_info.key.as_ref(),
            &metaplex_token_metadata::state::EDITION.as_bytes(),
        ];
        let _ = assert_derivation(program_id, master_edition_info, master_ed_seed_keys)?;

        let master_edition = &metaplex_token_metadata::state::MasterEditionV2::from_account_info(master_edition_info)?;
        if master_edition.key != metaplex_token_metadata::state::Key::MasterEditionV2 || master_edition.supply > 0 {
            return Err(NftError::WrongEdition.into());
        }
        let mut found = false;
        if let Some(creators) = metadata.data.creators {
            for i in 0..creators.len() {
                if creators[i].address == *pda_relayer_account.key {
                    found = true;
                }
            }
            if !found {
                return Err(NftError::SatelliteMustListAmongCreators.into());
            }
        }

        let &derived_associated_token_account = &spl_associated_token_account::get_associated_token_address(update_authority.key, mint_info.key);
        if derived_associated_token_account != *user_associated_token_account.key {
            return Err(ProgramError::InvalidSeeds);
        }

        let token_account = &spl_token::state::Account::unpack_from_slice(&user_associated_token_account.data.borrow())?;
        if token_account.amount < 1 {
            return Err(NftError::NotOwned.into());
        }

        let metadata_seed_keys = &[
            Self::RELAYER_SEED.as_bytes(),
            program_id.as_ref(),
        ];
        let _ = assert_derivation(program_id, pda_relayer_account, metadata_seed_keys)?;

        // stake
        invoke(
            &spl_token::instruction::set_authority(
                &spl_token::id(),
                &user_associated_token_account.key,
                Some(pda_relayer_account.key),
                spl_token::instruction::AuthorityType::AccountOwner,
                &update_authority.key,
                &[],
            )?,
            &[
                user_associated_token_account.clone(),
                pda_relayer_account.clone(),
                update_authority.clone(),
                token_program.clone()
            ],
        )?;
        Ok(())
    }

    pub fn process_unstake_master_edition(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let mint_info = next_account_info(account_info_iter)?;
        let user_associated_token_account = next_account_info(account_info_iter)?;
        let pda_relayer_account = next_account_info(account_info_iter)?;
        let update_authority = next_account_info(account_info_iter)?;
        let metadata_info = next_account_info(account_info_iter)?;
        let master_edition_info = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        let metadata = metaplex_token_metadata::state::Metadata::from_account_info(metadata_info)?;
        
        if !update_authority.is_signer || metadata.update_authority == *update_authority.key {
            return Err(ProgramError::MissingRequiredSignature);
        }
        if metadata.mint != *mint_info.key {
            return Err(NftError::MetadataMismatch.into());
        }

        let metaplex_pid = metaplex_token_metadata::id();
        let master_ed_seed_keys = &[
            &metaplex_token_metadata::state::PREFIX.as_bytes(),
            metaplex_pid.as_ref(),
            &mint_info.key.as_ref(),
            &metaplex_token_metadata::state::EDITION.as_bytes(),
        ];
        let _ = assert_derivation(program_id, master_edition_info, master_ed_seed_keys)?;

        let master_edition = &metaplex_token_metadata::state::MasterEditionV2::from_account_info(master_edition_info)?;
        if master_edition.key != metaplex_token_metadata::state::Key::MasterEditionV2 {
            return Err(NftError::WrongEdition.into());
        }
        if let Some(max_supply) = master_edition.max_supply {
            if master_edition.supply != max_supply {
                return Err(NftError::OngoingSales.into());
            }
        }

        let &derived_associated_token_account = &spl_associated_token_account::get_associated_token_address(update_authority.key, mint_info.key);
        if derived_associated_token_account != *user_associated_token_account.key {
            return Err(ProgramError::InvalidSeeds);
        }

        let token_account = &spl_token::state::Account::unpack_from_slice(&user_associated_token_account.data.borrow())?;
        if token_account.amount != 1 {
            return Err(NftError::NotOwned.into());
        }

        let metadata_seed_keys = &[
            Self::RELAYER_SEED.as_bytes(),
            program_id.as_ref(),
        ];
        let bump_seed = assert_derivation(program_id, pda_relayer_account, metadata_seed_keys)?;
        let signature = &[
            Self::RELAYER_SEED.as_bytes(),
            program_id.as_ref(),
            &[bump_seed],
        ];

        // stake
        invoke_signed(
            &spl_token::instruction::set_authority(
                &spl_token::id(),
                &user_associated_token_account.key,
                Some(update_authority.key),
                spl_token::instruction::AuthorityType::AccountOwner,
                &pda_relayer_account.key,
                &[],
            )?,
            &[
                user_associated_token_account.clone(),
                update_authority.clone(),
                pda_relayer_account.clone(),
                token_program.clone()
            ],
            &[signature]
        )?;
        Ok(())
    }

    pub fn process_mint_new_edition(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        edition: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let edition_metadata_info = next_account_info(account_info_iter)?;
        let edition_info = next_account_info(account_info_iter)?;
        let master_edition_info = next_account_info(account_info_iter)?;
        let edition_mint_info = next_account_info(account_info_iter)?;
        let edition_update_authority = next_account_info(account_info_iter)?;
        let pda_relayer_account = next_account_info(account_info_iter)?;
        let creator_associated_token_account = next_account_info(account_info_iter)?;
        let master_metadata_info = next_account_info(account_info_iter)?;
        let master_mint_info = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;
        let metaplex_program = next_account_info(account_info_iter)?;
        let rent_account_info = next_account_info(account_info_iter)?;

        // INSERT HERE PAYMENT LOGIC

        let metadata_seed_keys = &[
            Self::RELAYER_SEED.as_bytes(),
            program_id.as_ref(),
        ];
        let bump_seed = assert_derivation(program_id, pda_relayer_account, metadata_seed_keys)?;
        let signature = &[
            Self::RELAYER_SEED.as_bytes(),
            program_id.as_ref(),
            &[bump_seed],
        ];

        invoke_signed(
            &metaplex_token_metadata::instruction::mint_new_edition_from_master_edition_via_token(
                metaplex_token_metadata::id(),
                *edition_metadata_info.key,
                *edition_info.key,
                *master_edition_info.key,
                *edition_mint_info.key,
                *edition_update_authority.key,
                *edition_update_authority.key,
                *pda_relayer_account.key,
                *creator_associated_token_account.key,
                *edition_update_authority.key,
                *master_metadata_info.key,
                *master_mint_info.key,
                edition
            ),
            &[
                edition_metadata_info.clone(),
                edition_info.clone(),
                master_edition_info.clone(),
                edition_mint_info.clone(),
                edition_update_authority.clone(),
                edition_update_authority.clone(),
                pda_relayer_account.clone(),
                creator_associated_token_account.clone(),
                edition_update_authority.clone(),
                master_metadata_info.clone(),
                master_mint_info.clone(),
                token_program.clone(),
                system_program.clone(),
                metaplex_program.clone(),
                rent_account_info.clone()
            ],
            &[signature]
        )?;
        Ok(())
    }

    /// Processes an instruction
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        input: &[u8],
    ) -> ProgramResult {
        let instruction =
            NftInstruction::try_from_slice(input).or(Err(ProgramError::InvalidInstructionData))?;
        match instruction {
            NftInstruction::StakeMasterEdition => {
                msg!("Instruction: StakeMasterEdition");
                Self::process_stake_master_edition(program_id, accounts)
            }
            NftInstruction::UnstakeMasterEdition => {
                msg!("Instruction: UnstakeMasterEdition");
                Self::process_unstake_master_edition(program_id, accounts)
            }
            NftInstruction::MintNewEdition(edition) => {
                msg!("Instruction: UnstakeMasterEdition");
                Self::process_mint_new_edition(program_id, accounts, edition)
            }
        }
    }
}

// fn transfer_lamports(
//     signer: &AccountInfo,
//     mut lamports: std::cell::RefMut<&mut u64>,
// ) -> Result<(), ProgramError> {
//     let mut thanks_for_cleaning_garbage = signer.try_borrow_mut_lamports()?;
//     let value = (**thanks_for_cleaning_garbage)
//         .checked_add(**lamports)
//         .ok_or(NftError::Overflow)?;
//     **thanks_for_cleaning_garbage = value;
//     **lamports = 0;
//     Ok(())
// }

// fn validate_program(program_id: &Pubkey, account: &AccountInfo) -> ProgramResult {
//     if program_id != account.owner {
//         return Err(ProgramError::IncorrectProgramId);
//     }

//     Ok(())
// }

pub fn assert_derivation(
    program_id: &Pubkey,
    account: &AccountInfo,
    path: &[&[u8]],
) -> Result<u8, ProgramError> {
    let (key, bump) = Pubkey::find_program_address(&path, program_id);
    if key != *account.key {
        return Err(ProgramError::InvalidSeeds);
    }
    Ok(bump)
}
