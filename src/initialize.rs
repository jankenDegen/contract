use crate::{
    constants::{
        CONFIG_SEED, CONFIG_SPACE, DICE_MANAGER_SEED, DICE_MANAGER_SPACE, MANAGER_SEED,
        MANAGER_SPACE, RAFFLE_MANAGER_SEED, RAFFLE_MANAGER_SPACE, RUSSIAN_ROULETTE_GAME_SPACE,
        RUSSIAN_ROULETTE_PARTICIPATION_FEE, RUSSIAN_ROULETTE_PLAYER_COUNT, RUSSIAN_ROULETTE_SEED,
        RUSSIAN_ROULETTE_STATUS_OPEN, RUSSIAN_ROULETTE_TABLE_COUNT, UNLUCKY_PLAYER_INDEX_NONE,
    },
    error::RPSProgramError::InvalidRussianRouletteConfiguration,
    state::{
        Config, DiceManager, InitRussianRouletteTable, Manager, RaffleManager, RussianRouletteGame,
    },
    utils::Utils,
};
use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

pub struct Init;
impl Init {
    pub fn init_game_accounts(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let payer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let manager_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let admin_1: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let admin_2: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let admin_3: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let admin_4: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let admin_5: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let config_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let raffle_manager_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let system_program_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        Utils::create_pda(
            payer,
            manager_account,
            system_program_account,
            &[MANAGER_SEED],
            MANAGER_SPACE,
            program_id,
        )?;
        Utils::create_pda(
            payer,
            config_account,
            system_program_account,
            &[CONFIG_SEED],
            CONFIG_SPACE,
            program_id,
        )?;
        Utils::create_pda(
            payer,
            raffle_manager_account,
            system_program_account,
            &[RAFFLE_MANAGER_SEED],
            RAFFLE_MANAGER_SPACE,
            program_id,
        )?;

        let manager: Manager = Manager {
            is_init: 1,
            allowed_time: 60,
            fee_percentage: 100,
            minimum_stake: 10_000_000,
            collected_fee: 0,
        };

        let raffle_manager: RaffleManager = RaffleManager {
            is_init: 1,
            total_raffles: 0,
            ticket_price: 20_000_000,
            winner_gets: 1_000_000_000,
            decimal_digit_match_prize: 40_000_000,
            unit_digit_match_prize: 60_000_000,
            program_fee: 100_000_000,
        };

        let config_data: Config = Config {
            is_init: 1,
            admin_1: admin_1.key.to_bytes(),
            admin_2: admin_2.key.to_bytes(),
            admin_3: admin_3.key.to_bytes(),
            admin_4: admin_4.key.to_bytes(),
            admin_5: admin_5.key.to_bytes(),
        };

        config_data.serialize(&mut &mut config_account.data.borrow_mut()[..])?;
        manager.serialize(&mut &mut manager_account.data.borrow_mut()[..])?;
        raffle_manager.serialize(&mut &mut raffle_manager_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn init_dice_manager(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let admin: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let dice_manager_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let config_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let system_program_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        Utils::check_admin(admin, config_account, program_id)?;

        Utils::create_pda(
            admin,
            dice_manager_account,
            system_program_account,
            &[DICE_MANAGER_SEED],
            DICE_MANAGER_SPACE,
            program_id,
        )?;

        let dice_manager = DiceManager {
            is_init: 1,
            minimum_stake: 10_000_000,
            program_fee: 3_000_000,
        };

        dice_manager.serialize(&mut &mut dice_manager_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn init_russian_roulette_table(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        init_table: InitRussianRouletteTable,
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let admin: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let table_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let config_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let system_program_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        Utils::check_admin(admin, config_account, program_id)?;
        Self::validate_roulette_table_config(
            init_table.table_id,
            init_table.stake,
            init_table.program_fee,
        )?;

        let table_id_bytes = [init_table.table_id];

        Utils::create_pda(
            admin,
            table_account,
            system_program_account,
            &[RUSSIAN_ROULETTE_SEED, &table_id_bytes],
            RUSSIAN_ROULETTE_GAME_SPACE,
            program_id,
        )?;

        let table = RussianRouletteGame {
            table_id: init_table.table_id,
            round_id: 1,
            draw_status: RUSSIAN_ROULETTE_STATUS_OPEN,
            number_of_players: 0,
            stake: init_table.stake,
            program_fee: init_table.program_fee,
            seat_1: [0; 32],
            seat_2: [0; 32],
            seat_3: [0; 32],
            seat_4: [0; 32],
            seat_5: [0; 32],
            seat_6: [0; 32],
            unlucky_player_index: UNLUCKY_PLAYER_INDEX_NONE,
            vrf_seed: [0; 32],
            unlucky_player: [0; 32],
        };

        table.serialize(&mut &mut table_account.data.borrow_mut()[..])?;

        Ok(())
    }

    fn validate_roulette_table_config(table_id: u8, stake: u64, program_fee: u64) -> ProgramResult {
        if table_id >= RUSSIAN_ROULETTE_TABLE_COUNT
            || stake == 0
            || program_fee != RUSSIAN_ROULETTE_PARTICIPATION_FEE
        {
            return Err(InvalidRussianRouletteConfiguration.into());
        }

        if RUSSIAN_ROULETTE_PLAYER_COUNT != 6 {
            return Err(InvalidRussianRouletteConfiguration.into());
        }

        Ok(())
    }
}
