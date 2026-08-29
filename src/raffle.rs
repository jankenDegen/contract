use crate::{
    constants::{
        MANAGER_SEED, RAFFLE_SEED, RAFFLE_SPACE, RAFFLE_STATUS_DRAWN, RAFFLE_STATUS_OPEN,
        RAFFLE_STATUS_PENDING, RAFFLE_TICKET_COUNT, RAFFLE_VRF_CALLBACK_TAG, RAFFLE_VRF_SEED,
        TICKET_SEED, TICKET_SPACE, UNDRAWN_TICKET_NO,
    },
    error::RPSProgramError::InvalidManager,
    magicblock_vrf::MagicBlockVrf,
    state::{Buy, Manager, RaffleManager, RaffleState, Ticket},
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
};
use solana_system_interface::{instruction::transfer, program as system_program};

use crate::error::RPSProgramError::{
    ArithmeticError, InvalidPayer, InvalidPlayer, InvalidRaffleAccount, InvalidRaffleState,
    InvalidTicketAccount, InvalidTicketNo, PlayerNotSigner, RaffleDrawAlreadyRequested,
    RaffleDrawPending, TikcetsNotSold,
};

pub struct Raffle;
impl Raffle {
    pub fn buy_ticket(accounts: &[AccountInfo], program_id: &Pubkey, buy: Buy) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let payer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let player: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let ticket_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let raffle_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let system_program_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        if raffle_account.owner != program_id {
            return Err(InvalidRaffleAccount.into());
        }

        if buy.ticket_no >= RAFFLE_TICKET_COUNT {
            return Err(InvalidTicketNo.into());
        }

        let mut raffle: RaffleState = RaffleState::try_from_slice(&raffle_account.data.borrow())?;
        if raffle.draw_status != RAFFLE_STATUS_OPEN || raffle.tickets_sold >= RAFFLE_TICKET_COUNT {
            return Err(InvalidRaffleState.into());
        }

        let ticket_no_bytes = buy.ticket_no.to_le_bytes();
        let raffle_no_bytes = raffle.raffle_no.to_le_bytes();
        let seeds: &[&[u8]] = &[TICKET_SEED, &ticket_no_bytes, RAFFLE_SEED, &raffle_no_bytes];

        Utils::create_pda(
            payer,
            ticket_account,
            system_program_account,
            seeds,
            TICKET_SPACE,
            program_id,
        )?;

        let ticket: Ticket = Ticket {
            raffle_no: raffle.raffle_no,
            ticket_no: buy.ticket_no,
            player: player.key.to_bytes(),
            payer: payer.key.to_bytes(),
        };

        raffle.tickets_sold = raffle.tickets_sold.checked_add(1).ok_or(ArithmeticError)?;
        raffle.sold_tickets[buy.ticket_no as usize] = 1;

        invoke(
            &transfer(payer.key, raffle_account.key, raffle.ticket_price),
            &[
                payer.clone(),
                raffle_account.clone(),
                system_program_account.clone(),
            ],
        )?;

