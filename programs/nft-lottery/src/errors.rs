use anchor_lang::prelude::*;

#[error_code]
pub enum LotteryError {
    #[msg("User tried to buy ticket after the lottery has ended")]
    LotteryExpired,
    #[msg("The entries has been stopped by the owner")]
    EntryDisabled,
    #[msg("Only creator can draw the winner")]
    AccessDenied,
    #[msg("Lottery is Still active")]
    LotteryStillActive,
    #[msg("No tickets sold to pick the winner")]
    TicketsNotSold,
    #[msg("Winner Not Picked")]
    WinnerNotPicked,
    #[msg("Winner has already been picked")]
    WinnerAlreadyPicked,
    #[msg("There is was an error when fulfilling randomness")]
    RandomnessNotFulfilled,
    #[msg("Winning Tickets Mismatch")]
    InvalidWinningTicket,
}
