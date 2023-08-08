use anchor_lang::prelude::*;

#[account]
pub struct VendingMachine {
    pub bump: u8,

    pub authority: Pubkey,

    pub spl_mint: Pubkey, 
    pub spl_stock: u64,

    pub ticket_allocation: u64,
    pub tickets_sold: u64,

    pub ppa: u64,
    pub ppt: u64,

    pub ready: u8,

    pub presale_start: i64,
    pub presale_end: i64,
    pub pubsale_start: i64,
    pub pubsale_end: i64,
}

#[account]
pub struct Ticket {
    pub vending_machine: Pubkey,
    pub buyer: Pubkey,
    pub unspent: u64,
    pub spent: u64,
}

impl VendingMachine {
    pub const LEN: usize = 1 + 32 + 32 + 16 + 16 + 16 + 16 + 16 + 1 + 8 + 8 + 8 + 8;
}

impl Ticket {
    pub const LEN: usize = 32 + 32 + 16 + 16;
}