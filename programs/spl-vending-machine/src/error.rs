use anchor_lang::prelude::*;

#[error_code]
pub enum VendingMachineError {
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Machine not ready")]
    NotReady,
    #[msg("Sale not commenced")]
    NotStarted,
    #[msg("Not enough funds")]
    NotEnoughFunds,
    #[msg("Not enough SPL tokens to fulfill purchase")]
    ShortSupply,
    #[msg("Allocation tickets have sold out")]
    NoTickets,
}
