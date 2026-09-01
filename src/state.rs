use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Config {
    pub is_init: u8,
    pub admin_1: [u8; 32],
    pub admin_2: [u8; 32],
    pub admin_3: [u8; 32],
    pub admin_4: [u8; 32],
    pub admin_5: [u8; 32],
}

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Clone, Copy)]
pub struct Manager {
    pub is_init: u8,
    pub allowed_time: u64,
    pub fee_percentage: u16,
    pub minimum_stake: u64,
    pub collected_fee: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy)]
pub struct Game {
    pub initializer: [u8; 32],
    pub hash: [u8; 32],
    pub guest: [u8; 32],
    pub fee_percentage: u16,
    pub amount: u64,
    pub allowed_time: u64,
    pub last_play_time: u64,
    pub guest_decision: u8,
    pub state: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Copy)]
pub struct InitGame {
    pub amount: u64,
    pub hash: [u8; 32],
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Copy)]
pub struct Reveal {
    pub decision: u8,
    pub seed: [u8; 32],
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Copy)]
pub struct JoinGame {
    pub decision: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy)]
pub struct Ticket {
    pub ticket_no: u8,
    pub raffle_no: u32,
    pub player: [u8; 32],
    pub payer: [u8; 32],
}

#[cfg(test)]
mod ticket_tests {
    use super::Ticket;
    use crate::constants::TICKET_SPACE;
    use borsh::to_vec;

    #[test]
    fn ticket_space_constant_matches_borsh_layout() {
        let ticket = Ticket {
            ticket_no: 0,
            raffle_no: 1,
            player: [0; 32],
            payer: [0; 32],
        };

        assert_eq!(to_vec(&ticket).unwrap().len() as u64, TICKET_SPACE);
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
pub struct RaffleState {
    pub tickets_sold: u8,
    pub sold_tickets: [u8; 100],
    pub raffle_no: u32,
    pub ticket_price: u64,
    pub winner_prize: u64,
    pub decimal_digit_match_prize: u64,
    pub unit_digit_match_prize: u64,
    pub program_fee: u64,
    pub draw_status: u8,
    pub winning_ticket_no: u8,
    pub vrf_seed: [u8; 32],
    pub vrf_last_request_at: i64,
    pub vrf_retry_count: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
pub struct RaffleManager {
    pub is_init: u8,
    pub total_raffles: u32,
    pub ticket_price: u64,
    pub winner_gets: u64,
    pub decimal_digit_match_prize: u64,
    pub unit_digit_match_prize: u64,
    pub program_fee: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
pub struct DiceManager {
    pub is_init: u8,
    pub minimum_stake: u64,
    pub program_fee: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Copy)]
pub struct Buy {
    pub ticket_no: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Copy)]

pub struct InitDice {
    pub chosen_dices: [u8; 6],
    pub game_id: u64,
    pub stake: u64,
    pub generation_entropy: [u8; 32],
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Copy)]
pub struct JoinDice {
    pub chosen_dices: [u8; 6],
    pub expected_generation_nonce: [u8; 32],
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Copy)]
pub struct InitRussianRouletteTable {
    pub table_id: u8,
    pub stake: u64,
    pub program_fee: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Copy)]
pub struct UpdateRussianRouletteTable {
    pub stake: u64,
    pub program_fee: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Copy)]
pub struct UpdateRussianRouletteParticipationFee {
    pub expected_round_id: u64,
    pub program_fee: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Copy)]
pub struct InitRussianRoulette {
    pub table_id: u8,
    pub seat: u8,
    pub expected_round_id: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Copy)]
pub struct JoinRussianRoulette {
    pub seat: u8,
    pub expected_round_id: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy)]
pub struct DiceGame {
    pub game_id: u64,
    pub number_of_players: u8,
    pub stake: u64,
    pub initializer: [u8; 32],
    pub player_2: [u8; 32],
    pub player_3: [u8; 32],
    pub player_4: [u8; 32],
    pub player_5: [u8; 32],
    pub player_6: [u8; 32],
    pub chosen_dices: [u8; 6],
    pub draw_status: u8,
    pub winning_dice: u8,
    pub vrf_seed: [u8; 32],
    pub generation_nonce: [u8; 32],
    pub program_fee: u64,
    pub vrf_last_request_at: i64,
    pub vrf_retry_count: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy)]
