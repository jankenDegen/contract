use solana_program::program_error::ProgramError;
use thiserror::Error;

//ArithmeticError, InvalidAuth, InvalidConfig, InvalidCounter, NotSignerAuth, InvalidPool, InvalidPoolState

#[derive(Error, Debug, Copy, Clone)]
pub enum RPSProgramError {
    /// Invalid Instruction
    #[error("Invalid Instruction")] //0
    InvalidInstruction,

    #[error("game manager account is writable")] //1
    ManagerWritable,

    #[error("arithmetic error")] //2
    ArithmeticError,

    #[error("invalid counter account")] //3
    InvalidCounter,

    #[error("wage amount is less than minimum allowed amount")] //4
    MinStake,

    #[error("game state account does not belong to program or doesnt match")] //5
    InvalidGameAccount,

    #[error("player is not signer")] //6
    PlayerNotSigner,

    #[error("seed and the decision hash does not match with the provided hash")] //7
    InvalidHash,

    #[error("player still has time to make a move")] //8
    InvalidTime,

    #[error("game state is invalid")] //9
    InvalidGameState,

    #[error("manager account is invalid")] //10
    InvalidManager,

    #[error("invalid authhority")] //11
    InvalidAuth,

    #[error("admin is not signer")] //12
    NotSignerAuth,

    #[error("invalid initializer address")] //13
    InvalidInitializer,

    #[error("invalid guest address")] //14
    InvalidGuest,

    #[error("invalid config account")] //15
    InvalidConfig,

    #[error("invalid raffle account")] //16
    InvalidRaffleAccount,

    #[error("invalid rticket no")] //17
    InvalidTicketNo,

    #[error("all tickets must be sold")] //18
    TikcetsNotSold,

    #[error("derived account does not match expected PDA")] //19
    InvalidDerivedAccount,

    #[error("account is already initialized")] //20
    AccountAlreadyInitialized,

    #[error("raffle state is invalid for this action")] //21
    InvalidRaffleState,

    #[error("raffle draw is already pending or complete")] //22
    RaffleDrawAlreadyRequested,

    #[error("raffle randomness is not fulfilled yet")] //23
    RaffleDrawPending,

    #[error("invalid VRF program or account")] //24
    InvalidVrfAccount,

    #[error("invalid winning ticket account")] //25
    InvalidTicketAccount,

    #[error("invalid player account")] //26
    InvalidPlayer,

    #[error("invalid fee percentage")] //27
    InvalidFeePercentage,

    #[error("invalid raffle configuration")] //28
    InvalidRaffleConfiguration,

    #[error("dice should be between 1 and 6")] //29
    InvalidChosenDice,

    #[error("dice game is not ready for draw")] //30
    DiceGameNotReady,

    #[error("dice draw has already been requested or finalized")] //31
    DiceDrawAlreadyRequested,

    #[error("dice randomness is not fulfilled yet")] //32
    DiceDrawPending,

    #[error("invalid dice manager configuration")] //33
    InvalidDiceConfiguration,

    #[error("russian roulette game is not ready for draw")] //34
    RussianRouletteGameNotReady,

    #[error("russian roulette draw has already been requested or finalized")] //35
    RussianRouletteDrawAlreadyRequested,

    #[error("russian roulette randomness is not fulfilled yet")] //36
    RussianRouletteDrawPending,

    #[error("invalid russian roulette manager configuration")] //37
    InvalidRussianRouletteConfiguration,

    #[error("payer does not match")] //38
    InvalidPayer,

    #[error("VRF retry is not available yet")] //39
    VrfRetryTooEarly,

    #[error("all VRF retries have already been requested")] //40
    VrfRetryLimitReached,

    #[error("VRF failure cannot be declared yet")] //41
    VrfFailureTooEarly,

    #[error("VRF retries have not been exhausted")] //42
    VrfRetriesNotExhausted,
}

impl From<RPSProgramError> for ProgramError {
    fn from(e: RPSProgramError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
