use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::state::*;

// Accounts
#[derive(Accounts)]
pub struct InitializeMachine<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + VendingMachine::LEN,
    )]
    pub vending_machine: Box<Account<'info, VendingMachine>>,
    pub spl_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FundMachine<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub vending_machine: Box<Account<'info, VendingMachine>>,

    #[account(mut)]
    pub authority_spl_ata: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = spl_mint,
        associated_token::authority = vending_machine,
    )]
    pub vending_machine_spl_ata: Account<'info, TokenAccount>,

    pub spl_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateTicket<'info> {

    /// CHECK: using eq check
    pub authority: AccountInfo<'info>,
    /// CHECK: using eq check
    pub buyer: AccountInfo<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub vending_machine: Box<Account<'info, VendingMachine>>,

    #[account(
        init,
        payer = signer,
        space = 8 + Ticket::LEN,
    )]
    pub ticket: Box<Account<'info, Ticket>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BuySplWithTicket<'info> {

    /// CHECK: using eq check
    pub authority: AccountInfo<'info>,
    /// CHECK: using eq check
    pub buyer: AccountInfo<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub vending_machine: Box<Account<'info, VendingMachine>>,

    #[account(
        init,
        payer = signer,
        space = 8 + Ticket::LEN,
    )]
    pub ticket: Box<Account<'info, Ticket>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}


#[derive(Accounts)]
pub struct BuySpl<'info> {

    /// CHECK: using eq check
    pub authority: AccountInfo<'info>,
    /// CHECK: using eq check
    pub buyer: AccountInfo<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub vending_machine: Box<Account<'info, VendingMachine>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