        ticket.serialize(&mut &mut ticket_account.data.borrow_mut()[..])?;
        raffle.serialize(&mut &mut raffle_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn create_raffle(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let admin: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let raffle_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let raffle_manager: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let config_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let system_program_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        Utils::check_admin(admin, config_account, program_id)?;

        let mut manager: RaffleManager =
            RaffleManager::try_from_slice(&raffle_manager.data.borrow())?;
        if manager.is_init != 1 {
            return Err(InvalidRaffleState.into());
        }

        manager.total_raffles = manager
            .total_raffles
            .checked_add(1)
            .ok_or(ArithmeticError)?;

        let raffle_no_bytes = manager.total_raffles.to_le_bytes();
        let seeds: &[&[u8]] = &[RAFFLE_SEED, &raffle_no_bytes];

        Utils::create_pda(
            admin,
            raffle_account,
            system_program_account,
            seeds,
            RAFFLE_SPACE,
            program_id,
        )?;

        let lottery = RaffleState {
            tickets_sold: 0,
            raffle_no: manager.total_raffles,
            ticket_price: manager.ticket_price,
            winner_prize: manager.winner_gets,
            decimal_digit_match_prize: manager.decimal_digit_match_prize,
            unit_digit_match_prize: manager.unit_digit_match_prize,
            draw_status: RAFFLE_STATUS_OPEN,
            winning_ticket_no: UNDRAWN_TICKET_NO,
            vrf_seed: [0; 32],
            program_fee: manager.program_fee,
            sold_tickets: [0; 100],
        };

        lottery.serialize(&mut &mut raffle_account.data.borrow_mut()[..])?;
        manager.serialize(&mut &mut raffle_manager.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn request_draw(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let payer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let raffle_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let vrf_request_identity: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let oracle_queue: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let system_program_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let slot_hashes_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let vrf_program_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        if raffle_account.owner != program_id {
            return Err(InvalidRaffleAccount.into());
        }
        if !payer.is_signer {
            return Err(PlayerNotSigner.into());
        }

        let mut raffle: RaffleState = RaffleState::try_from_slice(&raffle_account.data.borrow())?;
        if raffle.tickets_sold != RAFFLE_TICKET_COUNT {
            return Err(TikcetsNotSold.into());
        }

        if raffle.draw_status != RAFFLE_STATUS_OPEN {
            return Err(RaffleDrawAlreadyRequested.into());
        }

        let vrf_seed = Self::vrf_request_seed(raffle_account.key, raffle.raffle_no);
        MagicBlockVrf::request_randomness(
            RAFFLE_VRF_CALLBACK_TAG,
            payer,
            vrf_request_identity,
            oracle_queue,
            system_program_account,
            slot_hashes_account,
            vrf_program_account,
            program_id,
            raffle_account,
            vrf_seed,
        )?;

        raffle.draw_status = RAFFLE_STATUS_PENDING;
        raffle.winning_ticket_no = UNDRAWN_TICKET_NO;
        raffle.vrf_seed = vrf_seed;
        raffle.serialize(&mut &mut raffle_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn finalize_draw(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let raffle_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let manager_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let winning_ticket_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        if raffle_account.owner != program_id {
            return Err(InvalidRaffleAccount.into());
        }
        if manager_account.owner != program_id {
            return Err(InvalidManager.into());
        }
        if winning_ticket_account.owner != program_id {
            return Err(InvalidTicketAccount.into());
        }

        let mut manager: Manager = Manager::try_from_slice(&manager_account.data.borrow())?;
        let expected_manager = Pubkey::find_program_address(&[MANAGER_SEED], program_id).0;
        if manager_account.key != &expected_manager {
            return Err(InvalidManager.into());
        }

        let mut raffle: RaffleState = RaffleState::try_from_slice(&raffle_account.data.borrow())?;
        if raffle.draw_status == RAFFLE_STATUS_DRAWN {
            return Err(RaffleDrawAlreadyRequested.into());
        }
        if raffle.draw_status != RAFFLE_STATUS_PENDING {
            return Err(InvalidRaffleState.into());
        }
        if manager.is_init != 1 {
            return Err(InvalidManager.into());
        }
        if raffle.winning_ticket_no == UNDRAWN_TICKET_NO {
            return Err(RaffleDrawPending.into());
        }
        let winning_ticket = Ticket::try_from_slice(&winning_ticket_account.data.borrow())?;
        if winning_ticket.raffle_no != raffle.raffle_no
            || winning_ticket.ticket_no != raffle.winning_ticket_no
            || winning_ticket_account.key
                != &Self::ticket_address(
                    program_id,
                    raffle.raffle_no,
                    raffle.winning_ticket_no,
                )
        {
            return Err(InvalidTicketAccount.into());
        }

        raffle.draw_status = RAFFLE_STATUS_DRAWN;
        manager.collected_fee = manager
            .collected_fee
            .checked_add(raffle.program_fee)
            .ok_or(ArithmeticError)?;

        **raffle_account.try_borrow_mut_lamports()? -= raffle.program_fee;
        **manager_account.try_borrow_mut_lamports()? += raffle.program_fee;

        raffle.serialize(&mut &mut raffle_account.data.borrow_mut()[..])?;
        manager.serialize(&mut &mut manager_account.data.borrow_mut()[..])?;

        msg!(
            "raffle_draw_finalized raffle_no={} winning_ticket_no={} winner_address={} tickets_sold={} ticket_stake_lamports={} winner_prize_lamports={} program_fee_lamports={}",
            raffle.raffle_no,
            raffle.winning_ticket_no,
            Pubkey::new_from_array(winning_ticket.player),
            raffle.tickets_sold,
            raffle.ticket_price,
            raffle.winner_prize,
            raffle.program_fee
        );

        Ok(())
    }

    pub fn consume_vrf_randomness(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        randomness: [u8; 32],
    ) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let vrf_program_identity: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let raffle_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        MagicBlockVrf::validate_callback_identity(vrf_program_identity, program_id)?;
        if raffle_account.owner != program_id {
            return Err(InvalidRaffleAccount.into());
        }

        let mut raffle: RaffleState = RaffleState::try_from_slice(&raffle_account.data.borrow())?;
        if raffle.draw_status != RAFFLE_STATUS_PENDING {
            return Err(InvalidRaffleState.into());
        }
        if raffle.winning_ticket_no != UNDRAWN_TICKET_NO {
            return Err(RaffleDrawAlreadyRequested.into());
        }

        let random_number = Self::randomness_number(&randomness);
        let winning_ticket_no =
            Self::winning_ticket_no(&randomness, &raffle.sold_tickets, raffle.tickets_sold)?;
        raffle.vrf_seed = randomness;
        raffle.winning_ticket_no = winning_ticket_no;
        raffle.serialize(&mut &mut raffle_account.data.borrow_mut()[..])?;

        msg!(
            "raffle_vrf_result raffle_no={} randomness={} random_u64={} winning_ticket_no={}",
            raffle.raffle_no,
            Pubkey::new_from_array(randomness),
            random_number,
            winning_ticket_no
        );

        Ok(())
    }

    pub fn claim_prize(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let payer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let player: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let ticket_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let raffle_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

        if raffle_account.owner != program_id {
            return Err(InvalidRaffleAccount.into());
        }
        if ticket_account.owner != program_id {
            return Err(InvalidTicketAccount.into());
        }

        let mut raffle: RaffleState = RaffleState::try_from_slice(&raffle_account.data.borrow())?;
        let ticket = Ticket::try_from_slice(&ticket_account.data.borrow())?;

        if raffle.raffle_no != ticket.raffle_no {
            return Err(InvalidTicketAccount.into());
        }
        let expected_ticket = Self::ticket_address(program_id, raffle.raffle_no, ticket.ticket_no);
        if ticket_account.key != &expected_ticket {
            return Err(InvalidTicketAccount.into());
        }
        if raffle.draw_status != RAFFLE_STATUS_DRAWN {
            return Err(InvalidRaffleState.into());
        }
        if player.key.to_bytes() != ticket.player {
            return Err(InvalidPlayer.into());
        }
        if payer.key.to_bytes() != ticket.payer {
            return Err(InvalidPayer.into());
        }

        let (outcome, prize) = if raffle.winning_ticket_no == ticket.ticket_no {
            ("winner", raffle.winner_prize)
        } else if raffle.winning_ticket_no % 10 == ticket.ticket_no % 10 {
            ("unit_match", raffle.unit_digit_match_prize)
        } else if raffle.winning_ticket_no / 10 == ticket.ticket_no / 10 {
            ("decimal_match", raffle.decimal_digit_match_prize)
        } else {
            ("loser", 0)
        };
        if prize > 0 {
            **raffle_account.try_borrow_mut_lamports()? -= prize;
            **player.try_borrow_mut_lamports()? += prize;
        }

        raffle.tickets_sold = raffle.tickets_sold.checked_sub(1).ok_or(ArithmeticError)?;

        let rest: u64 = **ticket_account.try_borrow_lamports()?;

        **ticket_account.try_borrow_mut_lamports()? -= rest;
        **payer.try_borrow_mut_lamports()? += rest;

        ticket_account.resize(0)?;
        ticket_account.assign(&system_program::ID);

        raffle.serialize(&mut &mut raffle_account.data.borrow_mut()[..])?;

        msg!(
            "raffle_ticket_result raffle_no={} ticket_no={} winning_ticket_no={} player={} outcome={} ticket_stake_lamports={} payout_lamports={}",
            raffle.raffle_no,
            ticket.ticket_no,
            raffle.winning_ticket_no,
            player.key,
            outcome,
            raffle.ticket_price,
            prize
        );

        Ok(())
    }

    fn ticket_address(program_id: &Pubkey, raffle_no: u32, ticket_no: u8) -> Pubkey {
        let ticket_no_bytes = ticket_no.to_le_bytes();
        let raffle_no_bytes = raffle_no.to_le_bytes();
        Pubkey::find_program_address(
            &[TICKET_SEED, &ticket_no_bytes, RAFFLE_SEED, &raffle_no_bytes],
            program_id,
        )
        .0
    }

    fn vrf_request_seed(raffle_address: &Pubkey, raffle_no: u32) -> [u8; 32] {
        let raffle_no_bytes = raffle_no.to_le_bytes();
        hash(&[RAFFLE_VRF_SEED, raffle_address.as_ref(), &raffle_no_bytes].concat()).to_bytes()
    }

    fn winning_ticket_no(
        randomness: &[u8; 32],
        sold_tickets: &[u8; RAFFLE_TICKET_COUNT as usize],
        tickets_sold: u8,
    ) -> Result<u8, ProgramError> {
        if tickets_sold == 0 {
            return Err(InvalidRaffleState.into());
        }

        if sold_tickets.iter().any(|marker| *marker > 1) {
            return Err(InvalidRaffleState.into());
        }
        let marked_ticket_count = sold_tickets.iter().filter(|marker| **marker == 1).count() as u8;
        if marked_ticket_count != tickets_sold {
            return Err(InvalidRaffleState.into());
        }

        let number = Self::randomness_number(randomness);
        let selected_index = number % tickets_sold as u64;

        sold_tickets
            .iter()
            .enumerate()
            .filter(|(_, marker)| **marker == 1)
            .nth(selected_index as usize)
            .map(|(ticket_no, _)| ticket_no as u8)
            .ok_or_else(|| InvalidRaffleState.into())
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

    fn randomness(value: u64) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        bytes[..8].copy_from_slice(&value.to_le_bytes());
        bytes
    }

    #[test]
    fn winner_is_selected_from_sparse_sold_ticket_numbers() {
        let mut sold_tickets = [0u8; RAFFLE_TICKET_COUNT as usize];
        sold_tickets[3] = 1;
        sold_tickets[42] = 1;
        sold_tickets[99] = 1;

        assert_eq!(
            Raffle::winning_ticket_no(&randomness(0), &sold_tickets, 3).unwrap(),
            3
        );
        assert_eq!(
            Raffle::winning_ticket_no(&randomness(1), &sold_tickets, 3).unwrap(),
            42
        );
        assert_eq!(
            Raffle::winning_ticket_no(&randomness(2), &sold_tickets, 3).unwrap(),
            99
        );
        assert_eq!(
            Raffle::winning_ticket_no(&randomness(4), &sold_tickets, 3).unwrap(),
            42
        );
    }

    #[test]
    fn winner_rejects_bitmap_count_mismatch_and_invalid_markers() {
        let mut sold_tickets = [0u8; RAFFLE_TICKET_COUNT as usize];
        sold_tickets[7] = 1;
        assert!(Raffle::winning_ticket_no(&randomness(0), &sold_tickets, 2).is_err());

        sold_tickets[8] = 2;
        assert!(Raffle::winning_ticket_no(&randomness(0), &sold_tickets, 2).is_err());
    }
}
