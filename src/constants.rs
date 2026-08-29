pub const GAME_SPACE: u64 = 124;
pub const MANAGER_SPACE: u64 = 27;
pub const CONFIG_SPACE: u64 = 161;
pub const TICKET_SPACE: u64 = 69;
pub const RAFFLE_SPACE: u64 = 179;
pub const RAFFLE_MANAGER_SPACE: u64 = 45;
pub const DICE_MANAGER_SPACE: u64 = 17;
pub const DICE_GAME_SPACE: u64 = 281;
pub const RUSSIAN_ROULETTE_MANAGER_SPACE: u64 = 17;
pub const RUSSIAN_ROULETTE_GAME_SPACE: u64 = 284;

pub const GAME_SEED: &[u8] = b"game";
pub const MANAGER_SEED: &[u8] = b"manager";
pub const CONFIG_SEED: &[u8] = b"config";
pub const TICKET_SEED: &[u8] = b"ticket";
pub const RAFFLE_SEED: &[u8] = b"raffle";
pub const RAFFLE_MANAGER_SEED: &[u8] = b"rafflemanager";
pub const RAFFLE_VRF_SEED: &[u8] = b"raffle-vrf";
pub const DICE_SEED: &[u8] = b"dice";
pub const DICE_MANAGER_SEED: &[u8] = b"dicemanager";
pub const DICE_VRF_SEED: &[u8] = b"dice-vrf";
pub const RUSSIAN_ROULETTE_SEED: &[u8] = b"russianroulette";
pub const RUSSIAN_ROULETTE_MANAGER_SEED: &[u8] = b"russianroulettemanager";
pub const RUSSIAN_ROULETTE_VRF_SEED: &[u8] = b"russianroulette-vrf";

pub const RAFFLE_VRF_CALLBACK_TAG: u8 = 90;
pub const DICE_VRF_CALLBACK_TAG: u8 = 91;
pub const RUSSIAN_ROULETTE_VRF_CALLBACK_TAG: u8 = 92;

pub const RAFFLE_STATUS_OPEN: u8 = 0;
pub const RAFFLE_STATUS_PENDING: u8 = 1;
pub const RAFFLE_STATUS_DRAWN: u8 = 2;

pub const RAFFLE_TICKET_COUNT: u8 = 100;
pub const UNDRAWN_TICKET_NO: u8 = u8::MAX;

pub const DICE_PLAYER_COUNT: u8 = 6;
pub const DICE_STATUS_OPEN: u8 = 0;
pub const DICE_STATUS_PENDING: u8 = 1;
pub const DICE_STATUS_DRAWN: u8 = 2;
pub const UNDRAWN_DICE_NO: u8 = 0;

pub const RUSSIAN_ROULETTE_PLAYER_COUNT: u8 = 6;
pub const RUSSIAN_ROULETTE_TABLE_COUNT: u8 = 3;
pub const RUSSIAN_ROULETTE_STARTER_STAKE: u64 = 100_000_000;
pub const RUSSIAN_ROULETTE_PRIME_STAKE: u64 = 500_000_000;
pub const RUSSIAN_ROULETTE_APEX_STAKE: u64 = 1_000_000_000;
pub const RUSSIAN_ROULETTE_PARTICIPATION_FEE: u64 = 10_000_000;
pub const RUSSIAN_ROULETTE_STATUS_OPEN: u8 = 0;
pub const RUSSIAN_ROULETTE_STATUS_PENDING: u8 = 1;
pub const RUSSIAN_ROULETTE_STATUS_DRAWN: u8 = 2;
pub const UNLUCKY_PLAYER_INDEX_NONE: u8 = u8::MAX;
