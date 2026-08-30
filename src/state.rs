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
pub struct LegacyRaffleState {
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

impl From<LegacyRaffleState> for RaffleState {
    fn from(value: LegacyRaffleState) -> Self {
        Self {
            tickets_sold: value.tickets_sold,
            sold_tickets: value.sold_tickets,
            raffle_no: value.raffle_no,
            ticket_price: value.ticket_price,
            winner_prize: value.winner_prize,
            decimal_digit_match_prize: value.decimal_digit_match_prize,
            unit_digit_match_prize: value.unit_digit_match_prize,
            program_fee: value.program_fee,
            draw_status: value.draw_status,
            winning_ticket_no: value.winning_ticket_no,
            vrf_seed: value.vrf_seed,
            vrf_last_request_at: 0,
            vrf_retry_count: 0,
        }
    }
}

impl From<RaffleState> for LegacyRaffleState {
    fn from(value: RaffleState) -> Self {
        Self {
            tickets_sold: value.tickets_sold,
            sold_tickets: value.sold_tickets,
            raffle_no: value.raffle_no,
            ticket_price: value.ticket_price,
            winner_prize: value.winner_prize,
            decimal_digit_match_prize: value.decimal_digit_match_prize,
            unit_digit_match_prize: value.unit_digit_match_prize,
            program_fee: value.program_fee,
            draw_status: value.draw_status,
            winning_ticket_no: value.winning_ticket_no,
            vrf_seed: value.vrf_seed,
        }
    }
}

impl RaffleState {
    pub fn try_from_compatible_slice(data: &[u8]) -> std::io::Result<Self> {
        match data.len() as u64 {
            crate::constants::LEGACY_RAFFLE_SPACE => {
                LegacyRaffleState::try_from_slice(data).map(Into::into)
            }
            crate::constants::RAFFLE_SPACE => Self::try_from_slice(data),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid raffle account size",
            )),
        }
    }

    pub fn serialize_compatible(&self, data: &mut [u8]) -> std::io::Result<()> {
        match data.len() as u64 {
            crate::constants::LEGACY_RAFFLE_SPACE => {
                LegacyRaffleState::from(*self).serialize(&mut &mut data[..])
            }
            crate::constants::RAFFLE_SPACE => self.serialize(&mut &mut data[..]),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid raffle account size",
            )),
        }
    }
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
    pub winner: [u8; 32],
    pub program_fee: u64,
    pub vrf_last_request_at: i64,
    pub vrf_retry_count: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy)]
pub struct LegacyDiceGame {
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

impl From<LegacyDiceGame> for DiceGame {
    fn from(value: LegacyDiceGame) -> Self {
        Self {
            game_id: value.game_id,
            number_of_players: value.number_of_players,
            stake: value.stake,
            initializer: value.initializer,
            player_2: value.player_2,
            player_3: value.player_3,
            player_4: value.player_4,
            player_5: value.player_5,
            player_6: value.player_6,
            chosen_dices: value.chosen_dices,
            draw_status: value.draw_status,
            winning_dice: value.winning_dice,
            vrf_seed: value.vrf_seed,
            winner: value.winner,
            program_fee: 0,
            vrf_last_request_at: 0,
            vrf_retry_count: 0,
        }
    }
}

impl DiceGame {
    pub fn try_from_compatible_slice(data: &[u8]) -> std::io::Result<Self> {
        match data.len() as u64 {
            crate::constants::LEGACY_DICE_GAME_SPACE => {
                LegacyDiceGame::try_from_slice(data).map(Into::into)
            }
            crate::constants::DICE_GAME_SPACE => Self::try_from_slice(data),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid dice account size",
            )),
        }
    }
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

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy)]
pub struct LegacyRussianRouletteGame {
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

impl From<LegacyRussianRouletteGame> for RussianRouletteGame {
    fn from(value: LegacyRussianRouletteGame) -> Self {
        Self {
            table_id: value.table_id,
            round_id: value.round_id,
            draw_status: value.draw_status,
            number_of_players: value.number_of_players,
            stake: value.stake,
            program_fee: value.program_fee,
            seat_1: value.seat_1,
            seat_2: value.seat_2,
            seat_3: value.seat_3,
            seat_4: value.seat_4,
            seat_5: value.seat_5,
            seat_6: value.seat_6,
            unlucky_player_index: value.unlucky_player_index,
            vrf_seed: value.vrf_seed,
            unlucky_player: value.unlucky_player,
            vrf_last_request_at: 0,
            vrf_retry_count: 0,
            settled_at: 0,
        }
    }
}

impl From<RussianRouletteGame> for LegacyRussianRouletteGame {
    fn from(value: RussianRouletteGame) -> Self {
        Self {
            table_id: value.table_id,
            round_id: value.round_id,
            draw_status: value.draw_status,
            number_of_players: value.number_of_players,
            stake: value.stake,
            program_fee: value.program_fee,
            seat_1: value.seat_1,
            seat_2: value.seat_2,
            seat_3: value.seat_3,
            seat_4: value.seat_4,
            seat_5: value.seat_5,
            seat_6: value.seat_6,
            unlucky_player_index: value.unlucky_player_index,
            vrf_seed: value.vrf_seed,
            unlucky_player: value.unlucky_player,
        }
    }
}

impl RussianRouletteGame {
    pub fn try_from_compatible_slice(data: &[u8]) -> std::io::Result<Self> {
        match data.len() as u64 {
            crate::constants::LEGACY_RUSSIAN_ROULETTE_GAME_SPACE => {
                LegacyRussianRouletteGame::try_from_slice(data).map(Into::into)
            }
            crate::constants::RUSSIAN_ROULETTE_GAME_SPACE => Self::try_from_slice(data),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid roulette account size",
            )),
        }
    }

    pub fn serialize_compatible(&self, data: &mut [u8]) -> std::io::Result<()> {
        match data.len() as u64 {
            crate::constants::LEGACY_RUSSIAN_ROULETTE_GAME_SPACE => {
                LegacyRussianRouletteGame::from(*self).serialize(&mut &mut data[..])
            }
            crate::constants::RUSSIAN_ROULETTE_GAME_SPACE => self.serialize(&mut &mut data[..]),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid roulette account size",
            )),
        }
    }
}

