use crate::{
    constants::{
        MANAGER_SEED, RUSSIAN_ROULETTE_PARTICIPATION_FEE, RUSSIAN_ROULETTE_PLAYER_COUNT,
        RUSSIAN_ROULETTE_STATUS_DRAWN, RUSSIAN_ROULETTE_STATUS_OPEN,
        RUSSIAN_ROULETTE_STATUS_PENDING, RUSSIAN_ROULETTE_VRF_CALLBACK_TAG,
        RUSSIAN_ROULETTE_VRF_SEED, UNLUCKY_PLAYER_INDEX_NONE,
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

        let vrf_seed = Self::vrf_request_seed(game_account.key, game.table_id, game.round_id);
        let callback_accounts = [(*game_account.key, false, true)];
        MagicBlockVrf::request_randomness_with_callback_accounts(
            RUSSIAN_ROULETTE_VRF_CALLBACK_TAG,
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

        if !payer.is_signer {
            return Err(PlayerNotSigner.into());
        }
        if game_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }

        let mut game = RussianRouletteGame::try_from_slice(&game_account.data.borrow())?;
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

        let vrf_seed = Self::retry_vrf_request_seed(
            game_account.key,
            game.table_id,
            game.round_id,
            &game.vrf_seed,
        )?;
        let callback_accounts = [(*game_account.key, false, true)];
        MagicBlockVrf::request_randomness_with_callback_accounts(
            RUSSIAN_ROULETTE_VRF_CALLBACK_TAG,
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

        game.unlucky_player_index = UNLUCKY_PLAYER_INDEX_NONE;
        game.vrf_seed = vrf_seed;
        game.serialize(&mut &mut game_account.data.borrow_mut()[..])?;

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
            .checked_add(game.stake.checked_div(survivor_count).ok_or(ArithmeticError)?)
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
        if game.round_id != expected_round_id
            || game.draw_status != RUSSIAN_ROULETTE_STATUS_DRAWN
        {
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
        let amount = game
            .stake
            .checked_add(RUSSIAN_ROULETTE_PARTICIPATION_FEE)
            .ok_or(ArithmeticError)?;
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
        let participation_fee_total = RUSSIAN_ROULETTE_PARTICIPATION_FEE
            .checked_mul(RUSSIAN_ROULETTE_PLAYER_COUNT as u64)
            .ok_or(ArithmeticError)?;
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
        game.program_fee = RUSSIAN_ROULETTE_PARTICIPATION_FEE;

        Ok(())
    }

    fn vrf_request_seed(game_address: &Pubkey, table_id: u8, round_id: u64) -> [u8; 32] {
        hash(
            &[
                RUSSIAN_ROULETTE_VRF_SEED,
                game_address.as_ref(),
                &[table_id],
                &round_id.to_le_bytes(),
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
    ) -> Result<[u8; 32], ProgramError> {
        let slot = Clock::get()?.slot;

        Ok(hash(
            &[
                RUSSIAN_ROULETTE_VRF_SEED,
                b"retry",
                game_address.as_ref(),
                &[table_id],
                &round_id.to_le_bytes(),
                previous_seed,
                &slot.to_le_bytes(),
            ]
            .concat(),
        )
        .to_bytes())
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
