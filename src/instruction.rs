use crate::{
    error::RPSProgramError::InvalidInstruction,
    state::{
        Buy, DiceManager, InitDice, InitGame, InitRussianRoulette, InitRussianRouletteTable,
        JoinDice, JoinGame, JoinRussianRoulette, Manager, RaffleManager, Reveal,
        UpdateRussianRouletteParticipationFee, UpdateRussianRouletteTable,
    },
};
use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

#[derive(Debug, PartialEq)]
pub enum RPSProgramInstruction {
    InitGame {
        init_game: InitGame,
    },
    JoinGame {
        join_game: JoinGame,
    },
    Reveal {
        reveal: Reveal,
    },
    TimeIsUp,
    SetConfig,
    InitAccounts,
    SetManager {
        new_manager: Manager,
    },
    CollectFee,
    SetRaffleManager {
        new_manager: RaffleManager,
    },
    CreateRaffle,
    BuyTicket {
        buy: Buy,
    },
    RequestRaffleDraw,
    FinalizeRaffleDraw,
    RaffleVrfCallback {
        expected_vrf_seed: [u8; 32],
        randomness: [u8; 32],
    },
    ClaimPrize,
    CloseRaffle,
    SetDiceManager {
        new_manager: DiceManager,
    },
    InitDiceManager,
    CloseDiceGame,
    SetRussianRouletteTable {
        new_table: UpdateRussianRouletteTable,
    },
    InitRussianRouletteTable {
        init_table: InitRussianRouletteTable,
    },
    CreateRussianRouletteGame {
        init_game: InitRussianRoulette,
    },
    JoinRussianRouletteGame {
        join_game: JoinRussianRoulette,
    },
    RequestRussianRouletteDraw {
        expected_round_id: u64,
    },
    FinalizeRussianRouletteDraw {
        expected_round_id: u64,
    },
    RussianRouletteVrfCallback {
        expected_round_id: u64,
        expected_vrf_seed: [u8; 32],
        randomness: [u8; 32],
    },
    CloseRussianRouletteGame,
    ResetRussianRouletteTable {
        expected_round_id: u64,
    },
    RetryRussianRouletteDraw {
        expected_round_id: u64,
    },
    SetRussianRouletteParticipationFee {
        update: UpdateRussianRouletteParticipationFee,
    },
    RetryRaffleDraw,
    MarkRaffleVrfFailed,
    RefundFailedRaffleTicket,
    MarkRussianRouletteVrfFailed {
        expected_round_id: u64,
    },
    RefundFailedRussianRouletteRound {
        expected_round_id: u64,
    },
    CreateDiceGame {
        init_dice: InitDice,
    },
    JoinDiceGame {
        join_dice: JoinDice,
    },
    RequestDiceDraw {
        expected_generation_nonce: [u8; 32],
    },
    FinalizeDiceDraw {
        expected_generation_nonce: [u8; 32],
    },
    RetryDiceDraw {
        expected_generation_nonce: [u8; 32],
    },
    MarkDiceVrfFailed {
        expected_generation_nonce: [u8; 32],
    },
    RefundFailedDiceGame {
        expected_generation_nonce: [u8; 32],
    },
    DiceVrfCallback {
        expected_vrf_seed: [u8; 32],
        randomness: [u8; 32],
    },
}

