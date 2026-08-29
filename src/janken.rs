use crate::{
    constants::{GAME_SEED, GAME_SPACE},
    state::{Game, InitGame, JoinGame, Manager, Reveal},
    utils::Utils,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    keccak, msg,
    program::invoke,
    pubkey::Pubkey,
    sysvar::{clock, Sysvar},
};
use solana_system_interface::{instruction::transfer, program as system_program};

use crate::error::RPSProgramError::{
    ArithmeticError, InvalidGameAccount, InvalidGameState, InvalidGuest, InvalidHash,
    InvalidInitializer, InvalidManager, InvalidTime, ManagerWritable, MinStake, PlayerNotSigner,
};

pub struct Janken;
impl Janken {
    pub fn init_game(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        init_game: InitGame,
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let initializer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let game_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let manager_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let system_program_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        let manager: Manager = Manager::try_from_slice(&manager_account.data.borrow())?;

        Self::check_game_initialization_accounts(manager_account, program_id, manager)?;

        if manager.minimum_stake > init_game.amount {
            return Err(MinStake.into());
        }

        let seeds: &[&[u8]] = &[GAME_SEED, &init_game.hash];

        Utils::create_pda(
            initializer,
            game_account,
            system_program_account,
            seeds,
            GAME_SPACE,
            program_id,
        )?;

        invoke(
            &transfer(initializer.key, game_account.key, init_game.amount),
            &[
                initializer.clone(),
                game_account.clone(),
                system_program_account.clone(),
            ],
        )?;

        let game_state: Game = Game {
            initializer: initializer.key.to_bytes(),
            hash: init_game.hash,
            guest: [0; 32],
            fee_percentage: manager.fee_percentage,
            amount: init_game.amount,
            state: 1,
            allowed_time: manager.allowed_time,
            last_play_time: 0,
            guest_decision: 0,
        };

        game_state.serialize(&mut &mut game_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn join_game(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        join_game: JoinGame,
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let guest: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let game_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let system_program_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        if !guest.is_signer {
            return Err(PlayerNotSigner.into());
        }

        let mut game: Game = Game::try_from_slice(&game_account.data.borrow())?;

        Self::check_game_participation_accounts(game_account, program_id, game)?;

        if join_game.decision != 1 && join_game.decision != 2 && join_game.decision != 3 {
            return Err(InvalidGameAccount.into());
        }

        let clock: Clock = Clock::get()?;
        let unix_timestamp: u64 = clock.unix_timestamp as u64;

        invoke(
            &transfer(guest.key, game_account.key, game.amount),
            &[
                guest.clone(),
                game_account.clone(),
                system_program_account.clone(),
            ],
        )?;

        game.guest_decision = join_game.decision;
        game.last_play_time = unix_timestamp;
        game.state = 2;
        game.guest = guest.key.to_bytes();

        game.serialize(&mut &mut game_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn reveal(accounts: &[AccountInfo], program_id: &Pubkey, reveal: Reveal) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let initializer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let guest: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let game_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let manager_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        let mut manager: Manager = Manager::try_from_slice(&manager_account.data.borrow())?;
        let game: Game = Game::try_from_slice(&game_account.data.borrow())?;

        Self::check_finish_game_accounts(
            manager_account,
            game_account,
            program_id,
            initializer.key,
            guest.key,
            manager,
            game,
        )?;

        if game_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }

        let hash: keccak::Hash = keccak::hashv(&[&reveal.decision.to_le_bytes(), &reveal.seed]);

        if reveal.decision != 1 && reveal.decision != 2 && reveal.decision != 3 {
            return Err(InvalidGameAccount.into());
        }

        if game.hash != hash.to_bytes() {
            return Err(InvalidHash.into());
        }

        let (prize, fee) = Self::calculate_prize_and_fee(game.amount, game.fee_percentage);

        let (outcome, guest_payout) = match (reveal.decision, game.guest_decision) {
            (1, 1) | (2, 2) | (3, 3) => {
                ("draw", prize / 2)
            }
            (1, 2) | (2, 3) | (3, 1) => {
                ("guest_win", prize)
            }
            _ => {
                ("initializer_win", 0)
            }
        };

        Self::settle_game(
            game_account,
            manager_account,
            initializer,
            guest,
            fee,
            guest_payout,
        )?;

        game_account.resize(0)?;
        game_account.assign(&system_program::ID);

        manager.collected_fee = manager
            .collected_fee
            .checked_add(fee)
            .ok_or(ArithmeticError)?;

        manager.serialize(&mut &mut manager_account.data.borrow_mut()[..])?;

        msg!(
            "rps_finalized outcome={} game_address={} initializer={} initializer_throw={} initializer_throw_name={} guest={} guest_throw={} guest_throw_name={} stake_lamports={} committed_hash={} revealed_hash={} initializer_seed={}",
            outcome,
            game_account.key,
            initializer.key,
            reveal.decision,
            Self::decision_name(reveal.decision),
            guest.key,
            game.guest_decision,
            Self::decision_name(game.guest_decision),
            game.amount,
            Pubkey::new_from_array(game.hash),
            Pubkey::new_from_array(hash.to_bytes()),
            Pubkey::new_from_array(reveal.seed)
        );
        match outcome {
            "guest_win" => msg!(
                "rps_players winner={} winner_throw={} loser={} loser_throw={}",
                guest.key,
                game.guest_decision,
                initializer.key,
                reveal.decision
            ),
            "initializer_win" => msg!(
                "rps_players winner={} winner_throw={} loser={} loser_throw={}",
                initializer.key,
                reveal.decision,
                guest.key,
                game.guest_decision
            ),
            _ => msg!("rps_players winner=none loser=none outcome=draw"),
        }

        Ok(())
    }

    pub fn time_is_up(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let game_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let manager_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let guest: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let initializer: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        let game: Game = Game::try_from_slice(&game_account.data.borrow())?;
        let mut manager: Manager = Manager::try_from_slice(&manager_account.data.borrow())?;

        Self::check_finish_game_accounts(
            manager_account,
            game_account,
            program_id,
            initializer.key,
            guest.key,
            manager,
            game,
        )?;

        let game_state: Game = Game::try_from_slice(&game_account.data.borrow())?;

        let clock: Clock = clock::Clock::get()?;
        let current_time: u64 = clock.unix_timestamp as u64;

        let time_passed: u64 = current_time
            .checked_sub(game_state.last_play_time)
            .ok_or(ArithmeticError)?;

        if time_passed < game.allowed_time {
            return Err(InvalidTime.into());
        }

        let (_prize, fee) = Self::calculate_prize_and_fee(game_state.amount, game.fee_percentage);

        manager.collected_fee = manager
            .collected_fee
            .checked_add(fee)
            .ok_or(ArithmeticError)?;

        **game_account.try_borrow_mut_lamports()? -= fee;
        **manager_account.try_borrow_mut_lamports()? += fee;

        let rest: u64 = **game_account.try_borrow_lamports()?;

        **game_account.try_borrow_mut_lamports()? -= rest;
        **guest.try_borrow_mut_lamports()? += rest;

        game_account.resize(0)?;
        game_account.assign(&system_program::ID);

        manager.serialize(&mut &mut manager_account.data.borrow_mut()[..])?;

        msg!(
            "rps_finalized outcome=guest_win_timeout game_address={} winner={} winner_throw={} winner_throw_name={} loser={} loser_throw=unrevealed stake_lamports={} committed_hash={} revealed_hash=unavailable initializer_seed=unavailable",
            game_account.key,
            guest.key,
            game_state.guest_decision,
            Self::decision_name(game_state.guest_decision),
            initializer.key,
            game_state.amount,
            Pubkey::new_from_array(game_state.hash)
        );

        Ok(())
    }

    fn check_game_initialization_accounts(
        manager_account: &AccountInfo,
        program_id: &Pubkey,
        manager: Manager,
    ) -> ProgramResult {
        if manager_account.is_writable {
            return Err(ManagerWritable.into());
        }

        if manager_account.owner != program_id {
            return Err(InvalidManager.into());
        }

        if manager.is_init != 1 {
            return Err(InvalidManager.into());
        }

        Ok(())
    }

    fn check_game_participation_accounts(
        game_account: &AccountInfo,
        program_id: &Pubkey,
        game: Game,
    ) -> ProgramResult {
        if game_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }

        if game.state != 1 {
            return Err(InvalidGameState.into());
        }

        let (key, _bump) = Pubkey::find_program_address(&[GAME_SEED, &game.hash], program_id);
        if &key != game_account.key {
            return Err(InvalidGameAccount.into());
        }

        Ok(())
    }

    fn check_finish_game_accounts(
        manager_account: &AccountInfo,
        game_account: &AccountInfo,
        program_id: &Pubkey,
        initializer: &Pubkey,
        guest: &Pubkey,
        manager: Manager,
        game: Game,
    ) -> ProgramResult {
        if game_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }

        if game.state != 2 {
            return Err(InvalidGameState.into());
        }

        if manager_account.owner != program_id {
            return Err(InvalidManager.into());
        }

        if manager.is_init != 1 {
            return Err(InvalidManager.into());
        }

        let (key, _bump) = Pubkey::find_program_address(&[GAME_SEED, &game.hash], program_id);

        if guest.to_bytes() != game.guest {
            return Err(InvalidGuest.into());
        }

        if initializer.to_bytes() != game.initializer {
            return Err(InvalidInitializer.into());
        }

        if &key != game_account.key {
            return Err(InvalidGameAccount.into());
        }

        Ok(())
    }

