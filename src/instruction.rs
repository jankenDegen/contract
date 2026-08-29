use crate::{
    error::RPSProgramError::InvalidInstruction,
    state::{
        Buy, DiceManager, InitDice, InitGame, InitRussianRoulette, InitRussianRouletteTable,
        JoinDice, JoinGame, JoinRussianRoulette, Manager, RaffleManager, Reveal,
        UpdateRussianRouletteTable,
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
        randomness: [u8; 32],
    },
    ClaimPrize,
    CloseRaffle,
    SetDiceManager {
        new_manager: DiceManager,
    },
    InitDiceManager,
    CreateDiceGame {
        init_dice: InitDice,
    },
    JoinDiceGame {
        join_dice: JoinDice,
    },
    RequestDiceDraw,
    FinalizeDiceDraw,
    DiceVrfCallback {
        randomness: [u8; 32],
    },
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
        randomness: [u8; 32],
    },
    CloseRussianRouletteGame,
    ResetRussianRouletteTable {
        expected_round_id: u64,
    },
    RetryRussianRouletteDraw {
        expected_round_id: u64,
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
            20 => Self::CreateDiceGame {
                init_dice: InitDice::try_from_slice(&rest)?,
            },
            21 => Self::JoinDiceGame {
                join_dice: JoinDice::try_from_slice(&rest)?,
            },
            22 => Self::RequestDiceDraw,
            23 => Self::FinalizeDiceDraw,
            24 => Self::CloseDiceGame,
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
            90 => Self::RaffleVrfCallback {
                randomness: Self::unpack_randomness(rest)?,
            },
            91 => Self::DiceVrfCallback {
                randomness: Self::unpack_randomness(rest)?,
            },
            92 => Self::RussianRouletteVrfCallback {
                randomness: Self::unpack_randomness(rest)?,
            },

            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_randomness(input: &[u8]) -> Result<[u8; 32], ProgramError> {
        if input.len() < 32 {
            return Err(InvalidInstruction.into());
        }

        let mut randomness = [0u8; 32];
        randomness.copy_from_slice(&input[..32]);
        Ok(randomness)
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
}
