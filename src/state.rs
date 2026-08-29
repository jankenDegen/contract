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
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Copy)]
pub struct JoinDice {
    pub chosen_dices: [u8; 6],
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
    pub winner: [u8; 32],
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
}
