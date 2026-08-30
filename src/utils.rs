use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};
use solana_system_interface::{
    instruction::{create_account, transfer},
    program as system_program,
};

use crate::state::Config;

use crate::constants::{VRF_MAX_RETRIES, VRF_RETRY_DELAY_SECONDS};
use crate::error::RPSProgramError::{
    AccountAlreadyInitialized, ArithmeticError, InvalidAuth, InvalidConfig, InvalidDerivedAccount,
    NotSignerAuth, VrfFailureTooEarly, VrfRetriesNotExhausted, VrfRetryLimitReached,
    VrfRetryTooEarly,
};

pub struct Utils;
impl Utils {
    pub fn check_admin(
        admin: &AccountInfo,
        config_account: &AccountInfo,
        program_id: &Pubkey,
    ) -> ProgramResult {
        if !admin.is_signer {
            return Err(NotSignerAuth.into());
        }

        if config_account.owner != program_id {
            return Err(InvalidConfig.into());
        }

        let config: Config = Config::try_from_slice(&config_account.data.borrow())?;

        if config.is_init != 1 {
            return Err(InvalidConfig.into());
        }

        let admin_address_1: Pubkey = Pubkey::new_from_array(config.admin_1);
        let admin_address_2: Pubkey = Pubkey::new_from_array(config.admin_2);
        let admin_address_3: Pubkey = Pubkey::new_from_array(config.admin_3);
        let admin_address_4: Pubkey = Pubkey::new_from_array(config.admin_4);
        let admin_address_5: Pubkey = Pubkey::new_from_array(config.admin_5);

        let valid_authorities: [Pubkey; 5] = [
            admin_address_1,
            admin_address_2,
            admin_address_3,
            admin_address_4,
            admin_address_5,
        ];

        if !valid_authorities.contains(admin.key) {
            return Err(InvalidAuth.into());
        }

        Ok(())
    }

    pub fn create_pda<'a>(
        payer: &AccountInfo<'a>,
        pda: &AccountInfo<'a>,
        system_program_account: &AccountInfo<'a>,
        seeds: &[&[u8]],
        space: u64,
        program_id: &Pubkey,
    ) -> Result<u8, ProgramError> {
        if system_program_account.key != &system_program::ID {
            return Err(InvalidDerivedAccount.into());
        }

        let rent: Rent = Rent::get()?;
        let rent_amount: u64 = rent.minimum_balance(space.try_into().unwrap());

        let (pda_address, bump) = Pubkey::find_program_address(seeds, program_id);
        if pda.key != &pda_address {
            return Err(InvalidDerivedAccount.into());
        }

        if pda.owner != &system_program::ID || pda.executable || !pda.data_is_empty() {
            return Err(AccountAlreadyInitialized.into());
        }

        let create_ix = create_account(payer.key, &pda_address, rent_amount, space, program_id);
        let bump_seed = [bump];
        let mut signer_seeds: Vec<&[u8]> = seeds.to_vec();
        signer_seeds.push(&bump_seed);

        let prefunded_lamports = **pda.try_borrow_lamports()?;
        if prefunded_lamports != 0 {
            invoke_signed(
                &transfer(pda.key, payer.key, prefunded_lamports),
                &[pda.clone(), payer.clone(), system_program_account.clone()],
                &[signer_seeds.as_slice()],
            )?;
        }

        invoke_signed(
            &create_ix,
            &[payer.clone(), pda.clone(), system_program_account.clone()],
            &[signer_seeds.as_slice()],
        )?;

        Ok(bump)
    }

    pub fn next_vrf_retry_count(current: u8) -> Result<u8, ProgramError> {
        if current >= VRF_MAX_RETRIES {
            return Err(VrfRetryLimitReached.into());
        }
        current.checked_add(1).ok_or(ArithmeticError.into())
    }

    pub fn require_vrf_retry_delay(last_request_at: i64, now: i64) -> ProgramResult {
        let retry_at = last_request_at
            .checked_add(VRF_RETRY_DELAY_SECONDS)
            .ok_or(ArithmeticError)?;
        if now < retry_at {
            return Err(VrfRetryTooEarly.into());
        }
        Ok(())
    }

    pub fn require_vrf_failure_delay(
        retry_count: u8,
        last_request_at: i64,
        now: i64,
    ) -> ProgramResult {
        if retry_count != VRF_MAX_RETRIES {
            return Err(VrfRetriesNotExhausted.into());
        }
        let failure_at = last_request_at
            .checked_add(VRF_RETRY_DELAY_SECONDS)
            .ok_or(ArithmeticError)?;
        if now < failure_at {
            return Err(VrfFailureTooEarly.into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vrf_retry_sequence_enforces_two_spaced_retries_then_failure() {
        assert_eq!(Utils::next_vrf_retry_count(0).unwrap(), 1);
        assert_eq!(Utils::next_vrf_retry_count(1).unwrap(), 2);
        assert_eq!(
            Utils::next_vrf_retry_count(2).unwrap_err(),
            ProgramError::from(VrfRetryLimitReached)
        );

        assert_eq!(
            Utils::require_vrf_retry_delay(1_000, 1_119).unwrap_err(),
            ProgramError::from(VrfRetryTooEarly)
        );
        assert!(Utils::require_vrf_retry_delay(1_000, 1_120).is_ok());
        assert_eq!(
            Utils::require_vrf_failure_delay(1, 1_000, 1_120).unwrap_err(),
            ProgramError::from(VrfRetriesNotExhausted)
        );
        assert_eq!(
            Utils::require_vrf_failure_delay(2, 1_000, 1_119).unwrap_err(),
            ProgramError::from(VrfFailureTooEarly)
        );
        assert!(Utils::require_vrf_failure_delay(2, 1_000, 1_120).is_ok());
    }
}
