use anchor_lang::prelude::*;
use anchor_spl::token::{self};
use anchor_lang::system_program;

use instructions::*;
use error::*;

declare_id!("HBxSXtxCvNC8sK2z8XxF5YsSsGP8Se8MKc6XD9xBkyuf");

pub mod instructions;
pub mod state;
pub mod error;

#[program]
mod spl_vending_machine {
    use super::*;
    
    pub fn create_machine(
        ctx: Context<InitializeMachine>,

        ppa: u64,
        ppt: u64,

        ticket_allocation: u64,

        presale_start: i64,
        presale_end: i64,
        pubsale_start: i64,
        pubsale_end: i64,

    ) -> Result<()> {
        
        let vending_machine = &mut ctx.accounts.vending_machine;

        vending_machine.bump = *ctx.bumps.get("vending_machine").unwrap();

        vending_machine.authority = ctx.accounts.authority.key();

        vending_machine.spl_mint = ctx.accounts.spl_mint.key();
        vending_machine.spl_stock = 0;

        vending_machine.ticket_allocation = ticket_allocation;
        vending_machine.tickets_sold = 0;

        vending_machine.ppa = ppa;
        vending_machine.ppt = ppt;

        vending_machine.presale_start = presale_start;
        vending_machine.presale_end = presale_end;
        vending_machine.pubsale_start = pubsale_start;
        vending_machine.pubsale_end = pubsale_end;

        //false by default
        vending_machine.ready = 0;
    
        Ok(())
    }

