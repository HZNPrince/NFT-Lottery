use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum LotteryStatus {
    Active,
    Completed,
    Cancelled,
}

#[account]
#[derive(InitSpace)]
pub struct Lottery {
    pub creator: Pubkey,
    pub ticket_price: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub force: [u8; 32],
    pub nft_mint: Pubkey,
    pub tickets_sold: u64,
    pub lottery_status: LotteryStatus,
    pub winner: Option<Pubkey>,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct UserTicket {
    pub owner: Pubkey,
    pub lottery: Pubkey,
    pub ticket_number: u64,
    pub bump: u8,
}
