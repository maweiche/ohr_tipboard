import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { TipBoard } from "../target/types/tip_board"
import { LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { assert, expect } from "chai";

describe("tipboard", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  console.log('about to check for program', anchor.workspace)
  const program = anchor.workspace.TipBoard as Program<TipBoard>;
  console.log('program', program)
  const wallet = anchor.workspace.TipBoard.provider.wallet;
  console.log("wallet", wallet.publicKey.toString());
  const [tipboardAccount] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("tipboard")],
    program.programId
  );
  console.log("tipboard account", tipboardAccount.toString());
  const [tipboard] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("tipboard"), wallet.publicKey.toBuffer()],
    program.programId
  );
  console.log("tipboard account", tipboard.toString());

  const entryFee = new anchor.BN(0.5 * LAMPORTS_PER_SOL);

  it("Tipboard is initialized!", async () => {
    try{
    // Add your test here.
    const tx = await program.methods
      .initializeTipboard()
      .accounts({
        tipboardAccount: tipboardAccount,
        tipboard: tipboard,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

      console.log("Your transaction signature", tx);
    } catch (err) {
      console.log("err", err);
    }

    const tipboardAccountFetch = await program.account.tipboard.fetch(
      tipboardAccount
    );
    // decode the data
    const all_tipboards = tipboardAccountFetch.tipboards;
    console.log("all_tips", all_tipboards);

    // expect(
    //   all_tipboards.length.toString(),
    //   "0"
    // )
  });

  it("Add tip", async () => {
    try{
      const tipAmount = new anchor.BN(50);
      const timestamp = new anchor.BN(Date.now());
      const nftMint = "ARBofYtiiKzXGWaWQZhyvCw2eXzsSACy3taWQgLpdEbX";
      // Add your test here.
      const tx = await program.methods
        .addTip(
          tipAmount,
          timestamp,
          nftMint
        )
        .accounts({
          tipboard: tipboard,
          to: new PublicKey("7wK3jPMYjpZHZAghjersW6hBNMgi9VAGr75AhYRqR2n"),
          solUsdPriceAccount: new PublicKey("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix"),
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

        console.log("Your transaction signature", tx);
    } catch (err) {
      console.log("err", err);
    }

    const tipboardAccount = await program.account.tipboard.fetch(
      tipboard
    );
    // decode the data
    const all_tips = tipboardAccount.tips;
    console.log("all_tips", all_tips);
    
    const tipboardAccountBalance = await program.provider.connection.getBalance(
      tipboard
    );
    console.log('balance after adding tip', tipboardAccountBalance.toString());
    expect(
      tipboardAccount.tips.length.toString(),
      "1"
    )
  });

  // it("Withdraw tip", async () => {
  //   try{
  //     const tx = await program.methods
  //       .withdrawTips()
  //       .accounts({
  //         tipboard: tipboard,
  //       })
  //       .rpc();
  //   } catch (err) {
  //     console.log("err", err);
  //   }

  //   const tipboardAccountBalance = await program.provider.connection.getBalance(
  //     tipboard
  //   );
  //   console.log('balance after withdraw', tipboardAccountBalance.toString());
  //   expect(
  //     tipboardAccountBalance.toString(),
  //     "0"
  //   )
  // })

  // it("Reset Tipboard", async () => {
  //   try{
      
  //     // Add your test here.
  //     const tx = await program.methods
  //       .resetTipboard()
  //       .accounts({
  //         tipboard: tipboard,
  //       })
  //       .rpc();
  //   } catch (err) {
  //     console.log("err", err);
  //   }

  //   const tipboardAccount = await program.account.tipboard.fetch(
  //     tipboard
  //   );
  //   // decode the data
  //   const all_tips = tipboardAccount.tips;
  //   console.log("all_tips", all_tips);

  //   expect(
  //     tipboardAccount.tips.length.toString(),
  //     "0"
  //   )
  // });

  
});
