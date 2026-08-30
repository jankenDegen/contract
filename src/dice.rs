use crate::{
    constants::{
        DICE_GAME_SPACE, DICE_MANAGER_SEED, DICE_PLAYER_COUNT, DICE_SEED, DICE_STATUS_DRAWN,
        DICE_STATUS_OPEN, DICE_STATUS_PENDING, DICE_STATUS_VRF_FAILED, DICE_VRF_CALLBACK_TAG,
        DICE_VRF_SEED, MANAGER_SEED, UNDRAWN_DICE_NO,
    },
    error::RPSProgramError::{
        ArithmeticError, DiceDrawAlreadyRequested, DiceDrawPending, DiceGameNotReady,
        InvalidChosenDice, InvalidDiceConfiguration, InvalidGameAccount, InvalidGameState,
        InvalidManager, InvalidPlayer, MinStake, PlayerNotSigner,
    },
    magicblock_vrf::MagicBlockVrf,
    state::{DiceGame, DiceManager, InitDice, JoinDice, Manager},
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
use solana_system_interface::{instruction::transfer, program as system_program};

pub struct Dice;
impl Dice {
    pub fn create_game(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        init_dice: InitDice,
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let player: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let dice_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let dice_manager_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let system_program_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        if !player.is_signer {
            return Err(PlayerNotSigner.into());
        }
        let selected_dices = Self::selected_dices_from_payload(&init_dice.chosen_dices)?;
        let selected_count = selected_dices.len() as u8;

        let dice_manager = Self::load_dice_manager(dice_manager_account, program_id)?;
        if init_dice.stake < dice_manager.minimum_stake {
            return Err(MinStake.into());
        }

        let seeds: &[&[u8]] = &[DICE_SEED, &init_dice.game_id.to_le_bytes()];

        Utils::create_pda(
            player,
            dice_account,
            system_program_account,
            seeds,
            DICE_GAME_SPACE,
            program_id,
        )?;

        let total_stake = init_dice
            .stake
            .checked_mul(selected_count as u64)
            .ok_or(ArithmeticError)?;
        let pot = init_dice
            .stake
            .checked_mul(DICE_PLAYER_COUNT as u64)
            .ok_or(ArithmeticError)?;
        if dice_manager.program_fee >= pot {
            return Err(InvalidDiceConfiguration.into());
        }

        invoke(
            &transfer(player.key, dice_account.key, total_stake),
            &[
                player.clone(),
                dice_account.clone(),
                system_program_account.clone(),
            ],
        )?;

        let mut chosen_dices = [0u8; 6];
        let mut players = [[0u8; 32]; 6];
        for (index, chosen_dice) in selected_dices.iter().enumerate() {
            chosen_dices[index] = *chosen_dice;
            players[index] = player.key.to_bytes();
        }

        let dice = DiceGame {
            game_id: init_dice.game_id,
            number_of_players: selected_count,
            stake: init_dice.stake,
            initializer: players[0],
            player_2: players[1],
            player_3: players[2],
            player_4: players[3],
            player_5: players[4],
            player_6: players[5],
            chosen_dices,
            draw_status: DICE_STATUS_OPEN,
            winning_dice: UNDRAWN_DICE_NO,
            vrf_seed: [0; 32],
            winner: [0; 32],
            program_fee: dice_manager.program_fee,
            vrf_last_request_at: 0,
            vrf_retry_count: 0,
        };

        dice.serialize(&mut &mut dice_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn join_game(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        join_dice: JoinDice,
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let player: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let dice_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let system_program_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        if !player.is_signer {
            return Err(PlayerNotSigner.into());
        }
        if dice_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }
        let selected_dices = Self::selected_dices_from_payload(&join_dice.chosen_dices)?;
        let selected_count = selected_dices.len() as u8;

        let mut dice = DiceGame::try_from_slice(&dice_account.data.borrow())?;
        Self::validate_game_account(dice_account, &dice, program_id)?;

        if dice.draw_status != DICE_STATUS_OPEN || dice.number_of_players >= DICE_PLAYER_COUNT {
            return Err(InvalidGameState.into());
        }
        if dice
            .number_of_players
            .checked_add(selected_count)
            .ok_or(ArithmeticError)?
            > DICE_PLAYER_COUNT
        {
            return Err(InvalidGameState.into());
        }
        for chosen_dice in selected_dices.iter() {
            if dice.chosen_dices.contains(chosen_dice) {
                return Err(InvalidChosenDice.into());
            }
        }

        let total_stake = dice
            .stake
            .checked_mul(selected_count as u64)
            .ok_or(ArithmeticError)?;

        invoke(
            &transfer(player.key, dice_account.key, total_stake),
            &[
                player.clone(),
                dice_account.clone(),
                system_program_account.clone(),
            ],
        )?;

        for chosen_dice in selected_dices {
            dice.number_of_players = dice
                .number_of_players
                .checked_add(1)
                .ok_or(ArithmeticError)?;

            let slot_index = (dice.number_of_players - 1) as usize;
            dice.chosen_dices[slot_index] = chosen_dice;

            match dice.number_of_players {
                1 => dice.initializer = player.key.to_bytes(),
                2 => dice.player_2 = player.key.to_bytes(),
                3 => dice.player_3 = player.key.to_bytes(),
                4 => dice.player_4 = player.key.to_bytes(),
                5 => dice.player_5 = player.key.to_bytes(),
                6 => dice.player_6 = player.key.to_bytes(),
                _ => return Err(InvalidGameState.into()),
            }
        }

        dice.serialize(&mut &mut dice_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn request_draw(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let payer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let dice_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let vrf_request_identity: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let oracle_queue: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let system_program_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let slot_hashes_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let vrf_program_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        if !payer.is_signer {
            return Err(PlayerNotSigner.into());
        }
        if dice_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }

        let mut dice = DiceGame::try_from_slice(&dice_account.data.borrow())?;
        Self::validate_game_account(dice_account, &dice, program_id)?;
        if dice.number_of_players != DICE_PLAYER_COUNT {
            return Err(DiceGameNotReady.into());
        }
        if dice.draw_status != DICE_STATUS_OPEN {
            return Err(DiceDrawAlreadyRequested.into());
        }

        let clock = Clock::get()?;
        let vrf_seed = Self::vrf_request_seed(dice_account.key, dice.game_id, clock.slot);
        MagicBlockVrf::request_randomness(
            DICE_VRF_CALLBACK_TAG,
            &vrf_seed,
            payer,
            vrf_request_identity,
            oracle_queue,
            system_program_account,
            slot_hashes_account,
            vrf_program_account,
            program_id,
            dice_account,
            vrf_seed,
        )?;

        dice.draw_status = DICE_STATUS_PENDING;
        dice.winning_dice = UNDRAWN_DICE_NO;
        dice.vrf_seed = vrf_seed;
        dice.vrf_last_request_at = clock.unix_timestamp;
        dice.vrf_retry_count = 0;
        dice.serialize(&mut &mut dice_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn finalize_draw(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let dice_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let manager_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let initializer_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let player_2_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let player_3_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let player_4_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let player_5_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let player_6_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        if dice_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }

        let mut manager = Self::load_fee_manager(manager_account, program_id)?;
        let mut dice = DiceGame::try_from_slice(&dice_account.data.borrow())?;
        Self::validate_game_account(dice_account, &dice, program_id)?;

        if dice.draw_status == DICE_STATUS_DRAWN {
            return Err(DiceDrawAlreadyRequested.into());
        }
        if dice.draw_status != DICE_STATUS_PENDING {
            return Err(InvalidGameState.into());
        }
        if dice.winning_dice == UNDRAWN_DICE_NO {
            return Err(DiceDrawPending.into());
        }

        let player_accounts = [
            initializer_account,
            player_2_account,
            player_3_account,
            player_4_account,
            player_5_account,
            player_6_account,
        ];
        Self::validate_player_accounts(&dice, &player_accounts)?;

        let winner_index = dice
            .chosen_dices
            .iter()
            .position(|chosen_dice| *chosen_dice == dice.winning_dice)
            .ok_or(InvalidGameState)?;
        let winner_account = player_accounts[winner_index];
        let pot = dice
            .stake
            .checked_mul(DICE_PLAYER_COUNT as u64)
            .ok_or(ArithmeticError)?;

        if dice.program_fee >= pot {
            return Err(InvalidDiceConfiguration.into());
        }

        let prize = pot.checked_sub(dice.program_fee).ok_or(ArithmeticError)?;

        manager.collected_fee = manager
            .collected_fee
            .checked_add(dice.program_fee)
            .ok_or(ArithmeticError)?;

        **dice_account.try_borrow_mut_lamports()? -= dice.program_fee;
        **manager_account.try_borrow_mut_lamports()? += dice.program_fee;

        **dice_account.try_borrow_mut_lamports()? -= prize;
        **winner_account.try_borrow_mut_lamports()? += prize;

        dice.draw_status = DICE_STATUS_DRAWN;
        dice.winner = winner_account.key.to_bytes();

        dice.serialize(&mut &mut dice_account.data.borrow_mut()[..])?;
        manager.serialize(&mut &mut manager_account.data.borrow_mut()[..])?;

        msg!(
            "dice_result game_id={} winning_face={} winner={} stake_lamports={} prize_lamports={}",
            dice.game_id,
            dice.winning_dice,
            winner_account.key,
            dice.stake,
            prize
        );
        for (index, account) in player_accounts.iter().enumerate() {
            let face = dice.chosen_dices[index];
            let outcome = if index == winner_index {
                "winner"
            } else {
                "loser"
            };
            let payout = if index == winner_index { prize } else { 0 };
            msg!(
                "dice_player_result face={} address={} outcome={} stake_lamports={} payout_lamports={}",
                face,
                account.key,
                outcome,
                dice.stake,
                payout
            );
        }

        Ok(())
    }

    pub fn consume_vrf_randomness(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        expected_vrf_seed: [u8; 32],
        randomness: [u8; 32],
    ) -> ProgramResult {
        Self::consume_vrf_randomness_inner(accounts, program_id, expected_vrf_seed, randomness)
    }

    fn consume_vrf_randomness_inner(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        expected_vrf_seed: [u8; 32],
        randomness: [u8; 32],
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let vrf_program_identity: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let dice_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        MagicBlockVrf::validate_callback_identity(vrf_program_identity, program_id)?;
        if dice_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }

        let mut dice = DiceGame::try_from_slice(&dice_account.data.borrow())?;
        Self::validate_game_account(dice_account, &dice, program_id)?;
        Self::validate_unresolved_pending(&dice)?;
        if dice.vrf_seed != expected_vrf_seed {
            return Err(InvalidGameState.into());
        }

        let random_number = Self::randomness_number(&randomness);
        let winning_dice = Self::winning_dice(&randomness);
        dice.vrf_seed = randomness;
        dice.winning_dice = winning_dice;
        dice.serialize(&mut &mut dice_account.data.borrow_mut()[..])?;

        msg!(
            "dice_vrf_result game_id={} randomness={} random_u64={} winning_face={}",
            dice.game_id,
            Pubkey::new_from_array(randomness),
            random_number,
            winning_dice
        );

        Ok(())
    }

    pub fn retry_draw(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let payer = next_account_info(accounts_iter)?;
        let dice_account = next_account_info(accounts_iter)?;
        let vrf_request_identity = next_account_info(accounts_iter)?;
        let oracle_queue = next_account_info(accounts_iter)?;
        let system_program_account = next_account_info(accounts_iter)?;
        let slot_hashes_account = next_account_info(accounts_iter)?;
        let vrf_program_account = next_account_info(accounts_iter)?;

        if !payer.is_signer {
            return Err(PlayerNotSigner.into());
        }
        if dice_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }
        let mut dice = DiceGame::try_from_slice(&dice_account.data.borrow())?;
        Self::validate_game_account(dice_account, &dice, program_id)?;
        Self::validate_unresolved_pending(&dice)?;
        let clock = Clock::get()?;
        let next_retry_count = Utils::next_vrf_retry_count(dice.vrf_retry_count)?;
        Utils::require_vrf_retry_delay(dice.vrf_last_request_at, clock.unix_timestamp)?;
        let caller_seed = Self::retry_vrf_request_seed(
            dice_account.key,
            dice.game_id,
            &dice.vrf_seed,
            next_retry_count,
            clock.slot,
        );
        MagicBlockVrf::request_randomness(
            DICE_VRF_CALLBACK_TAG,
            &dice.vrf_seed,
            payer,
            vrf_request_identity,
            oracle_queue,
            system_program_account,
            slot_hashes_account,
            vrf_program_account,
            program_id,
            dice_account,
            caller_seed,
        )?;

        dice.vrf_retry_count = next_retry_count;
        dice.vrf_last_request_at = clock.unix_timestamp;
        dice.serialize(&mut &mut dice_account.data.borrow_mut()[..])?;
        Ok(())
    }

    pub fn mark_vrf_failed(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let dice_account = next_account_info(&mut accounts.iter())?;
        if dice_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }
        let mut dice = DiceGame::try_from_slice(&dice_account.data.borrow())?;
        Self::validate_game_account(dice_account, &dice, program_id)?;
        Self::validate_unresolved_pending(&dice)?;
        Utils::require_vrf_failure_delay(
            dice.vrf_retry_count,
            dice.vrf_last_request_at,
            Clock::get()?.unix_timestamp,
        )?;

        dice.draw_status = DICE_STATUS_VRF_FAILED;
        dice.serialize(&mut &mut dice_account.data.borrow_mut()[..])?;
        Ok(())
    }

    pub fn refund_failed_game(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let dice_account = next_account_info(accounts_iter)?;
        let initializer_account = next_account_info(accounts_iter)?;
        let player_2_account = next_account_info(accounts_iter)?;
        let player_3_account = next_account_info(accounts_iter)?;
        let player_4_account = next_account_info(accounts_iter)?;
        let player_5_account = next_account_info(accounts_iter)?;
        let player_6_account = next_account_info(accounts_iter)?;

        if dice_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }
        let dice = DiceGame::try_from_slice(&dice_account.data.borrow())?;
        Self::validate_game_account(dice_account, &dice, program_id)?;
        if dice.draw_status != DICE_STATUS_VRF_FAILED
            || dice.number_of_players != DICE_PLAYER_COUNT
            || dice.winning_dice != UNDRAWN_DICE_NO
        {
            return Err(InvalidGameState.into());
        }
        let player_accounts = [
            initializer_account,
            player_2_account,
            player_3_account,
            player_4_account,
            player_5_account,
            player_6_account,
        ];
        Self::validate_player_accounts(&dice, &player_accounts)?;
        Self::settle_failed_refund(
            dice_account,
            initializer_account,
            &player_accounts,
            dice.stake,
        )?;
        dice_account.resize(0)?;
        dice_account.assign(&system_program::ID);
        Ok(())
    }

    fn validate_dice_choice(chosen_dice: u8) -> ProgramResult {
        if !(1..=6).contains(&chosen_dice) {
            return Err(InvalidChosenDice.into());
        }

        Ok(())
    }

    fn selected_dices_from_payload(chosen_dices: &[u8; 6]) -> Result<Vec<u8>, ProgramError> {
        let mut selected_dices = Vec::with_capacity(6);

        for chosen_dice in chosen_dices.iter().copied() {
            if chosen_dice == 0 {
                continue;
            }

            Self::validate_dice_choice(chosen_dice)?;
            if selected_dices.contains(&chosen_dice) {
                return Err(InvalidChosenDice.into());
            }

            selected_dices.push(chosen_dice);
        }

        if selected_dices.is_empty() {
            return Err(InvalidChosenDice.into());
        }

        Ok(selected_dices)
    }

    fn load_dice_manager(
        dice_manager_account: &AccountInfo,
        program_id: &Pubkey,
    ) -> Result<DiceManager, ProgramError> {
        if dice_manager_account.owner != program_id {
            return Err(InvalidManager.into());
        }
        let expected_manager = Pubkey::find_program_address(&[DICE_MANAGER_SEED], program_id).0;
        if dice_manager_account.key != &expected_manager {
            return Err(InvalidManager.into());
        }

        let dice_manager = DiceManager::try_from_slice(&dice_manager_account.data.borrow())?;
        if dice_manager.is_init != 1 {
            return Err(InvalidManager.into());
        }

        Ok(dice_manager)
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

    fn validate_player_accounts(
        dice: &DiceGame,
        player_accounts: &[&AccountInfo; 6],
    ) -> ProgramResult {
        let expected_players = [
            dice.initializer,
            dice.player_2,
            dice.player_3,
            dice.player_4,
            dice.player_5,
            dice.player_6,
        ];

        for (index, account) in player_accounts.iter().enumerate() {
            if account.key.to_bytes() != expected_players[index] {
                return Err(InvalidPlayer.into());
            }
        }

        Ok(())
    }

    fn validate_game_account(
        dice_account: &AccountInfo,
        dice: &DiceGame,
        program_id: &Pubkey,
    ) -> ProgramResult {
        if dice_account.owner != program_id {
            return Err(InvalidGameAccount.into());
        }
        let expected_address =
            Pubkey::find_program_address(&[DICE_SEED, &dice.game_id.to_le_bytes()], program_id).0;
        if dice_account.key != &expected_address {
            return Err(InvalidGameAccount.into());
        }

        Ok(())
    }

    fn vrf_request_seed(dice_address: &Pubkey, game_id: u64, request_slot: u64) -> [u8; 32] {
        let game_id_bytes = game_id.to_le_bytes();
        let request_slot_bytes = request_slot.to_le_bytes();
        hash(
            &[
                DICE_VRF_SEED,
                dice_address.as_ref(),
                &game_id_bytes,
                &request_slot_bytes,
            ]
            .concat(),
        )
        .to_bytes()
    }

    fn retry_vrf_request_seed(
        dice_address: &Pubkey,
        game_id: u64,
        stable_seed: &[u8; 32],
        retry_count: u8,
        request_slot: u64,
    ) -> [u8; 32] {
        hash(
            &[
                DICE_VRF_SEED,
                b"retry",
                dice_address.as_ref(),
                &game_id.to_le_bytes(),
                stable_seed,
                &[retry_count],
                &request_slot.to_le_bytes(),
            ]
            .concat(),
        )
        .to_bytes()
    }

    fn validate_unresolved_pending(dice: &DiceGame) -> ProgramResult {
        if dice.draw_status != DICE_STATUS_PENDING {
            return Err(InvalidGameState.into());
        }
        if dice.winning_dice != UNDRAWN_DICE_NO {
            return Err(DiceDrawAlreadyRequested.into());
        }
        Ok(())
    }

    fn move_lamports(
        source: &AccountInfo,
        destination: &AccountInfo,
        amount: u64,
    ) -> ProgramResult {
        let source_balance = **source.try_borrow_lamports()?;
        let destination_balance = **destination.try_borrow_lamports()?;
        **source.try_borrow_mut_lamports()? =
            source_balance.checked_sub(amount).ok_or(ArithmeticError)?;
        **destination.try_borrow_mut_lamports()? = destination_balance
            .checked_add(amount)
            .ok_or(ArithmeticError)?;
        Ok(())
    }

    fn settle_failed_refund(
        dice_account: &AccountInfo,
        initializer_account: &AccountInfo,
        player_accounts: &[&AccountInfo; 6],
        stake: u64,
    ) -> ProgramResult {
        for account in player_accounts {
            Self::move_lamports(dice_account, account, stake)?;
        }
        let rent_and_dust = **dice_account.try_borrow_lamports()?;
        Self::move_lamports(dice_account, initializer_account, rent_and_dust)
    }

    fn winning_dice(randomness: &[u8; 32]) -> u8 {
        (Self::randomness_number(randomness) % DICE_PLAYER_COUNT as u64) as u8 + 1
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

    fn game(game_id: u64) -> DiceGame {
        DiceGame {
            game_id,
            number_of_players: DICE_PLAYER_COUNT,
            stake: 100_000_000,
            initializer: [1; 32],
            player_2: [2; 32],
            player_3: [3; 32],
            player_4: [4; 32],
            player_5: [5; 32],
            player_6: [6; 32],
            chosen_dices: [1, 2, 3, 4, 5, 6],
            draw_status: DICE_STATUS_PENDING,
            winning_dice: UNDRAWN_DICE_NO,
            vrf_seed: [0; 32],
            winner: [0; 32],
            program_fee: 3_000_000,
            vrf_last_request_at: 1_000,
            vrf_retry_count: 0,
        }
    }

    #[test]
    fn request_seed_changes_when_request_slot_changes() {
        let dice_address = Pubkey::new_unique();
        let game_id: u64 = 42;
        let first = Dice::vrf_request_seed(&dice_address, game_id, 1_000);
        let second = Dice::vrf_request_seed(&dice_address, game_id, 1_001);

        assert_ne!(first, second);
    }

    #[test]
    fn retry_seeds_are_unique_but_keep_the_stable_callback_seed() {
        let dice_address = Pubkey::new_unique();
        let game_id = 42;
        let stable_seed = Dice::vrf_request_seed(&dice_address, game_id, 1_000);
        let first = Dice::retry_vrf_request_seed(&dice_address, game_id, &stable_seed, 1, 2_000);
        let second = Dice::retry_vrf_request_seed(&dice_address, game_id, &stable_seed, 2, 2_300);

        assert_ne!(first, second);
        assert_ne!(first, stable_seed);
        assert_ne!(second, stable_seed);
    }

    #[test]
    fn callback_validation_requires_the_exact_dice_pda() {
        let program_id = Pubkey::new_unique();
        let game = game(42);
        let game_id_bytes = game.game_id.to_le_bytes();
        let expected_address =
            Pubkey::find_program_address(&[DICE_SEED, &game_id_bytes], &program_id).0;
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

        assert!(Dice::validate_game_account(&expected_account, &game, &program_id).is_ok());
        assert_eq!(
            Dice::validate_game_account(&wrong_account, &game, &program_id).unwrap_err(),
            ProgramError::from(InvalidGameAccount)
        );
    }

    #[test]
    fn callback_and_failure_marker_share_the_same_unresolved_pending_gate() {
        let mut game = game(42);

        assert!(Dice::validate_unresolved_pending(&game).is_ok());

        // A callback that has already stored a result prevents failure marking.
        game.winning_dice = 4;
        assert!(Dice::validate_unresolved_pending(&game).is_err());

        // A failure marker that wins the race prevents a late callback.
        game.winning_dice = UNDRAWN_DICE_NO;
        game.draw_status = DICE_STATUS_VRF_FAILED;
        assert!(Dice::validate_unresolved_pending(&game).is_err());
    }

    #[test]
    fn failed_refund_returns_one_stake_per_face_even_with_duplicate_players() {
        let program_id = Pubkey::new_unique();
        let system_owner = system_program::ID;
        let dice_key = Pubkey::new_unique();
        let shared_key = Pubkey::new_unique();
        let player_3_key = Pubkey::new_unique();
        let player_4_key = Pubkey::new_unique();
        let player_5_key = Pubkey::new_unique();
        let player_6_key = Pubkey::new_unique();
        let stake = 100u64;
        let rent = 37u64;
        let mut dice_lamports = stake * 6 + rent;
        let mut shared_lamports = 10;
        let mut player_3_lamports = 10;
        let mut player_4_lamports = 10;
        let mut player_5_lamports = 10;
        let mut player_6_lamports = 10;
        let mut dice_data = [];
        let mut shared_data = [];
        let mut player_3_data = [];
        let mut player_4_data = [];
        let mut player_5_data = [];
        let mut player_6_data = [];
        let dice_account = AccountInfo::new(
            &dice_key,
            false,
            true,
            &mut dice_lamports,
            &mut dice_data,
            &program_id,
            false,
        );
        let shared_player = AccountInfo::new(
            &shared_key,
            false,
            true,
            &mut shared_lamports,
            &mut shared_data,
            &system_owner,
            false,
        );
        let duplicate_shared_player = shared_player.clone();
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
        let players = [
            &shared_player,
            &duplicate_shared_player,
            &player_3,
            &player_4,
            &player_5,
            &player_6,
        ];

        Dice::settle_failed_refund(&dice_account, &shared_player, &players, stake).unwrap();

        assert_eq!(**dice_account.try_borrow_lamports().unwrap(), 0);
        assert_eq!(
            **shared_player.try_borrow_lamports().unwrap(),
            10 + stake * 2 + rent
        );
        for player in [&player_3, &player_4, &player_5, &player_6] {
            assert_eq!(**player.try_borrow_lamports().unwrap(), 10 + stake);
        }
    }
}
