use anchor_lang::prelude::*;

mod instructions;
use instructions::*;

mod state;

mod errors;

declare_id!("HVfJHK3e4uofseQ8V7eHCAEMafZowgPpjVKEugdLeW2m");

#[program]
pub mod nft_lottery {

    use super::*;

    pub fn create_lottery(
        ctx: Context<CreateLottery>,
        ticket_price: u64,
        start_time: u64,
        end_time: u64,
        force: [u8; 32],
    ) -> Result<()> {
        process_create_lottery(ctx, ticket_price, start_time, end_time, force)
    }

    pub fn buy_ticket(ctx: Context<BuyTicket>) -> Result<()> {
        process_buy(ctx)
    }

    pub fn request_randomness(ctx: Context<RequestRandomness>) -> Result<()> {
        process_request_randomness(ctx)
    }

    pub fn pick_winner(ctx: Context<PickWinner>) -> Result<()> {
        process_pick_winner(ctx)
    }

    pub fn reward_winner(ctx: Context<RewardWinner>) -> Result<()> {
        process_reward_winner(ctx)
    }
}