    fn calculate_prize_and_fee(stake: u64, fee_percentage: u16) -> (u64, u64) {
        const FEE_PRECISION: u128 = 10_000;

        let total_amount: u128 = (stake * 2) as u128;

        let fee: u128 = (fee_percentage as u128 * total_amount) / FEE_PRECISION;

        let fee_u64: u64 = fee as u64;
        let prize_u64: u64 = total_amount as u64 - fee_u64;

        (prize_u64, fee_u64)
    }

    fn decision_name(decision: u8) -> &'static str {
        match decision {
            1 => "rock",
            2 => "paper",
            3 => "scissors",
            _ => "unknown",
        }
    }

    fn settle_game(
        game_account: &AccountInfo,
        manager_account: &AccountInfo,
        initializer: &AccountInfo,
        guest: &AccountInfo,
        fee: u64,
        guest_payout: u64,
    ) -> ProgramResult {
        if fee > 0 {
            **game_account.try_borrow_mut_lamports()? -= fee;
            **manager_account.try_borrow_mut_lamports()? += fee;
        }

        if guest_payout > 0 {
            **game_account.try_borrow_mut_lamports()? -= guest_payout;
            **guest.try_borrow_mut_lamports()? += guest_payout;
        }

        let initializer_payout = **game_account.try_borrow_lamports()?;
        if initializer_payout > 0 {
            **game_account.try_borrow_mut_lamports()? -= initializer_payout;
            **initializer.try_borrow_mut_lamports()? += initializer_payout;
        }

        Ok(())
    }
}
