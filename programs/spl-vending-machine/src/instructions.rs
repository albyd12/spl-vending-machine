use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::state::*;

// Accounts
#[derive(Accounts)]
pub struct InitializeMachine<'info> {

    #[account(
        init,
        space = 8 + VendingMachine::LEN,
        seeds =[
            b"vending-machine".as_ref(),
            authority.key().as_ref()
        ],
        bump,
        payer = authority,
    )]
    pub vending_machine: Box<Account<'info, VendingMachine>>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub spl_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FundMachine<'info> {

    #[account(
        mut,
        seeds = [
            b"vending-machine".as_ref(),
            vending_machine.authority.as_ref()
        ],
        bump = vending_machine.bump
    )]
    pub vending_machine: Box<Account<'info, VendingMachine>>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = spl_mint,
        associated_token::authority = vending_machine,
    )]
    pub vending_machine_spl_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub authority_spl_ata: Account<'info, TokenAccount>,

    pub spl_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateTicket<'info> {

    /// CHECK: using eq check
    #[account(mut)]
    pub authority: AccountInfo<'info>,
  
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            b"vending-machine".as_ref(),
            vending_machine.authority.as_ref()
        ],
        bump = vending_machine.bump
    )]
    pub vending_machine: Box<Account<'info, VendingMachine>>,

    #[account(
        init,
        payer = buyer,
        space = 8 + Ticket::LEN,
    )]
    pub ticket: Box<Account<'info, Ticket>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BuySplWithTicket<'info> {

    #[account(
        mut,
        seeds = [
            b"vending-machine".as_ref(),
            vending_machine.authority.as_ref()
        ],
        bump = vending_machine.bump
    )]
    pub vending_machine: Box<Account<'info, VendingMachine>>,
    
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = spl_mint,
        associated_token::authority = signer,
    )]
    pub buyer_spl_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = spl_mint,
        associated_token::authority = vending_machine,
    )]
    pub vending_machine_spl_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub ticket: Box<Account<'info, Ticket>>,

    /// CHECK: using eq check
    #[account(mut)]
    pub authority: AccountInfo<'info>,

    pub spl_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BuySpl<'info> {

    #[account(
        mut,
        seeds = [
            b"vending-machine".as_ref(),
            vending_machine.authority.as_ref()
        ],
        bump = vending_machine.bump
    )]
    pub vending_machine: Box<Account<'info, VendingMachine>>,
    
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = spl_mint,
        associated_token::authority = signer,
    )]
    pub buyer_spl_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = spl_mint,
        associated_token::authority = vending_machine,
    )]
    pub vending_machine_spl_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK: using eq check
    #[account(mut)]
    pub authority: AccountInfo<'info>,

    pub spl_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
