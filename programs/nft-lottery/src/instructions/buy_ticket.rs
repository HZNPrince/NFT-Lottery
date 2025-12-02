use crate::errors::LotteryError;
use crate::state::{Lottery, LotteryStatus, UserTicket};
use anchor_lang::{
    prelude::*,
    system_program::{self, Transfer},
};

#[derive(Accounts)]
pub struct BuyTicket<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(mut)]
    pub lottery: Account<'info, Lottery>,

    #[account(
        init,
        payer = buyer,
        space = 8 + UserTicket::INIT_SPACE,
        seeds = [b"ticket", lottery.key().as_ref(), lottery.tickets_sold.to_le_bytes().as_ref()],
        bump,
    )]
    pub user_ticket: Account<'info, UserTicket>,

    pub system_program: Program<'info, System>,
}

pub fn process_buy(ctx: Context<BuyTicket>) -> Result<()> {
    // Checks to ensure ticket bought is Valid
    let lottery = &mut ctx.accounts.lottery;
    let clock = Clock::get()?.unix_timestamp;
    if clock > lottery.end_time as i64 {
        return Err(LotteryError::LotteryExpired.into());
    }

    require!(
        lottery.lottery_status == LotteryStatus::Active,
        LotteryError::EntryDisabled
    );

    // Create users buy ticket account
    let user_ticket = &mut ctx.accounts.user_ticket;
    user_ticket.owner = ctx.accounts.buyer.key();
    user_ticket.lottery = lottery.key();
    user_ticket.ticket_number = lottery.tickets_sold;
    user_ticket.bump = ctx.bumps.user_ticket;

    // Transfer Buying amount to the lottery_nft_vault (System program)
    let transfer_amount = lottery.ticket_price;

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: lottery.to_account_info(),
            },
        ),
        transfer_amount,
    )?;

    // Update the lottery state
    lottery.tickets_sold += 1;
    Ok(())
}
