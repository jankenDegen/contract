use crate::error::RPSProgramError::InvalidVrfAccount;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
    pubkey,
    pubkey::Pubkey,
    sysvar::slot_hashes,
};
use solana_system_interface::program as system_program;

const VRF_PROGRAM_ID: Pubkey = pubkey!("Vrf1RNUjXmQGjmQrQLvJHs9SNkvDJEsRVFPkfSQUwGz");
const DEFAULT_QUEUE: Pubkey = pubkey!("Cuj97ggrhhidhbu39TijNVqE74xvKJ69gDervRUXAxGh");
const DEFAULT_EPHEMERAL_QUEUE: Pubkey = pubkey!("5hBR571xnXppuCPveTrctfTU7tJLSN94nq7kv7FRK5Tc");
const IDENTITY: &[u8] = b"identity";
const REQUEST_SCOPED_RANDOMNESS_TAG: u8 = 10;

pub struct MagicBlockVrf;

impl MagicBlockVrf {
    pub fn request_randomness<'a>(
        callback_tag: u8,
        callback_args: &[u8],
        payer: &AccountInfo<'a>,
        request_identity: &AccountInfo<'a>,
        oracle_queue: &AccountInfo<'a>,
        system_program_account: &AccountInfo<'a>,
        slot_hashes_account: &AccountInfo<'a>,
        vrf_program: &AccountInfo<'a>,
        callback_program_id: &Pubkey,
        game_account: &AccountInfo<'a>,
        caller_seed: [u8; 32],
    ) -> ProgramResult {
        Self::request_randomness_with_callback_accounts(
            callback_tag,
            callback_args,
            payer,
            request_identity,
            oracle_queue,
            system_program_account,
            slot_hashes_account,
            vrf_program,
            callback_program_id,
            &[(*game_account.key, false, true)],
            caller_seed,
        )
    }

    pub fn request_randomness_with_callback_accounts<'a>(
        callback_tag: u8,
        callback_args: &[u8],
        payer: &AccountInfo<'a>,
        request_identity: &AccountInfo<'a>,
        oracle_queue: &AccountInfo<'a>,
        system_program_account: &AccountInfo<'a>,
        slot_hashes_account: &AccountInfo<'a>,
        vrf_program: &AccountInfo<'a>,
        callback_program_id: &Pubkey,
        callback_accounts: &[(Pubkey, bool, bool)],
        caller_seed: [u8; 32],
    ) -> ProgramResult {
        if vrf_program.key != &VRF_PROGRAM_ID {
            return Err(InvalidVrfAccount.into());
        }
        if oracle_queue.key != &DEFAULT_EPHEMERAL_QUEUE && oracle_queue.key != &DEFAULT_QUEUE {
            return Err(InvalidVrfAccount.into());
        }
        if system_program_account.key != &system_program::ID {
            return Err(InvalidVrfAccount.into());
        }
        if slot_hashes_account.key != &slot_hashes::ID {
            return Err(InvalidVrfAccount.into());
        }

        let (expected_identity, identity_bump) =
            Pubkey::find_program_address(&[IDENTITY], callback_program_id);
        if request_identity.key != &expected_identity {
            return Err(InvalidVrfAccount.into());
        }

        let request_ix = Self::request_scoped_randomness_instruction(
            *payer.key,
            *oracle_queue.key,
            *callback_program_id,
            caller_seed,
            callback_tag,
            callback_args,
            callback_accounts,
        );

        let identity_bump_seed = [identity_bump];
        invoke_signed(
            &request_ix,
            &[
                payer.clone(),
                request_identity.clone(),
                oracle_queue.clone(),
                system_program_account.clone(),
                slot_hashes_account.clone(),
                vrf_program.clone(),
            ],
            &[&[IDENTITY, &identity_bump_seed]],
        )
    }

    pub fn validate_callback_identity(
        vrf_program_identity: &AccountInfo,
        callback_program_id: &Pubkey,
    ) -> ProgramResult {
        if !vrf_program_identity.is_signer {
            return Err(InvalidVrfAccount.into());
        }
        if vrf_program_identity.key != &Self::scoped_vrf_identity(callback_program_id) {
            return Err(InvalidVrfAccount.into());
        }

        Ok(())
    }

    fn scoped_vrf_identity(callback_program_id: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[IDENTITY, callback_program_id.as_ref()], &VRF_PROGRAM_ID).0
    }

    fn request_scoped_randomness_instruction(
        payer: Pubkey,
        oracle_queue: Pubkey,
        callback_program_id: Pubkey,
        caller_seed: [u8; 32],
        callback_tag: u8,
        callback_args: &[u8],
        callback_accounts: &[(Pubkey, bool, bool)],
    ) -> Instruction {
        let program_identity = Pubkey::find_program_address(&[IDENTITY], &callback_program_id).0;
        let mut data = vec![REQUEST_SCOPED_RANDOMNESS_TAG, 0, 0, 0, 0, 0, 0, 0];

        data.extend_from_slice(&caller_seed);
        data.extend_from_slice(callback_program_id.as_ref());
        Self::extend_vec(&mut data, &[callback_tag]);

        data.extend_from_slice(&(callback_accounts.len() as u32).to_le_bytes());
        for (account, is_signer, is_writable) in callback_accounts {
            data.extend_from_slice(account.as_ref());
            data.push(u8::from(*is_signer));
            data.push(u8::from(*is_writable));
        }

        Self::extend_vec(&mut data, callback_args);

        Instruction {
            program_id: VRF_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(program_identity, true),
                AccountMeta::new(oracle_queue, false),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(slot_hashes::ID, false),
            ],
            data,
        }
    }

    fn extend_vec(data: &mut Vec<u8>, bytes: &[u8]) {
        data.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_serialization_uses_one_byte_discriminator_and_authenticated_callback_args() {
        let payer = Pubkey::new_unique();
        let queue = Pubkey::new_unique();
        let callback_program_id = Pubkey::new_unique();
        let callback_account = Pubkey::new_unique();
        let caller_seed = [41u8; 32];
        let mut callback_args = vec![];
        callback_args.extend_from_slice(&17u64.to_le_bytes());
        callback_args.extend_from_slice(&[43u8; 32]);

        let instruction = MagicBlockVrf::request_scoped_randomness_instruction(
            payer,
            queue,
            callback_program_id,
            caller_seed,
            92,
            &callback_args,
            &[(callback_account, false, true)],
        );

        let discriminator_length_offset = 8 + 32 + 32;
        let discriminator_offset = discriminator_length_offset + 4;
        assert_eq!(
            u32::from_le_bytes(
                instruction.data[discriminator_length_offset..discriminator_offset]
                    .try_into()
                    .unwrap()
            ) as usize,
            1
        );
        assert_eq!(instruction.data[discriminator_offset], 92);

        let account_count_offset = discriminator_offset + 1;
        let account_metas_offset = account_count_offset + 4;
        let account_count = u32::from_le_bytes(
            instruction.data[account_count_offset..account_metas_offset]
                .try_into()
                .unwrap(),
        ) as usize;
        let callback_args_length_offset = account_metas_offset + account_count * 34;
        let callback_args_offset = callback_args_length_offset + 4;
        assert_eq!(
            u32::from_le_bytes(
                instruction.data[callback_args_length_offset..callback_args_offset]
                    .try_into()
                    .unwrap(),
            ) as usize,
            callback_args.len()
        );
        assert_eq!(
            &instruction.data[callback_args_offset..callback_args_offset + callback_args.len()],
            callback_args.as_slice()
        );
    }
}
