use crate::{
    constants::{
        MANAGER_SEED, RUSSIAN_ROULETTE_PLAYER_COUNT, RUSSIAN_ROULETTE_SEED,
        RUSSIAN_ROULETTE_STATUS_DRAWN, RUSSIAN_ROULETTE_STATUS_OPEN,
        RUSSIAN_ROULETTE_STATUS_PENDING, RUSSIAN_ROULETTE_TABLE_COUNT,
        RUSSIAN_ROULETTE_VRF_CALLBACK_TAG, RUSSIAN_ROULETTE_VRF_SEED, UNLUCKY_PLAYER_INDEX_NONE,
    },
    error::RPSProgramError::{
        ArithmeticError, InvalidGameAccount, InvalidGameState, InvalidManager, InvalidPlayer,
        PlayerNotSigner, RussianRouletteDrawAlreadyRequested, RussianRouletteDrawPending,
        RussianRouletteGameNotReady,
    },
    magicblock_vrf::MagicBlockVrf,
    state::{InitRussianRoulette, JoinRussianRoulette, Manager, RussianRouletteGame},
    utils::Utils,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    hash::hash,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};
use solana_system_interface::instruction::transfer;

pub struct RussianRoulette;

impl RussianRoulette {
    pub fn create_game(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        init_game: InitRussianRoulette,
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let player: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let game_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let system_program_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        if !player.is_signer {
            return Err(PlayerNotSigner.into());
        }
        if game_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }

        let mut game = RussianRouletteGame::try_from_slice(&game_account.data.borrow())?;
        if game.table_id != init_game.table_id {
            return Err(InvalidGameAccount.into());
        }
        if game.round_id != init_game.expected_round_id {
            return Err(InvalidGameState.into());
        }
        Self::sit_player(&mut game, player.key, init_game.seat)?;
        Self::collect_participation_payment(player, game_account, system_program_account, &game)?;