pub struct RussianRouletteGame {
    pub table_id: u8,
    pub round_id: u64,
    pub draw_status: u8,
    pub number_of_players: u8,
    pub stake: u64,
    pub program_fee: u64,
    pub seat_1: [u8; 32],
    pub seat_2: [u8; 32],
    pub seat_3: [u8; 32],
    pub seat_4: [u8; 32],
    pub seat_5: [u8; 32],
    pub seat_6: [u8; 32],
    pub unlucky_player_index: u8,
    pub vrf_seed: [u8; 32],
    pub unlucky_player: [u8; 32],
    pub vrf_last_request_at: i64,
    pub vrf_retry_count: u8,
    pub settled_at: i64,
}

#[cfg(test)]
mod account_layout_tests {
    use super::*;
    use crate::constants::{DICE_GAME_SPACE, RAFFLE_SPACE, RUSSIAN_ROULETTE_GAME_SPACE};
    use borsh::to_vec;

    fn raffle() -> RaffleState {
        RaffleState {
            tickets_sold: 16,
            sold_tickets: [1; 100],
            raffle_no: 7,
            ticket_price: 20_000_000,
            winner_prize: 1_000_000_000,
            decimal_digit_match_prize: 50_000_000,
            unit_digit_match_prize: 25_000_000,
            program_fee: 100_000_000,
            draw_status: 0,
            winning_ticket_no: u8::MAX,
            vrf_seed: [3; 32],
            vrf_last_request_at: 1_234,
            vrf_retry_count: 2,
        }
    }

    fn dice() -> DiceGame {
        DiceGame {
            game_id: 11,
            number_of_players: 6,
            stake: 10_000_000,
            initializer: [1; 32],
            player_2: [2; 32],
            player_3: [3; 32],
            player_4: [4; 32],
            player_5: [5; 32],
            player_6: [6; 32],
            chosen_dices: [1, 2, 3, 4, 5, 6],
            draw_status: 2,
            winning_dice: 4,
            vrf_seed: [7; 32],
            generation_nonce: [4; 32],
            program_fee: 3_000_000,
            vrf_last_request_at: 1_234,
            vrf_retry_count: 2,
        }
    }

    fn roulette() -> RussianRouletteGame {
        RussianRouletteGame {
            table_id: 0,
            round_id: 12,
            draw_status: 0,
            number_of_players: 3,
            stake: 100_000_000,
            program_fee: 10_000_000,
            seat_1: [1; 32],
            seat_2: [2; 32],
            seat_3: [3; 32],
            seat_4: [0; 32],
            seat_5: [0; 32],
            seat_6: [0; 32],
            unlucky_player_index: u8::MAX,
            vrf_seed: [0; 32],
            unlucky_player: [0; 32],
            vrf_last_request_at: 1_234,
            vrf_retry_count: 2,
            settled_at: 1_500,
        }
    }

    #[test]
    fn current_account_layouts_match_their_space_constants() {
        assert_eq!(to_vec(&raffle()).unwrap().len() as u64, RAFFLE_SPACE);
        assert_eq!(to_vec(&dice()).unwrap().len() as u64, DICE_GAME_SPACE);
        assert_eq!(
            to_vec(&roulette()).unwrap().len() as u64,
            RUSSIAN_ROULETTE_GAME_SPACE
        );
    }

    #[test]
    fn dice_layout_pins_backend_offsets() {
        let bytes = to_vec(&dice()).unwrap();

        assert_eq!(&bytes[0..8], &11_u64.to_le_bytes());
        assert_eq!(bytes[8], 6);
        assert_eq!(&bytes[9..17], &10_000_000_u64.to_le_bytes());
        assert_eq!(&bytes[17..49], &[1; 32]);
        assert_eq!(&bytes[177..209], &[6; 32]);
        assert_eq!(&bytes[209..215], &[1, 2, 3, 4, 5, 6]);
        assert_eq!(bytes[215], 2);
        assert_eq!(bytes[216], 4);
        assert_eq!(&bytes[217..249], &[7; 32]);
        assert_eq!(&bytes[249..281], &[4; 32]);
        assert_eq!(&bytes[281..289], &3_000_000_u64.to_le_bytes());
        assert_eq!(&bytes[289..297], &1_234_u64.to_le_bytes());
        assert_eq!(bytes[297], 2);
    }

    #[test]
    fn truncated_pre_retry_metadata_layouts_are_rejected() {
        let raffle_bytes = to_vec(&raffle()).unwrap();
        let dice_bytes = to_vec(&dice()).unwrap();
        let roulette_bytes = to_vec(&roulette()).unwrap();

        assert!(RaffleState::try_from_slice(&raffle_bytes[..179]).is_err());
        assert!(DiceGame::try_from_slice(&dice_bytes[..281]).is_err());
        assert!(RussianRouletteGame::try_from_slice(&roulette_bytes[..284]).is_err());
    }
}
