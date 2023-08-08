use anchor_lang::prelude::*;
use anchor_spl::token::{self};
use anchor_lang::system_program;

use instructions::*;
use error::*;

declare_id!("11111111111111111111111111111111");

pub mod instructions;
pub mod state;
pub mod error;

#[program]
mod spl_vending_machine {
    use super::*;
    
    pub fn create_machine(
        ctx: Context<InitializeMachine>,
        ppa: u64,
        ticket_allocation: u64,
        presale_start: i64,
        presale_end: i64,
        pubsale_start: i64,
        pubsale_end: i64 
    ) -> Result<()> {
        
        let vending_machine = &mut ctx.accounts.vending_machine;

        vending_machine.authority = ctx.accounts.authority.key();
        vending_machine.spl_mint = ctx.accounts.spl_mint.key();

        vending_machine.spl_stock = 0;
        vending_machine.ticket_allocation = ticket_allocation;
        vending_machine.spent_ticket_allocation = 0;

        vending_machine.ppa = ppa;
        vending_machine.presale_start = presale_start;
        vending_machine.presale_end = presale_end;
        vending_machine.pubsale_start = pubsale_start;
        vending_machine.pubsale_end = pubsale_end;

        //false by default
        vending_machine.ready = 0;
    
        Ok(())
    }

    pub fn fund_farm(
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

        Ok(())
    }

    pub fn buy_ticket(
        ctx: Context<CreateTicket>,
        amount: u64
    ) -> Result<()> {

        let vending_machine = &ctx.accounts.vending_machine;

        if vending_machine.ready == 0 {
            return Err(VendingMachineError::NotReady.into())
        }

        //make sure the contract will send SOL to the vending machine owner
        require_keys_eq!(
            ctx.accounts.vending_machine.authority,
            ctx.accounts.authority.key(),
            VendingMachineError::Unauthorized
        );

        //make sure the buyer is the signer
        require_keys_eq!(
            ctx.accounts.buyer.key(),
            ctx.accounts.signer.key(),
            VendingMachineError::Unauthorized
        );

        //lamports
        let transfer_amount = amount * vending_machine.ppa;

        //currently will be transfered to authority
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(), 
            system_program::Transfer {
                from: ctx.accounts.buyer.clone(),
                to: ctx.accounts.authority.clone(),
            });
        system_program::transfer(cpi_context, transfer_amount)?;

        let ticket = &mut ctx.accounts.ticket;
        ticket.buyer = ctx.accounts.buyer.key();
        ticket.vending_machine = vending_machine.key();
        ticket.unspent = amount;
        ticket.spent = 0;

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
            return Err(VendingMachineError::NotEnoughFunds.into())
        }

        //since we're buying with a ticket we'll make sure there is enough ticket allocations left.
        if amount > vending_machine.ticket_allocation {
            return Err(VendingMachineError::ShortSupply.into())
        }

        //make sure the machine is ready
        if vending_machine.ready == 0 {
            return Err(VendingMachineError::NotReady.into())
        }

        //make sure the ticket owner is the signer
        require_keys_eq!(
            ticket.buyer,
            ctx.accounts.signer.key(),
            VendingMachineError::Unauthorized
        );

        //make sure the buyer is the signer
        require_keys_eq!(
            ctx.accounts.buyer.key(),
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
        ticket.spent = ticket.unspent - amount;
        //deduct our amount from our spent ticke allocation
        vending_machine.ticket_allocation = vending_machine.ticket_allocation - amount;
        //we also need to deduct the amount from the whole SPL supply
        vending_machine.spl_stock = vending_machine.spl_stock - amount;

        let transfer_amount = amount * vending_machine.ppt;

        //currently will be transfered to authority
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(), 
            system_program::Transfer {
                from: ctx.accounts.buyer.clone(),
                to: ctx.accounts.authority.clone(),
            });
        system_program::transfer(cpi_context, transfer_amount)?;

        Ok(())
    }

    pub fn buy_spl(
        ctx: Context<BuySpl>,
        amount: u64
    ) -> Result<()> {

        let vending_machine = &mut ctx.accounts.vending_machine;

        //make sure the machine is ready
        if vending_machine.ready == 0 {
            return Err(VendingMachineError::NotReady.into())
        }
  
        //since we're not buying with a ticket, we have to make sure we leave enough left for the ticket allocations if needed.
        if vending_machine.ticket_allocation == vending_machine.spl_stock {
            return Err(VendingMachineError::ShortSupply.into())
        }
        //make sure there is enough SPL tokens in the contract to make the purchase.
        if amount > vending_machine.spl_stock {
            return Err(VendingMachineError::ShortSupply.into())
        }

        //make sure the buyer is the signer
        require_keys_eq!(
            ctx.accounts.buyer.key(),
            ctx.accounts.signer.key(),
            VendingMachineError::Unauthorized
        );

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
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(), 
            system_program::Transfer {
                from: ctx.accounts.buyer.clone(),
                to: ctx.accounts.authority.clone(),
            });
        system_program::transfer(cpi_context, transfer_amount)?;

        Ok(())
    }

}