        game.serialize(&mut &mut game_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn join_game(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        join_game: JoinRussianRoulette,
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let player: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let game_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let system_program_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        if !player.is_signer {
            return Err(PlayerNotSigner.into());
        }
        if game_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }

        let mut game = RussianRouletteGame::try_from_slice(&game_account.data.borrow())?;
        if game.round_id != join_game.expected_round_id {
            return Err(InvalidGameState.into());
        }
        Self::sit_player(&mut game, player.key, join_game.seat)?;
        Self::collect_participation_payment(player, game_account, system_program_account, &game)?;

        game.serialize(&mut &mut game_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn request_draw(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        expected_round_id: u64,
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let payer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let game_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let vrf_request_identity: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let oracle_queue: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let system_program_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let slot_hashes_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let vrf_program_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let manager_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let seat_1_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let seat_2_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let seat_3_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let seat_4_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let seat_5_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let seat_6_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        if !payer.is_signer {
            return Err(PlayerNotSigner.into());
        }
        if game_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }

        let mut game = RussianRouletteGame::try_from_slice(&game_account.data.borrow())?;
        Self::validate_game_account(game_account, &game, program_id)?;
        if game.round_id != expected_round_id {
            return Err(InvalidGameState.into());
        }
        if game.number_of_players != RUSSIAN_ROULETTE_PLAYER_COUNT {
            return Err(RussianRouletteGameNotReady.into());
        }
        if game.draw_status != RUSSIAN_ROULETTE_STATUS_OPEN {
            return Err(RussianRouletteDrawAlreadyRequested.into());
        }
        let player_accounts = [
            seat_1_account,
            seat_2_account,
            seat_3_account,
            seat_4_account,
            seat_5_account,
            seat_6_account,
        ];
        Self::load_fee_manager(manager_account, program_id)?;
        Self::validate_player_accounts(&game, &player_accounts)?;

        let request_slot = Clock::get()?.slot;
        let vrf_seed =
            Self::vrf_request_seed(game_account.key, game.table_id, game.round_id, request_slot);
        let callback_args = Self::vrf_callback_args(game.round_id, &vrf_seed);
        let callback_accounts = [(*game_account.key, false, true)];
        MagicBlockVrf::request_randomness_with_callback_accounts(
            RUSSIAN_ROULETTE_VRF_CALLBACK_TAG,
            &callback_args,
            payer,
            vrf_request_identity,
            oracle_queue,
            system_program_account,
            slot_hashes_account,
            vrf_program_account,
            program_id,
            &callback_accounts,
            vrf_seed,
        )?;

        game.draw_status = RUSSIAN_ROULETTE_STATUS_PENDING;
        game.unlucky_player_index = UNLUCKY_PLAYER_INDEX_NONE;
        game.vrf_seed = vrf_seed;
        game.serialize(&mut &mut game_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn retry_draw(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        expected_round_id: u64,
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let payer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let game_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let vrf_request_identity: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let oracle_queue: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let system_program_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let slot_hashes_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let vrf_program_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let manager_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let seat_1_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let seat_2_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let seat_3_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let seat_4_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let seat_5_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let seat_6_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let config_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        Utils::check_admin(payer, config_account, program_id)?;
        if game_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }

        let game = RussianRouletteGame::try_from_slice(&game_account.data.borrow())?;
        Self::validate_game_account(game_account, &game, program_id)?;
        if game.round_id != expected_round_id {
            return Err(InvalidGameState.into());
        }
        if game.number_of_players != RUSSIAN_ROULETTE_PLAYER_COUNT {
            return Err(RussianRouletteGameNotReady.into());
        }
        if game.draw_status != RUSSIAN_ROULETTE_STATUS_PENDING {
            return Err(InvalidGameState.into());
        }
        if game.unlucky_player_index != UNLUCKY_PLAYER_INDEX_NONE {
            return Err(RussianRouletteDrawAlreadyRequested.into());
        }

        let player_accounts = [
            seat_1_account,
            seat_2_account,
            seat_3_account,
            seat_4_account,
            seat_5_account,
            seat_6_account,
        ];
        Self::load_fee_manager(manager_account, program_id)?;
        Self::validate_player_accounts(&game, &player_accounts)?;

        let request_seed = game.vrf_seed;
        let retry_slot = Clock::get()?.slot;
        let retry_caller_seed = Self::retry_vrf_request_seed(
            game_account.key,
            game.table_id,
            game.round_id,
            &request_seed,
            retry_slot,
        );
        let callback_args = Self::vrf_callback_args(game.round_id, &request_seed);
        let callback_accounts = [(*game_account.key, false, true)];
        MagicBlockVrf::request_randomness_with_callback_accounts(
            RUSSIAN_ROULETTE_VRF_CALLBACK_TAG,
            &callback_args,
            payer,
            vrf_request_identity,
            oracle_queue,
            system_program_account,
            slot_hashes_account,
            vrf_program_account,
            program_id,
            &callback_accounts,
            retry_caller_seed,
        )?;

        Ok(())
    }

    pub fn finalize_draw(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        expected_round_id: u64,
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let game_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let manager_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let seat_1_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let seat_2_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let seat_3_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let seat_4_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let seat_5_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let seat_6_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        if game_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }

        let mut manager = Self::load_fee_manager(manager_account, program_id)?;
        let mut game = RussianRouletteGame::try_from_slice(&game_account.data.borrow())?;
        if game.round_id != expected_round_id {
            return Err(InvalidGameState.into());
        }

        if game.draw_status != RUSSIAN_ROULETTE_STATUS_PENDING {
            return Err(InvalidGameState.into());
        }
        if game.unlucky_player_index == UNLUCKY_PLAYER_INDEX_NONE {
            return Err(RussianRouletteDrawPending.into());
        }

        let player_accounts = [
            seat_1_account,
            seat_2_account,
            seat_3_account,
            seat_4_account,
            seat_5_account,
            seat_6_account,
        ];
        Self::validate_player_accounts(&game, &player_accounts)?;

        Self::settle_draw(
            game_account,
            manager_account,
            &player_accounts,
            &mut game,
            &mut manager,
        )?;

        game.serialize(&mut &mut game_account.data.borrow_mut()[..])?;
        manager.serialize(&mut &mut manager_account.data.borrow_mut()[..])?;

        let losing_seat = game.unlucky_player_index + 1;
        msg!(
            "roulette_result table_id={} round_id={} losing_seat={} losing_player={} stake_lamports={}",
            game.table_id,
            game.round_id,
            losing_seat,
            Pubkey::new_from_array(game.unlucky_player),
            game.stake
        );
        let survivor_count = (RUSSIAN_ROULETTE_PLAYER_COUNT - 1) as u64;
        let survivor_payout = game
            .stake
            .checked_add(
                game.stake
                    .checked_div(survivor_count)
                    .ok_or(ArithmeticError)?,
            )
            .ok_or(ArithmeticError)?;
        for (index, account) in player_accounts.iter().enumerate() {
            let is_loser = index == game.unlucky_player_index as usize;
            msg!(
                "roulette_player_result seat={} address={} outcome={} stake_lamports={} payout_lamports={}",
                index + 1,
                account.key,
                if is_loser { "loser" } else { "winner" },
                game.stake,
                if is_loser { 0 } else { survivor_payout }
            );
        }

        Ok(())
    }

    pub fn consume_vrf_randomness(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        expected_round_id: u64,
        expected_vrf_seed: [u8; 32],
        randomness: [u8; 32],
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let vrf_program_identity: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let game_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        MagicBlockVrf::validate_callback_identity(vrf_program_identity, program_id)?;
        if game_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }

        let mut game = RussianRouletteGame::try_from_slice(&game_account.data.borrow())?;
        Self::validate_game_account(game_account, &game, program_id)?;
        if game.round_id != expected_round_id || game.vrf_seed != expected_vrf_seed {
            return Err(InvalidGameState.into());
        }
        if game.draw_status != RUSSIAN_ROULETTE_STATUS_PENDING {
            return Err(InvalidGameState.into());
        }
        if game.unlucky_player_index != UNLUCKY_PLAYER_INDEX_NONE {
            return Err(RussianRouletteDrawAlreadyRequested.into());
        }

        let random_number = Self::randomness_number(&randomness);
        let unlucky_player_index = Self::unlucky_player_index(&randomness);
        game.vrf_seed = randomness;
        game.unlucky_player_index = unlucky_player_index;
        game.serialize(&mut &mut game_account.data.borrow_mut()[..])?;

        msg!(
            "roulette_vrf_result table_id={} round_id={} randomness={} random_u64={} losing_seat={}",
            game.table_id,
            game.round_id,
            Pubkey::new_from_array(randomness),
            random_number,
            unlucky_player_index + 1
        );

        Ok(())
    }

