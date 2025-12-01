use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{
    errors::LotteryError,
    state::{Lottery, LotteryStatus},
};

#[derive(Accounts)]
pub struct RewardWinner<'info> {
    #[account(
        mut,
        address = lottery.winner.unwrap() @ LotteryError::AccessDenied
    )]
    pub winner: Signer<'info>,

    pub nft_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"lottery", lottery.creator.as_ref(), nft_mint.key().as_ref()],
        bump = lottery.bump,
    )]
    pub lottery: Account<'info, Lottery>,

    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = lottery,
        associated_token::token_program = token_program,
    )]
    pub nft_lottery_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = winner,
        associated_token::mint = nft_mint,
        associated_token::authority = winner,
        associated_token::token_program = token_program,
    )]
    pub winner_nft: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn process_reward_winner(ctx: Context<RewardWinner>) -> Result<()> {
    let lottery = &mut ctx.accounts.lottery;
    // Validate Winner is announced
    require!(lottery.winner.is_some(), LotteryError::WinnerNotPicked);

    // Verify Lottery Completed
    require!(
        lottery.lottery_status == LotteryStatus::Completed,
        LotteryError::LotteryStillActive
    );

    // Transfer from lottery vault to winners token account
    let transfer_accounts = TransferChecked {
        from: ctx.accounts.nft_lottery_vault.to_account_info(),
        to: ctx.accounts.winner_nft.to_account_info(),
        mint: ctx.accounts.nft_mint.to_account_info(),
        authority: lottery.to_account_info(),
    };
    let creator_keys = lottery.creator;
    let nft_mint_keys = lottery.nft_mint;
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"lottery",
        creator_keys.as_ref(),
        nft_mint_keys.as_ref(),
        &[lottery.bump],
    ]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_accounts,
        signer_seeds,
    );
    token_interface::transfer_checked(cpi_ctx, 1, 0)?;

    Ok(())
}
