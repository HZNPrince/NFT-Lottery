use anchor_lang::prelude::*;
use orao_solana_vrf::{
    program::OraoVrf,
    state::{NetworkState, RandomnessAccountData},
    CONFIG_ACCOUNT_SEED, RANDOMNESS_ACCOUNT_SEED,
};

use crate::{
    errors::LotteryError,
    state::{Lottery, LotteryStatus, UserTicket},
};

#[derive(Accounts)]
pub struct RequestRandomness<'info> {
    #[account(
        mut,
        address = lottery.creator @ LotteryError::AccessDenied
    )]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"lottery", lottery.creator.as_ref(), lottery.nft_mint.as_ref()],
        bump,
    )]
    pub lottery: Account<'info, Lottery>,

    /// CHECK: This is the randomness account checked by the orao vrf program
    #[account(
        mut,
        seeds = [RANDOMNESS_ACCOUNT_SEED, lottery.force.as_ref()],
        bump,
        seeds::program = orao_solana_vrf::ID
    )]
    pub randomness_account: AccountInfo<'info>,

    #[account(mut)]
    /// CHECK: This account is the treasury fees account of Orao
    pub vrf_treasury: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [CONFIG_ACCOUNT_SEED],
        bump,
        seeds::program = orao_solana_vrf::ID,
    )]
    pub vrf_state: Account<'info, NetworkState>,

    pub system_program: Program<'info, System>,
    pub vrf_program: Program<'info, OraoVrf>,
}

#[derive(Accounts)]
pub struct PickWinner<'info> {
    #[account(
        mut,
        address = lottery.creator @ LotteryError::AccessDenied
    )]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"lottery", lottery.creator.as_ref(), lottery.nft_mint.as_ref()],
        bump,
    )]
    pub lottery: Account<'info, Lottery>,

    #[account(
        mut,
        seeds = [RANDOMNESS_ACCOUNT_SEED, lottery.force.as_ref()],
        bump,
        seeds::program = orao_solana_vrf::ID
    )]
    /// CHECK: This is the randomness account checked by the orao vrf program
    pub randomness_account: AccountInfo<'info>,

    pub winning_ticket: Account<'info, UserTicket>,
}

pub fn process_request_randomness(ctx: Context<RequestRandomness>) -> Result<()> {
    let lottery = &mut ctx.accounts.lottery;

    // Validations
    // Lottery ended
    let clock = Clock::get()?.unix_timestamp;
    require!(
        clock > lottery.end_time as i64,
        LotteryError::LotteryStillActive
    );

    // Tickets sold
    require_gt!(lottery.tickets_sold, 0, LotteryError::TicketsNotSold);

    // Check Winner picked
    require!(lottery.winner.is_none(), LotteryError::WinnerAlreadyPicked);

    let cpi_program = ctx.accounts.vrf_program.to_account_info();
    let cpi_accounts = orao_solana_vrf::cpi::accounts::RequestV2 {
        payer: ctx.accounts.creator.to_account_info(),
        network_state: ctx.accounts.vrf_state.to_account_info(),
        treasury: ctx.accounts.vrf_treasury.to_account_info(),
        request: ctx.accounts.randomness_account.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    orao_solana_vrf::cpi::request_v2(cpi_ctx, lottery.force)?;

    Ok(())
}

pub fn process_pick_winner(ctx: Context<PickWinner>) -> Result<()> {
    let lottery = &mut ctx.accounts.lottery;

    // Deserialize
    let randomness_account = RandomnessAccountData::try_deserialize(
        &mut ctx.accounts.randomness_account.data.borrow().as_ref(),
    )?;

    // Get the Fulfilled randomness
    let randomness = randomness_account
        .fulfilled_randomness()
        .ok_or(LotteryError::RandomnessNotFulfilled)?;

    // Convert first 8bytes to u64
    let random_value = u64::from_le_bytes(randomness[0..8].try_into().unwrap());

    // Get the winning ticket number
    let winning_ticket_number = random_value % lottery.tickets_sold;

    // cross-verify the winning ticket
    require_eq!(
        ctx.accounts.winning_ticket.ticket_number,
        winning_ticket_number,
        LotteryError::InvalidWinningTicket
    );

    // Update the states
    lottery.winner = Some(ctx.accounts.winning_ticket.owner);
    lottery.lottery_status = LotteryStatus::Completed;

    Ok(())
}