    pub fn reset_table_for_next_round(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        expected_round_id: u64,
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let admin: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let game_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let config_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        Utils::check_admin(admin, config_account, program_id)?;
        if game_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }

        let mut game = RussianRouletteGame::try_from_slice(&game_account.data.borrow())?;
        if game.round_id != expected_round_id || game.draw_status != RUSSIAN_ROULETTE_STATUS_DRAWN {
            return Err(InvalidGameState.into());
        }

        Self::clear_table_for_next_round(&mut game)?;
        game.serialize(&mut &mut game_account.data.borrow_mut()[..])?;

        Ok(())
    }

    fn collect_participation_payment<'a>(
        player: &AccountInfo<'a>,
        game_account: &AccountInfo<'a>,
        system_program_account: &AccountInfo<'a>,
        game: &RussianRouletteGame,
    ) -> ProgramResult {
        let amount = Self::participation_payment_amount(game)?;
        invoke(
            &transfer(player.key, game_account.key, amount),
            &[
                player.clone(),
                game_account.clone(),
                system_program_account.clone(),
            ],
        )
    }

    fn sit_player(game: &mut RussianRouletteGame, player: &Pubkey, seat: u8) -> ProgramResult {
        if game.draw_status != RUSSIAN_ROULETTE_STATUS_OPEN
            || game.number_of_players >= RUSSIAN_ROULETTE_PLAYER_COUNT
        {
            return Err(InvalidGameState.into());
        }
        if Self::player_has_joined(game, player) {
            return Err(InvalidPlayer.into());
        }

        let seat_slot = Self::seat_slot_mut(game, seat)?;
        if *seat_slot != [0; 32] {
            return Err(InvalidGameState.into());
        }

        *seat_slot = player.to_bytes();
        game.number_of_players = game
            .number_of_players
            .checked_add(1)
            .ok_or(ArithmeticError)?;

        Ok(())
    }

    fn settle_draw(
        game_account: &AccountInfo,
        manager_account: &AccountInfo,
        player_accounts: &[&AccountInfo; 6],
        game: &mut RussianRouletteGame,
        manager: &mut Manager,
    ) -> ProgramResult {
        if game.draw_status != RUSSIAN_ROULETTE_STATUS_PENDING {
            return Err(InvalidGameState.into());
        }
        if game.unlucky_player_index == UNLUCKY_PLAYER_INDEX_NONE {
            return Err(RussianRouletteDrawPending.into());
        }

        Self::validate_player_accounts(game, player_accounts)?;

        let unlucky_index = game.unlucky_player_index;
        let unlucky_account = player_accounts[unlucky_index as usize];
        let survivor_count = (RUSSIAN_ROULETTE_PLAYER_COUNT - 1) as u64;
        let survivor_bonus = game
            .stake
            .checked_div(survivor_count)
            .ok_or(ArithmeticError)?;
        let remainder = game
            .stake
            .checked_rem(survivor_count)
            .ok_or(ArithmeticError)?;
        let survivor_payout = game
            .stake
            .checked_add(survivor_bonus)
            .ok_or(ArithmeticError)?;
        let participation_fee_total = Self::participation_fee_total(game)?;
        let fee_total = participation_fee_total
            .checked_add(remainder)
            .ok_or(ArithmeticError)?;

        manager.collected_fee = manager
            .collected_fee
            .checked_add(fee_total)
            .ok_or(ArithmeticError)?;

        **game_account.try_borrow_mut_lamports()? -= fee_total;
        **manager_account.try_borrow_mut_lamports()? += fee_total;

        for (index, account) in player_accounts.iter().enumerate() {
            if index == unlucky_index as usize {
                continue;
            }

            **game_account.try_borrow_mut_lamports()? -= survivor_payout;
            **account.try_borrow_mut_lamports()? += survivor_payout;
        }

        game.unlucky_player_index = unlucky_index;
        game.unlucky_player = unlucky_account.key.to_bytes();
        game.draw_status = RUSSIAN_ROULETTE_STATUS_DRAWN;

        Ok(())
    }

    fn load_fee_manager(
        manager_account: &AccountInfo,
        program_id: &Pubkey,
    ) -> Result<Manager, ProgramError> {
        if manager_account.owner != program_id {
            return Err(InvalidManager.into());
        }
        let expected_manager = Pubkey::find_program_address(&[MANAGER_SEED], program_id).0;
        if manager_account.key != &expected_manager {
            return Err(InvalidManager.into());
        }

        let manager = Manager::try_from_slice(&manager_account.data.borrow())?;
        if manager.is_init != 1 {
            return Err(InvalidManager.into());
        }

        Ok(manager)
    }

    fn player_has_joined(game: &RussianRouletteGame, player: &Pubkey) -> bool {
        let player_bytes = player.to_bytes();
        Self::seat_bytes(game).contains(&player_bytes)
    }

    fn seat_bytes(game: &RussianRouletteGame) -> [[u8; 32]; 6] {
        [
            game.seat_1,
            game.seat_2,
            game.seat_3,
            game.seat_4,
            game.seat_5,
            game.seat_6,
        ]
    }

    fn seat_slot_mut(
        game: &mut RussianRouletteGame,
        seat: u8,
    ) -> Result<&mut [u8; 32], ProgramError> {
        match seat {
            1 => Ok(&mut game.seat_1),
            2 => Ok(&mut game.seat_2),
            3 => Ok(&mut game.seat_3),
            4 => Ok(&mut game.seat_4),
            5 => Ok(&mut game.seat_5),
            6 => Ok(&mut game.seat_6),
            _ => Err(InvalidPlayer.into()),
        }
    }

    fn validate_player_accounts(
        game: &RussianRouletteGame,
        player_accounts: &[&AccountInfo; 6],
    ) -> ProgramResult {
        let expected_players = Self::seat_bytes(game);

        for (index, account) in player_accounts.iter().enumerate() {
            if expected_players[index] == [0; 32]
                || account.key.to_bytes() != expected_players[index]
            {
                return Err(InvalidPlayer.into());
            }
        }

        Ok(())
    }

    fn validate_game_account(
        game_account: &AccountInfo,
        game: &RussianRouletteGame,
        program_id: &Pubkey,
    ) -> ProgramResult {
        if game_account.owner != program_id || game.table_id >= RUSSIAN_ROULETTE_TABLE_COUNT {
            return Err(InvalidGameAccount.into());
        }
        let table_id = [game.table_id];
        let expected_address =
            Pubkey::find_program_address(&[RUSSIAN_ROULETTE_SEED, &table_id], program_id).0;
        if game_account.key != &expected_address {
            return Err(InvalidGameAccount.into());
        }

        Ok(())
    }

    fn clear_table_for_next_round(game: &mut RussianRouletteGame) -> ProgramResult {
        game.round_id = game.round_id.checked_add(1).ok_or(ArithmeticError)?;
        game.draw_status = RUSSIAN_ROULETTE_STATUS_OPEN;
        game.number_of_players = 0;
        game.seat_1 = [0; 32];
        game.seat_2 = [0; 32];
        game.seat_3 = [0; 32];
        game.seat_4 = [0; 32];
        game.seat_5 = [0; 32];
        game.seat_6 = [0; 32];
        game.unlucky_player_index = UNLUCKY_PLAYER_INDEX_NONE;
        game.vrf_seed = [0; 32];
        game.unlucky_player = [0; 32];
        Ok(())
    }

    fn participation_payment_amount(game: &RussianRouletteGame) -> Result<u64, ProgramError> {
        game.stake
            .checked_add(game.program_fee)
            .ok_or(ArithmeticError.into())
    }

    fn participation_fee_total(game: &RussianRouletteGame) -> Result<u64, ProgramError> {
        game.program_fee
            .checked_mul(RUSSIAN_ROULETTE_PLAYER_COUNT as u64)
            .ok_or(ArithmeticError.into())
    }

    fn vrf_request_seed(
        game_address: &Pubkey,
        table_id: u8,
        round_id: u64,
        request_slot: u64,
    ) -> [u8; 32] {
        hash(
            &[
                RUSSIAN_ROULETTE_VRF_SEED,
                game_address.as_ref(),
                &[table_id],
                &round_id.to_le_bytes(),
                &request_slot.to_le_bytes(),
            ]
            .concat(),
        )
        .to_bytes()
    }

    fn retry_vrf_request_seed(
        game_address: &Pubkey,
        table_id: u8,
        round_id: u64,
        previous_seed: &[u8; 32],
        request_slot: u64,
    ) -> [u8; 32] {
        hash(
            &[
                RUSSIAN_ROULETTE_VRF_SEED,
                b"retry",
                game_address.as_ref(),
                &[table_id],
                &round_id.to_le_bytes(),
                previous_seed,
                &request_slot.to_le_bytes(),
            ]
            .concat(),
        )
        .to_bytes()
    }

    fn vrf_callback_args(round_id: u64, vrf_seed: &[u8; 32]) -> [u8; 40] {
        let mut callback_args = [0u8; 40];
        callback_args[..8].copy_from_slice(&round_id.to_le_bytes());
        callback_args[8..].copy_from_slice(vrf_seed);
        callback_args
    }

    fn unlucky_player_index(randomness: &[u8; 32]) -> u8 {
        (Self::randomness_number(randomness) % RUSSIAN_ROULETTE_PLAYER_COUNT as u64) as u8
    }

    fn randomness_number(randomness: &[u8; 32]) -> u64 {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&randomness[..8]);
        u64::from_le_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(program_fee: u64) -> RussianRouletteGame {
        RussianRouletteGame {
            table_id: 0,
            round_id: 9,
            draw_status: RUSSIAN_ROULETTE_STATUS_DRAWN,
            number_of_players: RUSSIAN_ROULETTE_PLAYER_COUNT,
            stake: 100_000_000,
            program_fee,
            seat_1: [1; 32],
            seat_2: [2; 32],
            seat_3: [3; 32],
            seat_4: [4; 32],
            seat_5: [5; 32],
            seat_6: [6; 32],
            unlucky_player_index: 2,
            vrf_seed: [7; 32],
            unlucky_player: [3; 32],
        }
    }

    #[test]
    fn configured_fee_drives_participation_payment_and_settlement_total() {
        let game = game(25_000_000);

        assert_eq!(
            RussianRoulette::participation_payment_amount(&game).unwrap(),
            125_000_000
        );
        assert_eq!(
            RussianRoulette::participation_fee_total(&game).unwrap(),
            150_000_000
        );
    }

    #[test]
    fn roulette_table_layout_remains_compatible() {
        assert_eq!(
            borsh::to_vec(&game(25_000_000)).unwrap().len(),
            crate::constants::RUSSIAN_ROULETTE_GAME_SPACE as usize
        );
    }

    #[test]
    fn initial_request_seed_changes_when_request_slot_changes() {
        let table_address = Pubkey::new_unique();
        let first = RussianRoulette::vrf_request_seed(&table_address, 0, 1, 1_000);
        let second = RussianRoulette::vrf_request_seed(&table_address, 0, 1, 1_001);

        assert_ne!(first, second);
    }

    #[test]
    fn callback_args_bind_round_and_request_seed() {
        let round_id = 17;
        let vrf_seed = [31u8; 32];
        let callback_args = RussianRoulette::vrf_callback_args(round_id, &vrf_seed);

        assert_eq!(&callback_args[..8], &round_id.to_le_bytes());
        assert_eq!(&callback_args[8..], &vrf_seed);
    }

    #[test]
    fn callback_validation_requires_the_exact_table_pda() {
        let program_id = Pubkey::new_unique();
        let game = game(25_000_000);
        let table_id = [game.table_id];
        let expected_address =
            Pubkey::find_program_address(&[RUSSIAN_ROULETTE_SEED, &table_id], &program_id).0;
        let wrong_address = Pubkey::new_unique();
        let mut expected_lamports = 0;
        let mut wrong_lamports = 0;
        let mut expected_data = [];
        let mut wrong_data = [];
        let expected_account = AccountInfo::new(
            &expected_address,
            false,
            true,
            &mut expected_lamports,
            &mut expected_data,
            &program_id,
            false,
        );
        let wrong_account = AccountInfo::new(
            &wrong_address,
            false,
            true,
            &mut wrong_lamports,
            &mut wrong_data,
            &program_id,
            false,
        );

        assert!(
            RussianRoulette::validate_game_account(&expected_account, &game, &program_id).is_ok()
        );
        assert_eq!(
            RussianRoulette::validate_game_account(&wrong_account, &game, &program_id).unwrap_err(),
            ProgramError::from(InvalidGameAccount)
        );
    }

    #[test]
    fn retries_use_unique_caller_seeds_without_rotating_callback_identity() {
        let table_address = Pubkey::new_unique();
        let request_seed = [37u8; 32];
        let first =
            RussianRoulette::retry_vrf_request_seed(&table_address, 1, 23, &request_seed, 2_000);
        let second =
            RussianRoulette::retry_vrf_request_seed(&table_address, 1, 23, &request_seed, 2_001);

        assert_ne!(first, second);
        let callback_args = RussianRoulette::vrf_callback_args(23, &request_seed);
        assert_eq!(&callback_args[8..], &request_seed);
        assert_ne!(&callback_args[8..], &first);
        assert_ne!(&callback_args[8..], &second);
    }

    #[test]
    fn stale_create_and_join_transactions_fail_before_collecting_payment() {
        let mut table = game(25_000_000);
        table.draw_status = RUSSIAN_ROULETTE_STATUS_OPEN;
        table.number_of_players = 0;
        table.seat_1 = [0; 32];
        table.seat_2 = [0; 32];
        table.seat_3 = [0; 32];
        table.seat_4 = [0; 32];
        table.seat_5 = [0; 32];
        table.seat_6 = [0; 32];

        let program_id = Pubkey::new_unique();
        let player_key = Pubkey::new_unique();
        let game_key = Pubkey::new_unique();
        let system_program_key = solana_system_interface::program::ID;
        let system_owner = solana_system_interface::program::ID;
        let starting_player_balance = 200_000_000;
        let mut player_lamports = starting_player_balance;
        let mut game_lamports = 2_000_000;
        let mut system_lamports = 0;
        let mut player_data = [];
        let mut game_data = borsh::to_vec(&table).unwrap();
        let mut system_data = [];
        let player_account = AccountInfo::new(
            &player_key,
            true,
            true,
            &mut player_lamports,
            &mut player_data,
            &system_owner,
            false,
        );
        let game_account = AccountInfo::new(
            &game_key,
            false,
            true,
            &mut game_lamports,
            &mut game_data,
            &program_id,
            false,
        );
        let system_program_account = AccountInfo::new(
            &system_program_key,
            false,
            false,
            &mut system_lamports,
            &mut system_data,
            &system_owner,
            true,
        );
        let accounts = [player_account, game_account, system_program_account];

        assert_eq!(
            RussianRoulette::create_game(
                &accounts,
                &program_id,
                InitRussianRoulette {
                    table_id: table.table_id,
                    seat: 1,
                    expected_round_id: table.round_id - 1,
                },
            )
            .unwrap_err(),
            ProgramError::from(InvalidGameState)
        );
        assert_eq!(
            RussianRoulette::join_game(
                &accounts,
                &program_id,
                JoinRussianRoulette {
                    seat: 1,
                    expected_round_id: table.round_id - 1,
                },
            )
            .unwrap_err(),
            ProgramError::from(InvalidGameState)
        );
        assert_eq!(
            **accounts[0].try_borrow_lamports().unwrap(),
            starting_player_balance
        );
        let expected_table_data = borsh::to_vec(&table).unwrap();
        assert_eq!(&**accounts[1].data.borrow(), expected_table_data.as_slice());
    }

    #[test]
    fn settlement_moves_the_configured_fee_and_preserves_the_table_balance() {
        let mut game = game(25_000_000);
        game.draw_status = RUSSIAN_ROULETTE_STATUS_PENDING;
        game.stake = 100_000_003;
        let mut manager = Manager {
            is_init: 1,
            allowed_time: 0,
            fee_percentage: 0,
            minimum_stake: 0,
            collected_fee: 11,
        };

        let program_id = Pubkey::new_unique();
        let system_owner = solana_system_interface::program::ID;
        let game_key = Pubkey::new_unique();
        let manager_key = Pubkey::new_unique();
        let player_1_key = Pubkey::new_from_array(game.seat_1);
        let player_2_key = Pubkey::new_from_array(game.seat_2);
        let player_3_key = Pubkey::new_from_array(game.seat_3);
        let player_4_key = Pubkey::new_from_array(game.seat_4);
        let player_5_key = Pubkey::new_from_array(game.seat_5);
        let player_6_key = Pubkey::new_from_array(game.seat_6);

        let table_balance = 2_000_000;
        let deposited_balance = (game.stake + game.program_fee) * 6;
        let manager_starting_balance = 3_000_000;
        let mut game_lamports = table_balance + deposited_balance;
        let mut manager_lamports = manager_starting_balance;
        let mut player_1_lamports = 0;
        let mut player_2_lamports = 0;
        let mut player_3_lamports = 0;
        let mut player_4_lamports = 0;
        let mut player_5_lamports = 0;
        let mut player_6_lamports = 0;
        let mut game_data = [];
        let mut manager_data = [];
        let mut player_1_data = [];
        let mut player_2_data = [];
        let mut player_3_data = [];
        let mut player_4_data = [];
        let mut player_5_data = [];
        let mut player_6_data = [];

        let game_account = AccountInfo::new(
            &game_key,
            false,
            true,
            &mut game_lamports,
            &mut game_data,
            &program_id,
            false,
        );
        let manager_account = AccountInfo::new(
            &manager_key,
            false,
            true,
            &mut manager_lamports,
            &mut manager_data,
            &program_id,
            false,
        );
        let player_1 = AccountInfo::new(
            &player_1_key,
            false,
            true,
            &mut player_1_lamports,
            &mut player_1_data,
            &system_owner,
            false,
        );
        let player_2 = AccountInfo::new(
            &player_2_key,
            false,
            true,
            &mut player_2_lamports,
            &mut player_2_data,
            &system_owner,
            false,
        );
        let player_3 = AccountInfo::new(
            &player_3_key,
            false,
            true,
            &mut player_3_lamports,
            &mut player_3_data,
            &system_owner,
            false,
        );
        let player_4 = AccountInfo::new(
            &player_4_key,
            false,
            true,
            &mut player_4_lamports,
            &mut player_4_data,
            &system_owner,
            false,
        );
        let player_5 = AccountInfo::new(
            &player_5_key,
            false,
            true,
            &mut player_5_lamports,
            &mut player_5_data,
            &system_owner,
            false,
        );
        let player_6 = AccountInfo::new(
            &player_6_key,
            false,
            true,
            &mut player_6_lamports,
            &mut player_6_data,
            &system_owner,
            false,
        );
        let player_accounts = [
            &player_1, &player_2, &player_3, &player_4, &player_5, &player_6,
        ];

        RussianRoulette::settle_draw(
            &game_account,
            &manager_account,
            &player_accounts,
            &mut game,
            &mut manager,
        )
        .unwrap();

        let remainder = game.stake % 5;
        let fee_total = game.program_fee * 6 + remainder;
        let survivor_payout = game.stake + game.stake / 5;
        assert_eq!(**game_account.try_borrow_lamports().unwrap(), table_balance);
        assert_eq!(
            **manager_account.try_borrow_lamports().unwrap(),
            manager_starting_balance + fee_total
        );
        assert_eq!(manager.collected_fee, 11 + fee_total);
        for (index, player) in player_accounts.iter().enumerate() {
            assert_eq!(
                **player.try_borrow_lamports().unwrap(),
                if index == 2 { 0 } else { survivor_payout }
            );
        }
        assert_eq!(game.draw_status, RUSSIAN_ROULETTE_STATUS_DRAWN);
        assert_eq!(game.unlucky_player, player_3_key.to_bytes());
    }

    #[test]
    fn resetting_a_table_preserves_its_configured_fee() {
        let mut game = game(25_000_000);

        RussianRoulette::clear_table_for_next_round(&mut game).unwrap();

        assert_eq!(game.round_id, 10);
        assert_eq!(game.program_fee, 25_000_000);
        assert_eq!(game.draw_status, RUSSIAN_ROULETTE_STATUS_OPEN);
        assert_eq!(game.number_of_players, 0);
    }
}
