use crate::{
    constants::{
        DICE_STATUS_DRAWN, RAFFLE_STATUS_DRAWN, RAFFLE_TICKET_COUNT,
        RUSSIAN_ROULETTE_PARTICIPATION_FEE, RUSSIAN_ROULETTE_STATUS_OPEN,
    },
    error::RPSProgramError::{
        InvalidDiceConfiguration, InvalidFeePercentage, InvalidRaffleAccount,
        InvalidRaffleConfiguration, InvalidRaffleState, InvalidRussianRouletteConfiguration,
    },
    state::{
        Config, DiceGame, DiceManager, Manager, RaffleManager, RaffleState, RussianRouletteGame,
        UpdateRussianRouletteTable,
    },
    utils::Utils,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};
use solana_system_interface::program as system_program;

use crate::error::RPSProgramError::{
    InvalidGameAccount, InvalidGameState, NotSignerAuth,
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
        Self::validate_russian_roulette_table(new_table)?;
        if table_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }

        let mut table: RussianRouletteGame =
            RussianRouletteGame::try_from_slice(&table_account.data.borrow())?;
        if table.draw_status != RUSSIAN_ROULETTE_STATUS_OPEN || table.number_of_players != 0 {
            return Err(InvalidGameState.into());
        }

        table.stake = new_table.stake;
        table.program_fee = new_table.program_fee;
        table.serialize(&mut &mut table_account.data.borrow_mut()[..])?;

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

        let raffle: RaffleState = RaffleState::try_from_slice(&raffle_account.data.borrow())?;

        if raffle.draw_status != RAFFLE_STATUS_DRAWN {
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

    pub fn close_dice_game(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let admin: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let dice_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let config_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        Utils::check_admin(admin, config_account, program_id)?;
        if dice_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }

        let dice: DiceGame = DiceGame::try_from_slice(&dice_account.data.borrow())?;
        if dice.draw_status != DICE_STATUS_DRAWN {
            return Err(InvalidGameState.into());
        }

        let rest = **dice_account.try_borrow_lamports()?;

        **dice_account.try_borrow_mut_lamports()? -= rest;
        **admin.try_borrow_mut_lamports()? += rest;

        dice_account.resize(0)?;
        dice_account.assign(&system_program::ID);

        Ok(())
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
        if game_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }

        let game: RussianRouletteGame =
            RussianRouletteGame::try_from_slice(&game_account.data.borrow())?;
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
        if table.stake == 0 || table.program_fee != RUSSIAN_ROULETTE_PARTICIPATION_FEE {
            return Err(InvalidRussianRouletteConfiguration.into());
        }

        Ok(())
    }
}