    pub fn fund_machine(
        ctx: Context<FundMachine>, 
        amount: u64
    ) -> Result<()> {

        let vending_machine = &mut ctx.accounts.vending_machine;

        //make sure the spl token being deposited matches the one selected initially.
        require_keys_eq!(
            ctx.accounts.spl_mint.key(),
            vending_machine.spl_mint,
            VendingMachineError::Unauthorized
        );

        //make sure the funder is also the authority of the vending machine
        require_keys_eq!(
            ctx.accounts.authority.key(),
            vending_machine.authority,
            VendingMachineError::Unauthorized
        );

        //transfer the tokens 
        let token_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                authority: ctx.accounts.authority.to_account_info(),
                from: ctx.accounts.authority_spl_ata.to_account_info(),
                to: ctx.accounts.vending_machine_spl_ata.to_account_info(),
            },
        );
        token::transfer(token_ctx, amount)?;    
        
        vending_machine.spl_stock = vending_machine.spl_stock + amount;
        vending_machine.ready = 1;

        Ok(())
    }

    pub fn buy_ticket(
        ctx: Context<CreateTicket>,
        amount: u64
    ) -> Result<()> {

        let vending_machine = &mut ctx.accounts.vending_machine;

        if vending_machine.ready == 0 {
            return Err(VendingMachineError::NotReady.into())
        }

        if vending_machine.tickets_sold >= vending_machine.ticket_allocation {
            return Err(VendingMachineError::NoTickets.into())
        }

        //make sure the contract will send SOL to the vending machine owner
        require_keys_eq!(
            vending_machine.authority,
            ctx.accounts.authority.key(),
            VendingMachineError::Unauthorized
        );

        //lamports
        let transfer_amount = amount * vending_machine.ppa;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(), 
            system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.authority.clone(),
            });
        system_program::transfer(cpi_context, transfer_amount)?;

        let ticket = &mut ctx.accounts.ticket;
        ticket.buyer = ctx.accounts.buyer.key();
        ticket.vending_machine = vending_machine.key();
        ticket.unspent = amount;
        ticket.spent = 0;

        vending_machine.tickets_sold = vending_machine.tickets_sold + 1;

        Ok(())

    }

    pub fn buy_spl_with_ticket(
        ctx: Context<BuySplWithTicket>,
        amount: u64
    ) -> Result<()> {

        let ticket = &mut ctx.accounts.ticket;
        let vending_machine = &mut ctx.accounts.vending_machine;

        //make sure there enough allocation spots on the ticket.
        if (ticket.unspent - ticket.spent) > amount {
            return Err(VendingMachineError::NotEnoughFunds.into());
        }

        //since we're buying with a ticket we'll make sure there is enough ticket allocations left.
        if amount > vending_machine.ticket_allocation {
            return Err(VendingMachineError::ShortSupply.into());
        }

        //make sure the machine is ready
        if vending_machine.ready == 0 {
            return Err(VendingMachineError::NotReady.into());
        }

        //make sure the ticket owner is the signer
        require_keys_eq!(
            ticket.buyer,
            ctx.accounts.signer.key(),
            VendingMachineError::Unauthorized
        );

        //make sure the contract will send SOL to the vending machine owner
        require_keys_eq!(
            vending_machine.authority,
            ctx.accounts.authority.key(),
            VendingMachineError::Unauthorized
        );

        //update the ticket amount
        ticket.spent = ticket.spent + amount;
        ticket.unspent = ticket.unspent - amount;
        //deduct our amount from our spent ticke allocation
        vending_machine.ticket_allocation = vending_machine.ticket_allocation - amount;
        //we also need to deduct the amount from the whole SPL supply
        vending_machine.spl_stock = vending_machine.spl_stock - amount;

        let transfer_amount = amount * vending_machine.ppt;

        //currently will be transfered to authority
        let transfer_sol_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(), 
            system_program::Transfer {
                from: ctx.accounts.signer.to_account_info(),
                to: ctx.accounts.authority.clone(),
            });
        system_program::transfer(transfer_sol_ctx, transfer_amount)?;

        let authority = vending_machine.authority;
        let vending_machine_bump = vending_machine.bump;
        let seeds = &[b"vending-machine".as_ref(), authority.as_ref(), &[vending_machine_bump]];
        let signer =  &[&seeds[..]];

        let transfer_token_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                authority: ctx.accounts.vending_machine.to_account_info(),
                from: ctx.accounts.vending_machine_spl_ata.to_account_info(),
                to: ctx.accounts.buyer_spl_ata.to_account_info(),
            },
            signer
        );
        token::transfer(transfer_token_ctx, amount)?; 

        Ok(())
    }

    pub fn buy_spl(
        ctx: Context<BuySpl>,
        amount: u64
    ) -> Result<()> {

        let vending_machine = &mut ctx.accounts.vending_machine;

        //make sure the machine is ready
        if vending_machine.ready == 0 {
            return Err(VendingMachineError::NotReady.into());
        }
        //since we're not buying with a ticket, we have to make sure we leave enough left for the ticket allocations if needed.
        if vending_machine.ticket_allocation == vending_machine.spl_stock {
            return Err(VendingMachineError::ShortSupply.into());
        }
        //make sure there is enough SPL tokens in the contract to make the purchase.
        if amount > vending_machine.spl_stock {
            return Err(VendingMachineError::ShortSupply.into());
        }

        //make sure the contract will send SOL to the vending machine owner
        require_keys_eq!(
            vending_machine.authority,
            ctx.accounts.authority.key(),
            VendingMachineError::Unauthorized
        );

        //we also need to deduct the amount from the whole SPL supply
        vending_machine.spl_stock = vending_machine.spl_stock - amount;

        let transfer_amount = amount * vending_machine.ppt;

        //currently will be transfered to authority
        let transfer_sol_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(), 
            system_program::Transfer {
                from: ctx.accounts.signer.to_account_info(),
                to: ctx.accounts.authority.clone(),
            });
        system_program::transfer(transfer_sol_ctx, transfer_amount)?;

        let authority = vending_machine.authority;
        let vending_machine_bump = vending_machine.bump;
        let seeds = &[b"vending-machine".as_ref(), authority.as_ref(), &[vending_machine_bump]];
        let signer =  &[&seeds[..]];

        let transfer_token_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                authority: ctx.accounts.vending_machine.to_account_info(),
                from: ctx.accounts.vending_machine_spl_ata.to_account_info(),
                to: ctx.accounts.buyer_spl_ata.to_account_info(),
            },
            signer
        );
        token::transfer(transfer_token_ctx, amount)?; 

        Ok(())
    }
 }



