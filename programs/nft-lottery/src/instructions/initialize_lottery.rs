use crate::state::{Lottery, LotteryStatus};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
pub struct CreateLottery<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        space = 8 + Lottery::INIT_SPACE,
        seeds = [b"lottery", creator.key().as_ref() ,nft_mint.key().as_ref()],
        bump,
    )]
    pub lottery: Account<'info, Lottery>,

    pub nft_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = nft_mint,
        associated_token::authority = creator,
        associated_token::token_program = token_program,
    )]
    pub creator_nft: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = nft_mint,
        associated_token::authority = lottery,
        associated_token::token_program = token_program
    )]
    pub lottery_nft_vault: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn process_create_lottery(
    ctx: Context<CreateLottery>,
    ticket_price: u64,
    start_time: u64,
    end_time: u64,
    force: [u8; 32],
) -> Result<()> {
    // Update Lottery Account states
    let lottery = &mut ctx.accounts.lottery;
    lottery.creator = ctx.accounts.creator.key();
    lottery.ticket_price = ticket_price;
    lottery.start_time = start_time;
    lottery.end_time = end_time;
    lottery.lottery_status = LotteryStatus::Active;
    lottery.bump = ctx.bumps.lottery;
    lottery.nft_mint = ctx.accounts.nft_mint.key();
    lottery.winner = None;
    lottery.tickets_sold = 0;
    lottery.force = force;

    // Transfer the NFT from creator to the Lottery nft vault
    let transfer_accounts = TransferChecked {
        from: ctx.accounts.creator_nft.to_account_info(),
        to: ctx.accounts.lottery_nft_vault.to_account_info(),
        mint: ctx.accounts.nft_mint.to_account_info(),
        authority: ctx.accounts.creator.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_accounts,
    );

    token_interface::transfer_checked(cpi_ctx, 1, 0)?; // decimals is 0 cause its an NFT

    Ok(())
}
