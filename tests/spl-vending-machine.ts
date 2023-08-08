import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SplVendingMachine } from "../target/types/spl_vending_machine";
import {
  airdrop,
  getAtaForMint,
  getVendingMachinePda,
  mintToken,
} from "./utils";
import { assert } from "chai";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

const config = {
  tokens: 3,
  allocations: 2,
  ppt: 0.01 * anchor.web3.LAMPORTS_PER_SOL,
  ppa: 0.1 * anchor.web3.LAMPORTS_PER_SOL,
  presale_start: Date.now(),
  presale_end: Date.now() + 10000000000000,
  pubsale_start: Date.now(),
  pubsale_end: Date.now() + 10000000000000,
};

describe("spl-vending-machine", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace
    .SplVendingMachine as Program<SplVendingMachine>;

  const authority = new anchor.web3.Keypair();
  const buyer = new anchor.web3.Keypair();

  let tokenMint: anchor.web3.PublicKey = null;
  let authorityMintAta: anchor.web3.PublicKey = null;

  before("initialize client", async () => {
    //fund wallets
    await airdrop(authority.publicKey, program.provider.connection);
    await airdrop(buyer.publicKey, program.provider.connection);
    //create new spl token
    const token = await mintToken(
      program.provider,
      authority,
      authority,
      authority,
      config.tokens
    );
    //assign global variables
    tokenMint = token.tokenMint;
    authorityMintAta = token.payerAta;

    console.log("client initialized");
  });

  it("should create machine", async () => {
    //create vending machine account
    const [vending_machine, _] = await getVendingMachinePda(
      authority.publicKey,
      program.programId
    );
    //price per allocation (ticket price in SOL)
    const ppa = new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL);
    //amount of spl tokens reserved for those with tickets
    const ticketAllocation = new anchor.BN(config.allocations);
    //build instruction
    const instruction = await program.methods
      .createMachine(
        new anchor.BN(config.ppa),
        new anchor.BN(config.ppt),
        new anchor.BN(config.allocations),
        new anchor.BN(config.presale_start),
        new anchor.BN(config.presale_end),
        new anchor.BN(config.pubsale_start),
        new anchor.BN(config.pubsale_end)
      )
      .accounts({
        vendingMachine: vending_machine,
        authority: authority.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        splMint: tokenMint,
      })
      .instruction();
    //build transaction
    const transaction = new anchor.web3.Transaction();
    transaction.add(instruction);
    //broadcast transaction to the network
    const tx = await program.provider.sendAndConfirm(transaction, [authority]);
    //log outputs
    console.log(`
    Created Vending Machine
    Signature: ${tx}
    Machine Id: ${vending_machine.toString()}
    `);
  });

  it("should fund machine", async () => {
    //get ata of required accounts
    const vending_machine = (await program.account.vendingMachine.all())[0];
    const [vending_machine_ata, _] = await getAtaForMint(
      vending_machine.publicKey,
      tokenMint
    );
    const [authority_ata, _1] = await getAtaForMint(
      authority.publicKey,
      tokenMint
    );
    //spl_stock amount
    let amount = new anchor.BN(config.tokens);
    //build instruction
    const instruction = await program.methods
      .fundMachine(amount)
      .accounts({
        authority: authority.publicKey,
        vendingMachine: vending_machine.publicKey,
        splMint: tokenMint,
        systemProgram: anchor.web3.SystemProgram.programId,
        authoritySplAta: authority_ata,
        vendingMachineSplAta: vending_machine_ata,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .instruction();
    //build transaction
    const transaction = new anchor.web3.Transaction();
    transaction.add(instruction);
    //broadcast transaction to the network
    const tx = await program.provider.sendAndConfirm(transaction, [authority]);
    //log outputs
    console.log(`
    Funded Vending Machine
    Signature: ${tx}
    Machine Id: ${vending_machine.publicKey.toString()}
    `);
    //check the funds have changed
    const funded_vending_machine = (
      await program.account.vendingMachine.all()
    )[0];
    assert.ok(funded_vending_machine.account.splStock.toNumber() > 0);
  });

  it("should buy allocation ticket", async () => {
    const pre_authority_balance = await program.provider.connection.getBalance(
      authority.publicKey
    );
    const vending_machine = (await program.account.vendingMachine.all())[0];
    const ticket = new anchor.web3.Keypair();
    //amount of allocation spots to buy
    const amount = new anchor.BN(1);
    //build instrcution
    const instruction = await program.methods
      .buyTicket(amount)
      .accounts({
        authority: authority.publicKey,
        buyer: buyer.publicKey,
        vendingMachine: vending_machine.publicKey,
        ticket: ticket.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .instruction();
    //build transaction
    const transaction = new anchor.web3.Transaction();
    transaction.add(instruction);
    //broadcast transaction to the network
    const tx = await program.provider.sendAndConfirm(transaction, [
      buyer,
      ticket,
    ]);
    const ticket_account = (await program.account.ticket.all())[0].account;
    //log outputs
    console.log(`
    Buy Allocation Tickets (1)
    Signature: ${tx}
    Machine Id: ${vending_machine.publicKey.toString()}
    Ticket:
        ID: ${ticket.publicKey.toString()}
        unspent: ${ticket_account.unspent}
        spent: ${ticket_account.spent}
    `);
    const post_vending_machine = (
      await program.account.vendingMachine.all()
    )[0];

    const post_authority_balance = await program.provider.connection.getBalance(
      authority.publicKey
    );

    assert.ok(
      post_vending_machine.account.ticketsSold >
        vending_machine.account.ticketsSold,
      "sold tickets have increased"
    );

    assert.ok(
      post_authority_balance > pre_authority_balance,
      "authority balance has increased from ticket sale"
    );
  });

  it("should buy SPL token without ticket", async () => {
    const vending_machine = (await program.account.vendingMachine.all())[0];
    const [buyer_spl_ata, _] = await getAtaForMint(buyer.publicKey, tokenMint);
    const [vending_machine_ata, _1] = await getAtaForMint(
      vending_machine.publicKey,
      tokenMint
    );
    const instruction = await program.methods
      .buySpl(new anchor.BN(1))
      .accounts({
        vendingMachine: vending_machine.publicKey,
        authority: authority.publicKey,
        signer: buyer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        splMint: tokenMint,
        buyerSplAta: buyer_spl_ata,
        vendingMachineSplAta: vending_machine_ata,
      })
      .instruction();
    //build transaction
    const transaction = new anchor.web3.Transaction();
    transaction.add(instruction);
    //broadcast transaction to the network
    const tx = await program.provider.sendAndConfirm(transaction, [buyer]);
    const vending_machine_account = (
      await program.account.vendingMachine.all()
    )[0].account;
    //log outputs
    console.log(`
    Buy SPL with ticket (1)
    Signature: ${tx}
    Machine: 
        ID: ${vending_machine.publicKey.toString()}
        Tickets Bought: ${vending_machine_account.ticketsSold}
        SPL Remaining: ${vending_machine_account.splStock}
        SPL Allocation Remaining: ${vending_machine_account.ticketAllocation}
    `);
  });

  it("should buy SPL token with ticket", async () => {
    const vending_machine = (await program.account.vendingMachine.all())[0];
    const [buyer_spl_ata, _] = await getAtaForMint(buyer.publicKey, tokenMint);
    const [vending_machine_ata, _1] = await getAtaForMint(
      vending_machine.publicKey,
      tokenMint
    );
    const ticket = (await program.account.ticket.all())[0];
    const instruction = await program.methods
      .buySplWithTicket(new anchor.BN(1))
      .accounts({
        vendingMachine: vending_machine.publicKey,
        ticket: ticket.publicKey,
        authority: authority.publicKey,
        signer: buyer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        splMint: tokenMint,
        buyerSplAta: buyer_spl_ata,
        vendingMachineSplAta: vending_machine_ata,
      })
      .instruction();
    //build transaction
    const transaction = new anchor.web3.Transaction();
    transaction.add(instruction);
    //broadcast transaction to the network
    const tx = await program.provider.sendAndConfirm(transaction, [buyer]);
    const ticket_account = (await program.account.ticket.all())[0].account;
    const vending_machine_account = (
      await program.account.vendingMachine.all()
    )[0].account;
    //log outputs
    console.log(`
    Buy SPL with ticket (1)
    Signature: ${tx}
    Machine: 
        ID: ${vending_machine.publicKey.toString()}
        Tickets Bought: ${vending_machine_account.ticketsSold}
        SPL Remaining: ${vending_machine_account.splStock}
        SPL Allocation Remaining: ${vending_machine_account.ticketAllocation}
    Ticket:
        ID: ${ticket.publicKey.toString()}
        unspent: ${ticket_account.unspent}
        spent: ${ticket_account.spent}
    `);
  });
});