impl RPSProgramInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => Self::InitGame {
                init_game: InitGame::try_from_slice(&rest)?,
            },
            1 => Self::JoinGame {
                join_game: JoinGame::try_from_slice(&rest)?,
            },
            2 => Self::Reveal {
                reveal: Reveal::try_from_slice(&rest)?,
            },

            4 => Self::TimeIsUp,
            7 => Self::SetConfig,
            8 => Self::InitAccounts,
            9 => Self::SetManager {
                new_manager: Manager::try_from_slice(&rest)?,
            },
            10 => Self::CollectFee,
            11 => Self::SetRaffleManager {
                new_manager: RaffleManager::try_from_slice(&rest)?,
            },
            12 => Self::CreateRaffle,
            13 => Self::BuyTicket {
                buy: Buy::try_from_slice(&rest)?,
            },
            14 => Self::RequestRaffleDraw,
            15 => Self::FinalizeRaffleDraw,
            16 => Self::ClaimPrize,
            17 => Self::CloseRaffle,
            18 => Self::SetDiceManager {
                new_manager: DiceManager::try_from_slice(&rest)?,
            },
            19 => Self::InitDiceManager,
            24 => {
                Self::require_empty(rest)?;
                Self::CloseDiceGame
            }
            25 => Self::SetRussianRouletteTable {
                new_table: UpdateRussianRouletteTable::try_from_slice(&rest)?,
            },
            26 => Self::InitRussianRouletteTable {
                init_table: InitRussianRouletteTable::try_from_slice(&rest)?,
            },
            27 => Self::CreateRussianRouletteGame {
                init_game: InitRussianRoulette::try_from_slice(&rest)?,
            },
            28 => Self::JoinRussianRouletteGame {
                join_game: JoinRussianRoulette::try_from_slice(&rest)?,
            },
            29 => Self::RequestRussianRouletteDraw {
                expected_round_id: u64::try_from_slice(rest)?,
            },
            30 => Self::FinalizeRussianRouletteDraw {
                expected_round_id: u64::try_from_slice(rest)?,
            },
            31 => Self::CloseRussianRouletteGame,
            32 => Self::ResetRussianRouletteTable {
                expected_round_id: u64::try_from_slice(rest)?,
            },
            33 => Self::RetryRussianRouletteDraw {
                expected_round_id: u64::try_from_slice(rest)?,
            },
            34 => Self::SetRussianRouletteParticipationFee {
                update: UpdateRussianRouletteParticipationFee::try_from_slice(rest)?,
            },
            35 => {
                Self::require_empty(rest)?;
                Self::RetryRaffleDraw
            }
            36 => {
                Self::require_empty(rest)?;
                Self::MarkRaffleVrfFailed
            }
            37 => {
                Self::require_empty(rest)?;
                Self::RefundFailedRaffleTicket
            }
            41 => Self::MarkRussianRouletteVrfFailed {
                expected_round_id: u64::try_from_slice(rest)?,
            },
            42 => Self::RefundFailedRussianRouletteRound {
                expected_round_id: u64::try_from_slice(rest)?,
            },
            43 => Self::CreateDiceGame {
                init_dice: InitDice::try_from_slice(rest)?,
            },
            44 => Self::JoinDiceGame {
                join_dice: JoinDice::try_from_slice(rest)?,
            },
            45 => Self::RequestDiceDraw {
                expected_generation_nonce: <[u8; 32]>::try_from_slice(rest)?,
            },
            46 => Self::FinalizeDiceDraw {
                expected_generation_nonce: <[u8; 32]>::try_from_slice(rest)?,
            },
            47 => Self::RetryDiceDraw {
                expected_generation_nonce: <[u8; 32]>::try_from_slice(rest)?,
            },
            48 => Self::MarkDiceVrfFailed {
                expected_generation_nonce: <[u8; 32]>::try_from_slice(rest)?,
            },
            49 => Self::RefundFailedDiceGame {
                expected_generation_nonce: <[u8; 32]>::try_from_slice(rest)?,
            },
            90 => {
                let (expected_vrf_seed, randomness) = Self::unpack_seeded_randomness(rest)?;
                Self::RaffleVrfCallback {
                    expected_vrf_seed,
                    randomness,
                }
            }
            91 => {
                let (expected_vrf_seed, randomness) = Self::unpack_seeded_randomness(rest)?;
                Self::DiceVrfCallback {
                    expected_vrf_seed,
                    randomness,
                }
            }
            92 => {
                let (expected_round_id, expected_vrf_seed, randomness) =
                    Self::unpack_roulette_randomness(rest)?;
                Self::RussianRouletteVrfCallback {
                    expected_round_id,
                    expected_vrf_seed,
                    randomness,
                }
            }

            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn require_empty(input: &[u8]) -> Result<(), ProgramError> {
        if input.is_empty() {
            Ok(())
        } else {
            Err(InvalidInstruction.into())
        }
    }

    fn unpack_seeded_randomness(input: &[u8]) -> Result<([u8; 32], [u8; 32]), ProgramError> {
        if input.len() != 64 {
            return Err(InvalidInstruction.into());
        }

        let mut randomness = [0u8; 32];
        randomness.copy_from_slice(&input[..32]);
        let mut expected_vrf_seed = [0u8; 32];
        expected_vrf_seed.copy_from_slice(&input[32..]);
        Ok((expected_vrf_seed, randomness))
    }

    fn unpack_roulette_randomness(input: &[u8]) -> Result<(u64, [u8; 32], [u8; 32]), ProgramError> {
        if input.len() != 72 {
            return Err(InvalidInstruction.into());
        }

        let mut randomness = [0u8; 32];
        randomness.copy_from_slice(&input[..32]);
        let mut round_id_bytes = [0u8; 8];
        round_id_bytes.copy_from_slice(&input[32..40]);
        let mut expected_vrf_seed = [0u8; 32];
        expected_vrf_seed.copy_from_slice(&input[40..]);
        Ok((
            u64::from_le_bytes(round_id_bytes),
            expected_vrf_seed,
            randomness,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh::BorshSerialize;

    fn round_instruction(tag: u8, round_id: u64) -> Vec<u8> {
        let mut data = vec![tag];
        data.extend_from_slice(&round_id.to_le_bytes());
        data
    }

    #[test]
    fn roulette_lifecycle_instructions_are_bound_to_an_expected_round() {
        let round_id = 42;

        assert_eq!(
            RPSProgramInstruction::unpack(&round_instruction(29, round_id)).unwrap(),
            RPSProgramInstruction::RequestRussianRouletteDraw {
                expected_round_id: round_id,
            }
        );
        assert_eq!(
            RPSProgramInstruction::unpack(&round_instruction(30, round_id)).unwrap(),
            RPSProgramInstruction::FinalizeRussianRouletteDraw {
                expected_round_id: round_id,
            }
        );
        assert_eq!(
            RPSProgramInstruction::unpack(&round_instruction(32, round_id)).unwrap(),
            RPSProgramInstruction::ResetRussianRouletteTable {
                expected_round_id: round_id,
            }
        );
        assert_eq!(
            RPSProgramInstruction::unpack(&round_instruction(33, round_id)).unwrap(),
            RPSProgramInstruction::RetryRussianRouletteDraw {
                expected_round_id: round_id,
            }
        );
    }

    #[test]
    fn roulette_lifecycle_instructions_reject_missing_round_ids() {
        for tag in [29, 30, 32, 33] {
            assert!(RPSProgramInstruction::unpack(&[tag]).is_err());
        }
    }

    #[test]
    fn roulette_participation_fee_instruction_decodes_expected_round_and_fee() {
        let update = UpdateRussianRouletteParticipationFee {
            expected_round_id: 74,
            program_fee: 25_000_000,
        };
        let mut data = vec![34];
        update.serialize(&mut data).unwrap();

        assert_eq!(
            RPSProgramInstruction::unpack(&data).unwrap(),
            RPSProgramInstruction::SetRussianRouletteParticipationFee { update }
        );
        assert!(RPSProgramInstruction::unpack(&data[..data.len() - 1]).is_err());
        data.push(0);
        assert!(RPSProgramInstruction::unpack(&data).is_err());
    }

    #[test]
    fn roulette_seat_instructions_require_and_decode_the_expected_round() {
        let round_id = 73;
        let init = InitRussianRoulette {
            table_id: 2,
            seat: 4,
            expected_round_id: round_id,
        };
        let join = JoinRussianRoulette {
            seat: 5,
            expected_round_id: round_id,
        };
        let mut init_data = vec![27];
        init.serialize(&mut init_data).unwrap();
        let mut join_data = vec![28];
        join.serialize(&mut join_data).unwrap();

        assert_eq!(
            RPSProgramInstruction::unpack(&init_data).unwrap(),
            RPSProgramInstruction::CreateRussianRouletteGame { init_game: init }
        );
        assert_eq!(
            RPSProgramInstruction::unpack(&join_data).unwrap(),
            RPSProgramInstruction::JoinRussianRouletteGame { join_game: join }
        );
        assert!(RPSProgramInstruction::unpack(&[27, 2, 4]).is_err());
        assert!(RPSProgramInstruction::unpack(&[28, 5]).is_err());
    }

    #[test]
    fn dice_vrf_callback_requires_the_exact_request_seed() {
        let expected_vrf_seed = [7u8; 32];
        let randomness = [9u8; 32];
        let mut data = vec![91];
        data.extend_from_slice(&randomness);
        data.extend_from_slice(&expected_vrf_seed);

        assert_eq!(
            RPSProgramInstruction::unpack(&data).unwrap(),
            RPSProgramInstruction::DiceVrfCallback {
                expected_vrf_seed,
                randomness,
            }
        );
        assert!(RPSProgramInstruction::unpack(&data[..data.len() - 1]).is_err());
        data.push(0);
        assert!(RPSProgramInstruction::unpack(&data).is_err());

        let mut unseeded_data = vec![91];
        unseeded_data.extend_from_slice(&randomness);
        assert!(RPSProgramInstruction::unpack(&unseeded_data).is_err());
    }

    #[test]
    fn roulette_vrf_callback_requires_the_exact_round_and_request_seed() {
        let expected_round_id: u64 = 91;
        let expected_vrf_seed = [11u8; 32];
        let randomness = [13u8; 32];
        let mut data = vec![92];
        data.extend_from_slice(&randomness);
        data.extend_from_slice(&expected_round_id.to_le_bytes());
        data.extend_from_slice(&expected_vrf_seed);

        assert_eq!(
            RPSProgramInstruction::unpack(&data).unwrap(),
            RPSProgramInstruction::RussianRouletteVrfCallback {
                expected_round_id,
                expected_vrf_seed,
                randomness,
            }
        );
        assert!(RPSProgramInstruction::unpack(&data[..data.len() - 1]).is_err());
        data.push(0);
        assert!(RPSProgramInstruction::unpack(&data).is_err());

        let mut unseeded_data = vec![92];
        unseeded_data.extend_from_slice(&randomness);
        assert!(RPSProgramInstruction::unpack(&unseeded_data).is_err());
    }

    #[test]
    fn raffle_vrf_callback_requires_the_exact_request_seed() {
        let randomness = [17u8; 32];
        let expected_vrf_seed = [19u8; 32];
        let mut data = vec![90];
        data.extend_from_slice(&randomness);
        data.extend_from_slice(&expected_vrf_seed);
        assert_eq!(
            RPSProgramInstruction::unpack(&data).unwrap(),
            RPSProgramInstruction::RaffleVrfCallback {
                expected_vrf_seed,
                randomness,
            }
        );

        data.push(0);
        assert!(RPSProgramInstruction::unpack(&data).is_err());
        let mut unseeded_data = vec![90];
        unseeded_data.extend_from_slice(&randomness);
        assert!(RPSProgramInstruction::unpack(&unseeded_data).is_err());
    }

    #[test]
    fn vrf_retry_failure_and_refund_tags_are_exact() {
        for (tag, expected) in [
            (35, RPSProgramInstruction::RetryRaffleDraw),
            (36, RPSProgramInstruction::MarkRaffleVrfFailed),
            (37, RPSProgramInstruction::RefundFailedRaffleTicket),
        ] {
            assert_eq!(RPSProgramInstruction::unpack(&[tag]).unwrap(), expected);
            assert!(RPSProgramInstruction::unpack(&[tag, 0]).is_err());
        }

        let round_id = 77u64;
        assert_eq!(
            RPSProgramInstruction::unpack(&round_instruction(41, round_id)).unwrap(),
            RPSProgramInstruction::MarkRussianRouletteVrfFailed {
                expected_round_id: round_id,
            }
        );
        assert_eq!(
            RPSProgramInstruction::unpack(&round_instruction(42, round_id)).unwrap(),
            RPSProgramInstruction::RefundFailedRussianRouletteRound {
                expected_round_id: round_id,
            }
        );
        assert!(RPSProgramInstruction::unpack(&[41]).is_err());
        assert!(RPSProgramInstruction::unpack(&[42]).is_err());
    }

    #[test]
    fn dice_instructions_decode_exact_generation_nonces() {
        let nonce = [23u8; 32];
        let init_dice = InitDice {
            chosen_dices: [1, 0, 0, 0, 0, 0],
            game_id: 81,
            stake: 10_000_000,
            generation_entropy: nonce,
        };
        let join_dice = JoinDice {
            chosen_dices: [2, 0, 0, 0, 0, 0],
            expected_generation_nonce: nonce,
        };
        let mut create_data = vec![43];
        init_dice.serialize(&mut create_data).unwrap();
        let mut join_data = vec![44];
        join_dice.serialize(&mut join_data).unwrap();

        assert_eq!(
            RPSProgramInstruction::unpack(&create_data).unwrap(),
            RPSProgramInstruction::CreateDiceGame { init_dice }
        );
        assert_eq!(
            RPSProgramInstruction::unpack(&join_data).unwrap(),
            RPSProgramInstruction::JoinDiceGame { join_dice }
        );

        for (tag, expected) in [
            (
                45,
                RPSProgramInstruction::RequestDiceDraw {
                    expected_generation_nonce: nonce,
                },
            ),
            (
                46,
                RPSProgramInstruction::FinalizeDiceDraw {
                    expected_generation_nonce: nonce,
                },
            ),
            (
                47,
                RPSProgramInstruction::RetryDiceDraw {
                    expected_generation_nonce: nonce,
                },
            ),
            (
                48,
                RPSProgramInstruction::MarkDiceVrfFailed {
                    expected_generation_nonce: nonce,
                },
            ),
            (
                49,
                RPSProgramInstruction::RefundFailedDiceGame {
                    expected_generation_nonce: nonce,
                },
            ),
        ] {
            let mut data = vec![tag];
            data.extend_from_slice(&nonce);
            assert_eq!(RPSProgramInstruction::unpack(&data).unwrap(), expected);
            assert!(RPSProgramInstruction::unpack(&data[..data.len() - 1]).is_err());
            data.push(0);
            assert!(RPSProgramInstruction::unpack(&data).is_err());
        }

        assert!(RPSProgramInstruction::unpack(&create_data[..create_data.len() - 1]).is_err());
        assert!(RPSProgramInstruction::unpack(&join_data[..join_data.len() - 1]).is_err());
    }

    #[test]
    fn obsolete_dice_tags_are_not_part_of_the_contract() {
        for tag in [20, 21, 22, 23, 38, 39, 40] {
            assert_eq!(
                RPSProgramInstruction::unpack(&[tag]).unwrap_err(),
                ProgramError::from(InvalidInstruction)
            );
        }
        assert_eq!(
            RPSProgramInstruction::unpack(&[24]).unwrap(),
            RPSProgramInstruction::CloseDiceGame
        );
        assert!(RPSProgramInstruction::unpack(&[24, 0]).is_err());
    }
}
