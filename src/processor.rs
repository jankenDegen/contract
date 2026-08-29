use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    admin::Admin, dice::Dice, initialize::Init, instruction::RPSProgramInstruction, janken::Janken,
    raffle::Raffle, russian_roulette::RussianRoulette,
};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction: RPSProgramInstruction = RPSProgramInstruction::unpack(instruction_data)?;

        match instruction {
            RPSProgramInstruction::InitGame { init_game } => {
                Janken::init_game(accounts, program_id, init_game)
            }
            RPSProgramInstruction::JoinGame { join_game } => {
                Janken::join_game(accounts, program_id, join_game)
            }
            RPSProgramInstruction::Reveal { reveal } => {
                Janken::reveal(accounts, program_id, reveal)
            }
            RPSProgramInstruction::TimeIsUp => Janken::time_is_up(accounts, program_id),
            RPSProgramInstruction::SetConfig => Admin::set_config(accounts, program_id),
            RPSProgramInstruction::InitAccounts => Init::init_game_accounts(accounts, program_id),
            RPSProgramInstruction::SetManager { new_manager } => {
                Admin::set_game_manager(accounts, program_id, new_manager)
            }
            RPSProgramInstruction::CollectFee => Admin::collect_fee(accounts, program_id),
            RPSProgramInstruction::SetRaffleManager { new_manager } => {
                Admin::set_raffle_manager(accounts, program_id, new_manager)
            }
            RPSProgramInstruction::CreateRaffle => Raffle::create_raffle(accounts, program_id),
            RPSProgramInstruction::BuyTicket { buy } => {
                Raffle::buy_ticket(accounts, program_id, buy)
            }
            RPSProgramInstruction::RequestRaffleDraw => Raffle::request_draw(accounts, program_id),
            RPSProgramInstruction::FinalizeRaffleDraw => {
                Raffle::finalize_draw(accounts, program_id)
            }
            RPSProgramInstruction::RaffleVrfCallback { randomness } => {
                Raffle::consume_vrf_randomness(accounts, program_id, randomness)
            }
            RPSProgramInstruction::ClaimPrize => Raffle::claim_prize(accounts, program_id),

            RPSProgramInstruction::CloseRaffle => Admin::close_raffle(accounts, program_id),
            RPSProgramInstruction::SetDiceManager { new_manager } => {
                Admin::set_dice_manager(accounts, program_id, new_manager)
            }
            RPSProgramInstruction::InitDiceManager => Init::init_dice_manager(accounts, program_id),
            RPSProgramInstruction::CreateDiceGame { init_dice } => {
                Dice::create_game(accounts, program_id, init_dice)
            }
            RPSProgramInstruction::JoinDiceGame { join_dice } => {
                Dice::join_game(accounts, program_id, join_dice)
            }
            RPSProgramInstruction::RequestDiceDraw => Dice::request_draw(accounts, program_id),
            RPSProgramInstruction::FinalizeDiceDraw => Dice::finalize_draw(accounts, program_id),
            RPSProgramInstruction::DiceVrfCallback { randomness } => {
                Dice::consume_vrf_randomness(accounts, program_id, randomness)
            }
            RPSProgramInstruction::CloseDiceGame => Admin::close_dice_game(accounts, program_id),
            RPSProgramInstruction::SetRussianRouletteTable { new_table } => {
                Admin::set_russian_roulette_table(accounts, program_id, new_table)
            }
            RPSProgramInstruction::InitRussianRouletteTable { init_table } => {
                Init::init_russian_roulette_table(accounts, program_id, init_table)
            }
            RPSProgramInstruction::CreateRussianRouletteGame { init_game } => {
                RussianRoulette::create_game(accounts, program_id, init_game)
            }
            RPSProgramInstruction::JoinRussianRouletteGame { join_game } => {
                RussianRoulette::join_game(accounts, program_id, join_game)
            }
            RPSProgramInstruction::RequestRussianRouletteDraw { expected_round_id } => {
                RussianRoulette::request_draw(accounts, program_id, expected_round_id)
            }
            RPSProgramInstruction::FinalizeRussianRouletteDraw { expected_round_id } => {
                RussianRoulette::finalize_draw(accounts, program_id, expected_round_id)
            }
            RPSProgramInstruction::RussianRouletteVrfCallback { randomness } => {
                RussianRoulette::consume_vrf_randomness(accounts, program_id, randomness)
            }
            RPSProgramInstruction::CloseRussianRouletteGame => {
                Admin::close_russian_roulette_game(accounts, program_id)
            }
            RPSProgramInstruction::ResetRussianRouletteTable { expected_round_id } => {
                RussianRoulette::reset_table_for_next_round(accounts, program_id, expected_round_id)
            }
            RPSProgramInstruction::RetryRussianRouletteDraw { expected_round_id } => {
                RussianRoulette::retry_draw(accounts, program_id, expected_round_id)
            }
        }
    }
}
