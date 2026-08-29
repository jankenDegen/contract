use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};
use solana_system_interface::{instruction::create_account, program as system_program};

use crate::state::Config;

use crate::error::RPSProgramError::{
    AccountAlreadyInitialized, InvalidAuth, InvalidConfig, InvalidDerivedAccount, NotSignerAuth,
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

        if **pda.try_borrow_lamports()? != 0 {
            return Err(AccountAlreadyInitialized.into());
        }

        let create_ix = create_account(payer.key, &pda_address, rent_amount, space, program_id);
        let bump_seed = [bump];
        let mut signer_seeds: Vec<&[u8]> = seeds.to_vec();
        signer_seeds.push(&bump_seed);

        invoke_signed(
            &create_ix,
            &[payer.clone(), pda.clone(), system_program_account.clone()],
            &[signer_seeds.as_slice()],
        )?;

        Ok(bump)
    }
}