#[cfg(test)]
mod layout_compatibility_tests {
    use super::*;
    use crate::constants::{
        DICE_GAME_SPACE, LEGACY_DICE_GAME_SPACE, LEGACY_RAFFLE_SPACE,
        LEGACY_RUSSIAN_ROULETTE_GAME_SPACE, RAFFLE_SPACE, RUSSIAN_ROULETTE_GAME_SPACE,
    };
    use borsh::to_vec;

    fn legacy_raffle() -> LegacyRaffleState {
        LegacyRaffleState {
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
        }
    }

    fn legacy_dice() -> LegacyDiceGame {
        LegacyDiceGame {
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
            winner: [4; 32],
        }
    }

    fn legacy_roulette() -> LegacyRussianRouletteGame {
        LegacyRussianRouletteGame {
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
        }
    }

    #[test]
    fn raffle_v1_and_v2_layouts_are_exact_and_prefix_compatible() {
        let legacy_bytes = to_vec(&legacy_raffle()).unwrap();
        assert_eq!(legacy_bytes.len() as u64, LEGACY_RAFFLE_SPACE);
        let mut current = RaffleState::try_from_compatible_slice(&legacy_bytes).unwrap();
        assert_eq!(current.tickets_sold, 16);
        assert_eq!(current.vrf_last_request_at, 0);
        assert_eq!(current.vrf_retry_count, 0);
        current.vrf_last_request_at = 1_234;
        current.vrf_retry_count = 2;
        let current_bytes = to_vec(&current).unwrap();
        assert_eq!(current_bytes.len() as u64, RAFFLE_SPACE);
        assert_eq!(
            &current_bytes[..legacy_bytes.len()],
            legacy_bytes.as_slice()
        );
    }

    #[test]
    fn dice_v1_and_v2_layouts_are_exact_and_prefix_compatible() {
        let legacy_bytes = to_vec(&legacy_dice()).unwrap();
        assert_eq!(legacy_bytes.len() as u64, LEGACY_DICE_GAME_SPACE);
        let mut current = DiceGame::try_from_compatible_slice(&legacy_bytes).unwrap();
        assert_eq!(current.game_id, 11);
        assert_eq!(current.program_fee, 0);
        assert_eq!(current.vrf_last_request_at, 0);
        assert_eq!(current.vrf_retry_count, 0);
        current.program_fee = 3_000_000;
        current.vrf_last_request_at = 1_234;
        current.vrf_retry_count = 2;
        let current_bytes = to_vec(&current).unwrap();
        assert_eq!(current_bytes.len() as u64, DICE_GAME_SPACE);
        assert_eq!(
            &current_bytes[..legacy_bytes.len()],
            legacy_bytes.as_slice()
        );
    }

    #[test]
    fn roulette_v1_and_v2_layouts_are_exact_and_prefix_compatible() {
        let legacy_bytes = to_vec(&legacy_roulette()).unwrap();
        assert_eq!(
            legacy_bytes.len() as u64,
            LEGACY_RUSSIAN_ROULETTE_GAME_SPACE
        );
        let mut current = RussianRouletteGame::try_from_compatible_slice(&legacy_bytes).unwrap();
        assert_eq!(current.round_id, 12);
        assert_eq!(current.vrf_last_request_at, 0);
        assert_eq!(current.vrf_retry_count, 0);
        assert_eq!(current.settled_at, 0);
        current.vrf_last_request_at = 1_234;
        current.vrf_retry_count = 2;
        current.settled_at = 1_500;
        let current_bytes = to_vec(&current).unwrap();
        assert_eq!(current_bytes.len() as u64, RUSSIAN_ROULETTE_GAME_SPACE);
        assert_eq!(
            &current_bytes[..legacy_bytes.len()],
            legacy_bytes.as_slice()
        );
    }
}
