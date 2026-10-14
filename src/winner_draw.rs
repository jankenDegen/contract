use crate::{
    constants::{
        WINNER_DRAW_MAX_WINNERS, WINNER_DRAW_SELECTION_DOMAIN, WINNER_DRAW_VRF_CALLBACK_TAG,
        WINNER_DRAW_VRF_SEED,
    },
    error::RPSProgramError::{InvalidInstruction, InvalidWinnerDrawConfiguration, PlayerNotSigner},
    magicblock_vrf::MagicBlockVrf,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    hash::hash,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

pub struct WinnerDraw;

#[derive(Debug, PartialEq)]
struct WinnerDrawVrfCallbackData {
    number_of_winners: u32,
    number_of_participants: u32,
    users_list_hash: [u8; 32],
    expected_vrf_seed: [u8; 32],
    randomness: [u8; 32],
}

impl WinnerDraw {
    pub fn process(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        instruction_data: &[u8],
    ) -> ProgramResult {
        let (tag, rest) = instruction_data.split_first().ok_or(InvalidInstruction)?;
        match tag {
            50 => {
                let (number_of_winners, number_of_participants, users_list_hash) =
                    Self::unpack_request(rest)?;
                Self::request(
                    accounts,
                    program_id,
                    number_of_winners,
                    number_of_participants,
                    users_list_hash,
                )
            }
            93 => {
                let callback = Self::unpack_callback(rest)?;
                Self::consume_vrf_randomness(accounts, program_id, &callback)
            }
            _ => Err(InvalidInstruction.into()),
        }
    }

    /// Requests a generic, account-free MagicBlock VRF draw.
    ///
    /// Participant numbers are one-based (`1..=number_of_participants`). The
    /// callback logs a uniformly selected set of unique winning numbers. No
    /// program-owned account or rent allocation is required.
    pub fn request(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        number_of_winners: u32,
        number_of_participants: u32,
        users_list_hash: [u8; 32],
    ) -> ProgramResult {
        Self::validate_counts(number_of_winners, number_of_participants)?;

        let accounts_iter = &mut accounts.iter();
        let payer = next_account_info(accounts_iter)?;
        let vrf_request_identity = next_account_info(accounts_iter)?;
        let oracle_queue = next_account_info(accounts_iter)?;
        let system_program_account = next_account_info(accounts_iter)?;
        let slot_hashes_account = next_account_info(accounts_iter)?;
        let vrf_program_account = next_account_info(accounts_iter)?;

        if !payer.is_signer {
            return Err(PlayerNotSigner.into());
        }

        let request_slot = Clock::get()?.slot;
        let request_seed = Self::request_seed(
            payer.key,
            number_of_winners,
            number_of_participants,
            request_slot,
        );
        let callback_args = Self::callback_args(
            number_of_winners,
            number_of_participants,
            &users_list_hash,
            &request_seed,
        );

        MagicBlockVrf::request_randomness_with_callback_accounts(
            WINNER_DRAW_VRF_CALLBACK_TAG,
            &callback_args,
            payer,
            vrf_request_identity,
            oracle_queue,
            system_program_account,
            slot_hashes_account,
            vrf_program_account,
            program_id,
            &[],
            request_seed,
        )?;

        msg!(
            "winner_draw_requested request_seed={} users_list_hash={} number_of_participants={} number_of_winners={}",
            Pubkey::new_from_array(request_seed),
            Pubkey::new_from_array(users_list_hash),
            number_of_participants,
            number_of_winners,
        );

        Ok(())
    }

    fn consume_vrf_randomness(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        callback: &WinnerDrawVrfCallbackData,
    ) -> ProgramResult {
        Self::validate_counts(callback.number_of_winners, callback.number_of_participants)?;

        let vrf_program_identity = next_account_info(&mut accounts.iter())?;
        MagicBlockVrf::validate_callback_identity(vrf_program_identity, program_id)?;

        let winning_numbers = Self::winning_numbers(
            callback.number_of_winners,
            callback.number_of_participants,
            &callback.expected_vrf_seed,
            &callback.randomness,
        )?;

        msg!(
            "winner_draw_result request_seed={} randomness={} users_list_hash={} number_of_participants={} number_of_winners={} winning_numbers={:?}",
            Pubkey::new_from_array(callback.expected_vrf_seed),
            Pubkey::new_from_array(callback.randomness),
            Pubkey::new_from_array(callback.users_list_hash),
            callback.number_of_participants,
            callback.number_of_winners,
            winning_numbers,
        );

        Ok(())
    }

    fn validate_counts(number_of_winners: u32, number_of_participants: u32) -> ProgramResult {
        if number_of_winners == 0
            || number_of_participants == 0
            || number_of_winners > number_of_participants
            || number_of_winners > WINNER_DRAW_MAX_WINNERS
        {
            return Err(InvalidWinnerDrawConfiguration.into());
        }
        Ok(())
    }

    fn unpack_request(input: &[u8]) -> Result<(u32, u32, [u8; 32]), ProgramError> {
        if input.len() != 40 {
            return Err(InvalidInstruction.into());
        }

        let mut winner_count_bytes = [0u8; 4];
        winner_count_bytes.copy_from_slice(&input[..4]);
        let mut participant_count_bytes = [0u8; 4];
        participant_count_bytes.copy_from_slice(&input[4..8]);
        let mut users_list_hash = [0u8; 32];
        users_list_hash.copy_from_slice(&input[8..]);
        Ok((
            u32::from_le_bytes(winner_count_bytes),
            u32::from_le_bytes(participant_count_bytes),
            users_list_hash,
        ))
    }

    fn unpack_callback(input: &[u8]) -> Result<WinnerDrawVrfCallbackData, ProgramError> {
        if input.len() != 104 {
            return Err(InvalidInstruction.into());
        }

        let mut randomness = [0u8; 32];
        randomness.copy_from_slice(&input[..32]);
        let (number_of_winners, number_of_participants, users_list_hash) =
            Self::unpack_request(&input[32..72])?;
        let mut expected_vrf_seed = [0u8; 32];
        expected_vrf_seed.copy_from_slice(&input[72..]);
        Ok(WinnerDrawVrfCallbackData {
            number_of_winners,
            number_of_participants,
            users_list_hash,
            expected_vrf_seed,
            randomness,
        })
    }

    fn request_seed(
        payer: &Pubkey,
        number_of_winners: u32,
        number_of_participants: u32,
        request_slot: u64,
    ) -> [u8; 32] {
        hash(
            &[
                WINNER_DRAW_VRF_SEED,
                payer.as_ref(),
                &number_of_winners.to_le_bytes(),
                &number_of_participants.to_le_bytes(),
                &request_slot.to_le_bytes(),
            ]
            .concat(),
        )
        .to_bytes()
    }

    fn callback_args(
        number_of_winners: u32,
        number_of_participants: u32,
        users_list_hash: &[u8; 32],
        request_seed: &[u8; 32],
    ) -> [u8; 72] {
        let mut callback_args = [0u8; 72];
        callback_args[..4].copy_from_slice(&number_of_winners.to_le_bytes());
        callback_args[4..8].copy_from_slice(&number_of_participants.to_le_bytes());
        callback_args[8..40].copy_from_slice(users_list_hash);
        callback_args[40..].copy_from_slice(request_seed);
        callback_args
    }

    /// Floyd's sampling algorithm selects a uniform subset without creating a
    /// participant-sized allocation. Keeping the selected set sorted makes
    /// duplicate checks efficient and the emitted result stable for clients.
    fn winning_numbers(
        number_of_winners: u32,
        number_of_participants: u32,
        request_seed: &[u8; 32],
        randomness: &[u8; 32],
    ) -> Result<Vec<u32>, solana_program::program_error::ProgramError> {
        Self::validate_counts(number_of_winners, number_of_participants)?;

        let mut winners = Vec::with_capacity(number_of_winners as usize);
        let first_range_end = number_of_participants - number_of_winners;
        for (draw_index, range_end) in (first_range_end..number_of_participants).enumerate() {
            let candidate =
                Self::uniform_below(range_end + 1, draw_index as u32, request_seed, randomness);
            match winners.binary_search(&candidate) {
                Ok(_) => winners.push(range_end),
                Err(insertion_index) => winners.insert(insertion_index, candidate),
            }
        }

        for winner in &mut winners {
            *winner += 1;
        }
        Ok(winners)
    }

    /// Rejection sampling avoids modulo bias for every nonzero u32 bound.
    fn uniform_below(
        upper_bound: u32,
        draw_index: u32,
        request_seed: &[u8; 32],
        randomness: &[u8; 32],
    ) -> u32 {
        let bound = u64::from(upper_bound);
        let rejection_threshold = bound.wrapping_neg() % bound;
        let mut attempt = 0u32;

        loop {
            let digest = hash(
                &[
                    WINNER_DRAW_SELECTION_DOMAIN,
                    request_seed,
                    randomness,
                    &draw_index.to_le_bytes(),
                    &attempt.to_le_bytes(),
                ]
                .concat(),
            )
            .to_bytes();
            let sample = u64::from_le_bytes(digest[..8].try_into().unwrap());
            if sample >= rejection_threshold {
                return (sample % bound) as u32;
            }
            attempt = attempt.wrapping_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::RPSProgramError;
    use solana_program::program_error::ProgramError;

    #[test]
    fn count_validation_rejects_empty_inverted_and_unbounded_draws() {
        for (winners, participants) in [
            (0, 1),
            (1, 0),
            (2, 1),
            (WINNER_DRAW_MAX_WINNERS + 1, WINNER_DRAW_MAX_WINNERS + 1),
        ] {
            assert_eq!(
                WinnerDraw::validate_counts(winners, participants).unwrap_err(),
                ProgramError::from(RPSProgramError::InvalidWinnerDrawConfiguration)
            );
        }
        WinnerDraw::validate_counts(1, 1).unwrap();
        WinnerDraw::validate_counts(WINNER_DRAW_MAX_WINNERS, u32::MAX).unwrap();
    }

    #[test]
    fn selection_is_deterministic_unique_sorted_and_one_based() {
        let winners = WinnerDraw::winning_numbers(5, 100, &[7; 32], &[11; 32]).unwrap();
        assert_eq!(winners, vec![5, 14, 18, 50, 88]);
        assert_eq!(winners.len(), 5);
        assert!(winners.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(winners.iter().all(|winner| (1..=100).contains(winner)));
        assert_eq!(
            winners,
            WinnerDraw::winning_numbers(5, 100, &[7; 32], &[11; 32]).unwrap()
        );
    }

    #[test]
    fn every_participant_wins_when_counts_match() {
        assert_eq!(
            WinnerDraw::winning_numbers(6, 6, &[3; 32], &[5; 32]).unwrap(),
            vec![1, 2, 3, 4, 5, 6]
        );
    }

    #[test]
    fn maximum_draw_stays_unique_and_in_u32_participant_range() {
        let winners =
            WinnerDraw::winning_numbers(WINNER_DRAW_MAX_WINNERS, u32::MAX, &[13; 32], &[17; 32])
                .unwrap();
        assert_eq!(winners.len(), WINNER_DRAW_MAX_WINNERS as usize);
        assert!(winners.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(winners.iter().all(|winner| *winner >= 1));
    }

    #[test]
    fn request_seed_binds_payer_counts_and_slot() {
        let payer = Pubkey::new_unique();
        let baseline = WinnerDraw::request_seed(&payer, 3, 20, 500);
        assert_ne!(
            baseline,
            WinnerDraw::request_seed(&Pubkey::new_unique(), 3, 20, 500)
        );
        assert_ne!(baseline, WinnerDraw::request_seed(&payer, 2, 20, 500));
        assert_ne!(baseline, WinnerDraw::request_seed(&payer, 3, 21, 500));
        assert_ne!(baseline, WinnerDraw::request_seed(&payer, 3, 20, 501));
    }

    #[test]
    fn callback_args_bind_counts_and_request_seed() {
        let request_seed = [29; 32];
        let users_list_hash = [31; 32];
        let args = WinnerDraw::callback_args(4, 50, &users_list_hash, &request_seed);
        assert_eq!(&args[..4], &4u32.to_le_bytes());
        assert_eq!(&args[4..8], &50u32.to_le_bytes());
        assert_eq!(&args[8..40], &users_list_hash);
        assert_eq!(&args[40..], &request_seed);
    }

    #[test]
    fn request_and_callback_decoders_require_exact_payloads() {
        let number_of_winners = 7u32;
        let number_of_participants = 50_000u32;
        let users_list_hash = [31u8; 32];
        let mut request = Vec::new();
        request.extend_from_slice(&number_of_winners.to_le_bytes());
        request.extend_from_slice(&number_of_participants.to_le_bytes());
        request.extend_from_slice(&users_list_hash);
        assert_eq!(
            WinnerDraw::unpack_request(&request).unwrap(),
            (number_of_winners, number_of_participants, users_list_hash)
        );
        assert!(WinnerDraw::unpack_request(&request[..request.len() - 1]).is_err());
        request.push(0);
        assert!(WinnerDraw::unpack_request(&request).is_err());

        let randomness = [23u8; 32];
        let expected_vrf_seed = [29u8; 32];
        let mut callback = Vec::new();
        callback.extend_from_slice(&randomness);
        callback.extend_from_slice(&number_of_winners.to_le_bytes());
        callback.extend_from_slice(&number_of_participants.to_le_bytes());
        callback.extend_from_slice(&users_list_hash);
        callback.extend_from_slice(&expected_vrf_seed);
        assert_eq!(
            WinnerDraw::unpack_callback(&callback).unwrap(),
            WinnerDrawVrfCallbackData {
                number_of_winners,
                number_of_participants,
                users_list_hash,
                expected_vrf_seed,
                randomness,
            }
        );
        assert!(WinnerDraw::unpack_callback(&callback[..callback.len() - 1]).is_err());
        callback.push(0);
        assert!(WinnerDraw::unpack_callback(&callback).is_err());
    }
}
