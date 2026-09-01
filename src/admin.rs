use crate::{
    constants::{
        RAFFLE_SEED, RAFFLE_STATUS_DRAWN, RAFFLE_STATUS_VRF_FAILED, RAFFLE_TICKET_COUNT,
        RUSSIAN_ROULETTE_PLAYER_COUNT, RUSSIAN_ROULETTE_SEED, RUSSIAN_ROULETTE_STATUS_OPEN,
        RUSSIAN_ROULETTE_TABLE_COUNT,
    },
    error::RPSProgramError::{
        InvalidDiceConfiguration, InvalidFeePercentage, InvalidRaffleAccount,
        InvalidRaffleConfiguration, InvalidRaffleState, InvalidRussianRouletteConfiguration,
    },
    state::{
        Config, DiceManager, Manager, RaffleManager, RaffleState, RussianRouletteGame,
        UpdateRussianRouletteParticipationFee, UpdateRussianRouletteTable,
    },
    utils::Utils,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};
use solana_system_interface::program as system_program;

use crate::error::RPSProgramError::{
    ArithmeticError, InvalidGameAccount, InvalidGameState, NotSignerAuth,
};

pub struct Admin;
impl Admin {
    pub fn set_config(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let admin: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let admin_1: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let admin_2: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let admin_3: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let admin_4: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let admin_5: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let config_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        Utils::check_admin(admin, config_account, program_id)?;

        let config_data: Config = Config {
            is_init: 1,
            admin_1: admin_1.key.to_bytes(),
            admin_2: admin_2.key.to_bytes(),
            admin_3: admin_3.key.to_bytes(),
            admin_4: admin_4.key.to_bytes(),
            admin_5: admin_5.key.to_bytes(),
        };

        config_data.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn set_game_manager(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        new_manager: Manager,
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let admin: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let manager_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let config_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        Utils::check_admin(admin, config_account, program_id)?;
        Self::validate_game_manager(new_manager)?;

        let game_manager: Manager = Manager::try_from_slice(&manager_account.data.borrow())?;

        let manager: Manager = Manager {
            is_init: 1,
            allowed_time: new_manager.allowed_time,
            fee_percentage: new_manager.fee_percentage,
            minimum_stake: new_manager.minimum_stake,
            collected_fee: game_manager.collected_fee,
        };

        manager.serialize(&mut &mut manager_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn set_raffle_manager(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        new_manager: RaffleManager,
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let admin: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let manager_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let config_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        Utils::check_admin(admin, config_account, program_id)?;
        Self::validate_raffle_manager(new_manager)?;

        let current_manager: RaffleManager =
            RaffleManager::try_from_slice(&manager_account.data.borrow())?;

        let manager: RaffleManager = RaffleManager {
            is_init: 1,
            total_raffles: current_manager.total_raffles,
            ticket_price: new_manager.ticket_price,
            winner_gets: new_manager.winner_gets,
            decimal_digit_match_prize: new_manager.decimal_digit_match_prize,
            unit_digit_match_prize: new_manager.unit_digit_match_prize,
            program_fee: new_manager.program_fee,
        };

        manager.serialize(&mut &mut manager_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn set_dice_manager(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        new_manager: DiceManager,
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let admin: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let dice_manager_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let config_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        Utils::check_admin(admin, config_account, program_id)?;
        Self::validate_dice_manager(new_manager)?;

        let manager = DiceManager {
            is_init: 1,
            minimum_stake: new_manager.minimum_stake,
            program_fee: new_manager.program_fee,
        };

        manager.serialize(&mut &mut dice_manager_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn set_russian_roulette_table(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        new_table: UpdateRussianRouletteTable,
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let admin: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let table_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let config_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        Utils::check_admin(admin, config_account, program_id)?;
        let mut table = Self::load_russian_roulette_table(table_account, program_id)?;
        if table.draw_status != RUSSIAN_ROULETTE_STATUS_OPEN || table.number_of_players != 0 {
            return Err(InvalidGameState.into());
        }
        Self::validate_russian_roulette_table(new_table)?;
        if new_table.program_fee != table.program_fee {
            return Err(InvalidRussianRouletteConfiguration.into());
        }

        if table.stake != new_table.stake {
            table.stake = new_table.stake;
            table.round_id = table.round_id.checked_add(1).ok_or(ArithmeticError)?;
        }
        table.serialize(&mut &mut table_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn set_russian_roulette_participation_fee(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        update: UpdateRussianRouletteParticipationFee,
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let admin: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let table_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let config_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        Utils::check_admin(admin, config_account, program_id)?;
        let mut table = Self::load_russian_roulette_table(table_account, program_id)?;
        if table.round_id != update.expected_round_id
            || table.draw_status != RUSSIAN_ROULETTE_STATUS_OPEN
            || table.number_of_players != 0
        {
            return Err(InvalidGameState.into());
        }
        Self::validate_russian_roulette_table(UpdateRussianRouletteTable {
            stake: table.stake,
            program_fee: update.program_fee,
        })?;

        if table.program_fee == update.program_fee {
            return Ok(());
        }

        let previous_fee = table.program_fee;
        table.program_fee = update.program_fee;
        table.round_id = table.round_id.checked_add(1).ok_or(ArithmeticError)?;
        table.serialize(&mut &mut table_account.data.borrow_mut()[..])?;

        msg!(
            "roulette_participation_fee_updated table_id={} round_id={} previous_fee_lamports={} program_fee_lamports={}",
            table.table_id,
            table.round_id,
            previous_fee,
            table.program_fee
        );

        Ok(())
    }

    pub fn collect_fee(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let admin: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let manager_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let config_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        Utils::check_admin(admin, config_account, program_id)?;

        let mut manager: Manager = Manager::try_from_slice(&manager_account.data.borrow())?;
        let collected_fee = manager.collected_fee;

        if !admin.is_signer {
            return Err(NotSignerAuth.into());
        }

        manager.collected_fee = 0;

        **manager_account.try_borrow_mut_lamports()? -= collected_fee;
        **admin.try_borrow_mut_lamports()? += collected_fee;

        manager.serialize(&mut &mut manager_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn close_raffle(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let admin: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let raffle_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let config_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        Utils::check_admin(admin, config_account, program_id)?;
        if raffle_account.owner != program_id {
            return Err(InvalidRaffleAccount.into());
        }

        let raffle = RaffleState::try_from_slice(&raffle_account.data.borrow())?;
        let expected_raffle = Pubkey::find_program_address(
            &[RAFFLE_SEED, &raffle.raffle_no.to_le_bytes()],
            program_id,
        )
        .0;
        if raffle_account.key != &expected_raffle {
            return Err(InvalidRaffleAccount.into());
        }

        if raffle.draw_status != RAFFLE_STATUS_DRAWN
            && raffle.draw_status != RAFFLE_STATUS_VRF_FAILED
        {
            return Err(InvalidRaffleState.into());
        }
        if raffle.tickets_sold != 0 {
            return Err(InvalidRaffleState.into());
        }

        let rest = **raffle_account.try_borrow_lamports()?;

        **raffle_account.try_borrow_mut_lamports()? -= rest;
        **admin.try_borrow_mut_lamports()? += rest;

        raffle_account.resize(0)?;
        raffle_account.assign(&system_program::ID);

        Ok(())
    }

    pub fn close_dice_game(accounts: &[AccountInfo], _program_id: &Pubkey) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let _initializer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let dice_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        // Finalization closes intrinsically. The backend's following tag-24
        // instruction checks the resulting account-state invariant.
        if dice_account.owner == &system_program::ID
            && !dice_account.executable
            && dice_account.data_is_empty()
            && dice_account.lamports() == 0
        {
            return Ok(());
        }

        Err(InvalidGameAccount.into())
    }

    pub fn close_russian_roulette_game(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let admin: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let game_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let config_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        Utils::check_admin(admin, config_account, program_id)?;
        let game = Self::load_russian_roulette_table(game_account, program_id)?;
        if game.draw_status != RUSSIAN_ROULETTE_STATUS_OPEN || game.number_of_players != 0 {
            return Err(InvalidGameState.into());
        }

        let rest = **game_account.try_borrow_lamports()?;

        **game_account.try_borrow_mut_lamports()? -= rest;
        **admin.try_borrow_mut_lamports()? += rest;

        game_account.resize(0)?;
        game_account.assign(&system_program::ID);

        Ok(())
    }

    fn validate_game_manager(manager: Manager) -> ProgramResult {
        if manager.fee_percentage > 10_000 {
            return Err(InvalidFeePercentage.into());
        }
        if manager.allowed_time == 0 || manager.minimum_stake == 0 {
            return Err(InvalidRaffleConfiguration.into());
        }

        Ok(())
    }

    fn validate_raffle_manager(manager: RaffleManager) -> ProgramResult {
        if manager.ticket_price == 0 {
            return Err(InvalidRaffleConfiguration.into());
        }

        let total_pool = (manager.ticket_price as u128) * (RAFFLE_TICKET_COUNT as u128);
        let max_payout = (manager.winner_gets as u128)
            .checked_add((manager.decimal_digit_match_prize as u128) * 9)
            .and_then(|value| value.checked_add((manager.unit_digit_match_prize as u128) * 9))
            .and_then(|value| value.checked_add(manager.program_fee as u128))
            .ok_or(InvalidRaffleConfiguration)?;

        if max_payout > total_pool {
            return Err(InvalidRaffleConfiguration.into());
        }

        Ok(())
    }

    fn validate_dice_manager(manager: DiceManager) -> ProgramResult {
        if manager.minimum_stake == 0 {
            return Err(InvalidDiceConfiguration.into());
        }

        let minimum_pool = (manager.minimum_stake as u128) * 6;
        if (manager.program_fee as u128) >= minimum_pool {
            return Err(InvalidDiceConfiguration.into());
        }

        Ok(())
    }

    fn validate_russian_roulette_table(table: UpdateRussianRouletteTable) -> ProgramResult {
        if table.stake == 0
            || table.program_fee >= table.stake
            || table
                .stake
                .checked_add(table.program_fee)
                .and_then(|payment| payment.checked_mul(RUSSIAN_ROULETTE_PLAYER_COUNT as u64))
                .is_none()
        {
            return Err(InvalidRussianRouletteConfiguration.into());
        }

        Ok(())
    }

    fn load_russian_roulette_table(
        table_account: &AccountInfo,
        program_id: &Pubkey,
    ) -> Result<RussianRouletteGame, solana_program::program_error::ProgramError> {
        if table_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }

        let table = RussianRouletteGame::try_from_slice(&table_account.data.borrow())?;
        if table.table_id >= RUSSIAN_ROULETTE_TABLE_COUNT {
            return Err(InvalidGameAccount.into());
        }
        let table_id = [table.table_id];
        let expected_table =
            Pubkey::find_program_address(&[RUSSIAN_ROULETTE_SEED, &table_id], program_id).0;
        if table_account.key != &expected_table {
            return Err(InvalidGameAccount.into());
        }

        Ok(table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{
        RUSSIAN_ROULETTE_STATUS_DRAWN, RUSSIAN_ROULETTE_STATUS_PENDING, UNLUCKY_PLAYER_INDEX_NONE,
    };
    use borsh::to_vec;
    use solana_program::program_error::ProgramError;

    fn table(number_of_players: u8, draw_status: u8) -> RussianRouletteGame {
        RussianRouletteGame {
            table_id: 1,
            round_id: 12,
            draw_status,
            number_of_players,
            stake: 500_000_000,
            program_fee: 10_000_000,
            seat_1: [0; 32],
            seat_2: [0; 32],
            seat_3: [0; 32],
            seat_4: [0; 32],
            seat_5: [0; 32],
            seat_6: [0; 32],
            unlucky_player_index: UNLUCKY_PLAYER_INDEX_NONE,
            vrf_seed: [0; 32],
            unlucky_player: [0; 32],
            vrf_last_request_at: 0,
            vrf_retry_count: 0,
            settled_at: 0,
        }
    }

    #[derive(Clone, Copy)]
    enum RouletteAdminUpdate {
        Fee(UpdateRussianRouletteParticipationFee),
        Table(UpdateRussianRouletteTable),
    }

    #[test]
    fn dice_close_confirmation_accepts_an_empty_system_account() {
        let program_id = Pubkey::new_unique();
        let initializer_key = Pubkey::new_unique();
        let dice_key = Pubkey::new_unique();
        let system_owner = system_program::ID;
        let mut initializer_lamports = 0;
        let mut dice_lamports = 0;
        let mut initializer_data = [];
        let mut dice_data = [];
        let initializer = AccountInfo::new(
            &initializer_key,
            false,
            true,
            &mut initializer_lamports,
            &mut initializer_data,
            &system_owner,
            false,
        );
        let dice = AccountInfo::new(
            &dice_key,
            false,
            true,
            &mut dice_lamports,
            &mut dice_data,
            &system_owner,
            false,
        );

        assert!(Admin::close_dice_game(&[initializer, dice], &program_id).is_ok());
    }

    #[test]
    fn dice_close_confirmation_rejects_a_prefunded_system_account() {
        let program_id = Pubkey::new_unique();
        let initializer_key = Pubkey::new_unique();
        let dice_key = Pubkey::new_unique();
        let system_owner = system_program::ID;
        let mut initializer_lamports = 0;
        let mut dice_lamports = 1;
        let mut initializer_data = [];
        let mut dice_data = [];
        let initializer = AccountInfo::new(
            &initializer_key,
            false,
            true,
            &mut initializer_lamports,
            &mut initializer_data,
            &system_owner,
            false,
        );
        let dice = AccountInfo::new(
            &dice_key,
            false,
            true,
            &mut dice_lamports,
            &mut dice_data,
            &system_owner,
            false,
        );

        assert_eq!(
            Admin::close_dice_game(&[initializer, dice], &program_id).unwrap_err(),
            ProgramError::from(InvalidGameAccount)
        );
    }

    #[test]
    fn dice_close_confirmation_rejects_a_program_owned_account() {
        let program_id = Pubkey::new_unique();
        let initializer_key = Pubkey::new_unique();
        let dice_key = Pubkey::new_unique();
        let system_owner = system_program::ID;
        let mut initializer_lamports = 0;
        let mut dice_lamports = 1;
        let mut initializer_data = [];
        let mut dice_data = [0u8; 1];
        let initializer = AccountInfo::new(
            &initializer_key,
            false,
            true,
            &mut initializer_lamports,
            &mut initializer_data,
            &system_owner,
            false,
        );
        let dice = AccountInfo::new(
            &dice_key,
            false,
            true,
            &mut dice_lamports,
            &mut dice_data,
            &program_id,
            false,
        );

        assert_eq!(
            Admin::close_dice_game(&[initializer, dice], &program_id).unwrap_err(),
            ProgramError::from(InvalidGameAccount)
        );
    }

    fn run_admin_update(
        initial_table: RussianRouletteGame,
        update: RouletteAdminUpdate,
        is_admin: bool,
        is_signer: bool,
        valid_table_address: bool,
    ) -> (Result<(), ProgramError>, RussianRouletteGame) {
        let program_id = Pubkey::new_unique();
        let admin_key = Pubkey::new_unique();
        let configured_admin = if is_admin {
            admin_key
        } else {
            Pubkey::new_unique()
        };
        let table_id = [initial_table.table_id];
        let derived_table =
            Pubkey::find_program_address(&[RUSSIAN_ROULETTE_SEED, &table_id], &program_id).0;
        let table_key = if valid_table_address {
            derived_table
        } else {
            Pubkey::new_unique()
        };
        let config_key = Pubkey::new_unique();
        let system_owner = system_program::ID;
        let config = Config {
            is_init: 1,
            admin_1: configured_admin.to_bytes(),
            admin_2: [0; 32],
            admin_3: [0; 32],
            admin_4: [0; 32],
            admin_5: [0; 32],
        };
        let mut admin_lamports = 0;
        let mut table_lamports = 1;
        let mut config_lamports = 1;
        let mut admin_data = [];
        let mut table_data = to_vec(&initial_table).unwrap();
        let mut config_data = to_vec(&config).unwrap();

        let result = {
            let admin_account = AccountInfo::new(
                &admin_key,
                is_signer,
                true,
                &mut admin_lamports,
                &mut admin_data,
                &system_owner,
                false,
            );
            let table_account = AccountInfo::new(
                &table_key,
                false,
                true,
                &mut table_lamports,
                &mut table_data,
                &program_id,
                false,
            );
            let config_account = AccountInfo::new(
                &config_key,
                false,
                false,
                &mut config_lamports,
                &mut config_data,
                &program_id,
                false,
            );

            let accounts = [admin_account, table_account, config_account];
            match update {
                RouletteAdminUpdate::Fee(update) => {
                    Admin::set_russian_roulette_participation_fee(&accounts, &program_id, update)
                }
                RouletteAdminUpdate::Table(update) => {
                    Admin::set_russian_roulette_table(&accounts, &program_id, update)
                }
            }
        };
        let updated_table = RussianRouletteGame::try_from_slice(&table_data).unwrap();

        (result, updated_table)
    }

    fn run_fee_update(
        initial_table: RussianRouletteGame,
        update: UpdateRussianRouletteParticipationFee,
        is_admin: bool,
        is_signer: bool,
        valid_table_address: bool,
    ) -> (Result<(), ProgramError>, RussianRouletteGame) {
        run_admin_update(
            initial_table,
            RouletteAdminUpdate::Fee(update),
            is_admin,
            is_signer,
            valid_table_address,
        )
    }

    fn run_table_update(
        initial_table: RussianRouletteGame,
        update: UpdateRussianRouletteTable,
    ) -> (Result<(), ProgramError>, RussianRouletteGame) {
        run_admin_update(
            initial_table,
            RouletteAdminUpdate::Table(update),
            true,
            true,
            true,
        )
    }

    #[test]
    fn admin_updates_fee_on_an_empty_open_table_and_advances_round() {
        let (result, updated) = run_fee_update(
            table(0, RUSSIAN_ROULETTE_STATUS_OPEN),
            UpdateRussianRouletteParticipationFee {
                expected_round_id: 12,
                program_fee: 25_000_000,
            },
            true,
            true,
            true,
        );

        result.unwrap();
        assert_eq!(updated.stake, 500_000_000);
        assert_eq!(updated.program_fee, 25_000_000);
        assert_eq!(updated.round_id, 13);
    }

    #[test]
    fn unchanged_fee_is_idempotent_and_does_not_advance_round() {
        let (result, updated) = run_fee_update(
            table(0, RUSSIAN_ROULETTE_STATUS_OPEN),
            UpdateRussianRouletteParticipationFee {
                expected_round_id: 12,
                program_fee: 10_000_000,
            },
            true,
            true,
            true,
        );

        result.unwrap();
        assert_eq!(updated.round_id, 12);
    }

    #[test]
    fn stake_table_setter_cannot_change_the_fee_or_bypass_the_round_bump() {
        let (mismatch_result, unchanged) = run_table_update(
            table(0, RUSSIAN_ROULETTE_STATUS_OPEN),
            UpdateRussianRouletteTable {
                stake: 750_000_000,
                program_fee: 25_000_000,
            },
        );
        assert_eq!(
            mismatch_result.unwrap_err(),
            ProgramError::from(InvalidRussianRouletteConfiguration)
        );
        assert_eq!(unchanged.stake, 500_000_000);
        assert_eq!(unchanged.program_fee, 10_000_000);
        assert_eq!(unchanged.round_id, 12);

        let (update_result, updated) = run_table_update(
            table(0, RUSSIAN_ROULETTE_STATUS_OPEN),
            UpdateRussianRouletteTable {
                stake: 750_000_000,
                program_fee: 10_000_000,
            },
        );
        update_result.unwrap();
        assert_eq!(updated.stake, 750_000_000);
        assert_eq!(updated.program_fee, 10_000_000);
        assert_eq!(updated.round_id, 13);
    }

    #[test]
    fn fee_update_rejects_stale_active_and_pending_tables() {
        for initial_table in [
            table(0, RUSSIAN_ROULETTE_STATUS_OPEN),
            table(1, RUSSIAN_ROULETTE_STATUS_OPEN),
            table(6, RUSSIAN_ROULETTE_STATUS_PENDING),
            table(6, RUSSIAN_ROULETTE_STATUS_DRAWN),
        ] {
            let expected_round_id = if initial_table.number_of_players == 0 {
                11
            } else {
                12
            };
            let (result, _) = run_fee_update(
                initial_table,
                UpdateRussianRouletteParticipationFee {
                    expected_round_id,
                    program_fee: 25_000_000,
                },
                true,
                true,
                true,
            );

            assert_eq!(result.unwrap_err(), ProgramError::from(InvalidGameState));
        }
    }

    #[test]
    fn fee_update_requires_an_authorized_signer_and_exact_table_pda() {
        let cases = [
            (
                false,
                true,
                true,
                ProgramError::from(crate::error::RPSProgramError::InvalidAuth),
            ),
            (
                true,
                false,
                true,
                ProgramError::from(crate::error::RPSProgramError::NotSignerAuth),
            ),
            (true, true, false, ProgramError::from(InvalidGameAccount)),
        ];

        for (is_admin, is_signer, valid_table_address, expected_error) in cases {
            let (result, _) = run_fee_update(
                table(0, RUSSIAN_ROULETTE_STATUS_OPEN),
                UpdateRussianRouletteParticipationFee {
                    expected_round_id: 12,
                    program_fee: 25_000_000,
                },
                is_admin,
                is_signer,
                valid_table_address,
            );

            assert_eq!(result.unwrap_err(), expected_error);
        }
    }

    #[test]
    fn roulette_fee_validation_allows_zero_and_rejects_stake_or_overflowing_fees() {
        assert!(
            Admin::validate_russian_roulette_table(UpdateRussianRouletteTable {
                stake: 500_000_000,
                program_fee: 0,
            })
            .is_ok()
        );
        assert!(
            Admin::validate_russian_roulette_table(UpdateRussianRouletteTable {
                stake: 500_000_000,
                program_fee: 499_999_999,
            })
            .is_ok()
        );

        for program_fee in [500_000_000, u64::MAX] {
            assert_eq!(
                Admin::validate_russian_roulette_table(UpdateRussianRouletteTable {
                    stake: 500_000_000,
                    program_fee,
                })
                .unwrap_err(),
                ProgramError::from(InvalidRussianRouletteConfiguration)
            );
        }

        assert_eq!(
            Admin::validate_russian_roulette_table(UpdateRussianRouletteTable {
                stake: u64::MAX / 4,
                program_fee: 1,
            })
            .unwrap_err(),
            ProgramError::from(InvalidRussianRouletteConfiguration)
        );
    }
}
