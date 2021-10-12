const anchor = require('@project-serum/anchor');
const serumCmn = require("@project-serum/common");
const assert = require("assert");
const { TOKEN_PROGRAM_ID } = require("@solana/spl-token");

describe('prize_pool_contract', () => {
  
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());
  const program = anchor.workspace.PrizePoolContract;
  const check1 = anchor.web3.Keypair.generate();
  const check2 = anchor.web3.Keypair.generate();
  const vault1 = anchor.web3.Keypair.generate();
  const vault2 = anchor.web3.Keypair.generate();
  const match = anchor.web3.Keypair.generate();

  let mint = null;
  let god = null;
  let receiver = null;

  it("Sets up initial test state", async () => {
    const [_mint, _god] = await serumCmn.createMintAndVault(
      program.provider,
      new anchor.BN(1000000)
    );
    mint = _mint;
    god = _god;

    receiver = await serumCmn.createTokenAccount(
      program.provider,
      mint,
      program.provider.wallet.publicKey
    );
  });

  let check1Signer = null;
  let check2Signer = null;

  it('Ready Player one!', async () => {
    // Add your test here.
    let [_checkSigner, nonce] = await anchor.web3.PublicKey.findProgramAddress(
      [check1.publicKey.toBuffer()],
      program.programId
    );
    check1Signer = _checkSigner;
    
    const tx = await program.rpc.registerPlayer(
      new anchor.BN(1000),
      nonce,
      {
        accounts: {
          playerCheck: check1.publicKey,
          playerVault: vault1.publicKey,
          checkSigner: check1Signer,
          from: god,
          owner: program.provider.wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        signers: [check1, vault1],
        instructions: [
          await program.account.check.createInstruction(check1, 300),
          ...(await serumCmn.createTokenAccountInstrs(
            program.provider,
            vault1.publicKey,
            mint,
            check1Signer
          )),
        ],
      }
    );

    const checkAccount = await program.account.check.fetch(check1.publicKey);
    assert.ok(checkAccount.from.equals(god));
    assert.ok(checkAccount.amount.eq(new anchor.BN(1000)));
    assert.ok(checkAccount.vault.equals(vault1.publicKey));
    assert.ok(checkAccount.nonce === nonce);
    assert.ok(checkAccount.burned === false);

    let vaultAccount = await serumCmn.getTokenAccount(
      program.provider,
      checkAccount.vault
    );
    assert.ok(vaultAccount.amount.eq(new anchor.BN(1000)));

    console.log("Your transaction signature", tx);
  });

  it('Ready Player two', async () => {
    // Add your test here.
    let [_checkSigner, nonce] = await anchor.web3.PublicKey.findProgramAddress(
      [check2.publicKey.toBuffer()],
      program.programId
    );
    check2Signer = _checkSigner;
    
    const tx = await program.rpc.registerPlayer(
      new anchor.BN(1000),
      nonce,
      {
        accounts: {
          playerCheck: check2.publicKey,
          playerVault: vault2.publicKey,
          checkSigner: check2Signer,
          from: god,
          owner: program.provider.wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        signers: [check2, vault2],
        instructions: [
          await program.account.check.createInstruction(check2, 300),
          ...(await serumCmn.createTokenAccountInstrs(
            program.provider,
            vault2.publicKey,
            mint,
            check2Signer
          )),
        ],
      }
    );
    
    const checkAccount = await program.account.check.fetch(check2.publicKey);
    assert.ok(checkAccount.from.equals(god));
    assert.ok(checkAccount.amount.eq(new anchor.BN(1000)));
    assert.ok(checkAccount.vault.equals(vault2.publicKey));
    assert.ok(checkAccount.nonce === nonce);
    assert.ok(checkAccount.burned === false);
    
    let vaultAccount = await serumCmn.getTokenAccount(
      program.provider,
      checkAccount.vault
    );
    assert.ok(vaultAccount.amount.eq(new anchor.BN(1000)));
    console.log("Your transaction signature", tx);
  });

  /**
   * Following calls are made from backend where the actual matchmaking happens:
   * 
   * @params In order to set up a match, both check accounts public keys need to be provided
   * !note dependencies can be derived using the fetch method on program account
   */
  it('Initialize Match', async () => {
    const tx = await program.rpc.startMatch(
      {
        accounts: {
          authority: program.provider.wallet.publicKey,
          game: match.publicKey,
          player1Check: check1.publicKey,
          player2Check: check2.publicKey,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        signers: [match],
        instructions: [await program.account.match.createInstruction(match)]
      }
    );
    console.log("transaction: ", tx);

    const gameData = await program.account.match.fetch(match.publicKey);
    assert.ok(gameData.player1Check.equals(check1.publicKey));
    assert.ok(gameData.player2Check.equals(check2.publicKey));
    assert.ok(gameData.matchAddress.equals(match.publicKey));
    assert.ok(gameData.wagerAmount.eq(new anchor.BN(2000)));
    assert.ok(gameData.prizeSettled == false);
    console.log("Match is set successfully, on Address ", gameData.matchAddress);
  });

  it('Conclude Match', async () => {
    let winner_check = check1.publicKey;
    let loser_check = check2.publicKey;
    let winner = await program.account.check.fetch(winner_check);
    let loser = await program.account.check.fetch(loser_check);

    let [loserCheckPDA, _nonce] = await anchor.web3.PublicKey.findProgramAddress(
      [loser_check.toBuffer()],
      program.programId
    );
    const tx = await program.rpc.concludeMatch(
      {
        accounts: {
          game: match.publicKey,
          authority: program.provider.wallet.publicKey,
          winnerCheck: winner_check,
          winnerVault: winner.vault,
          loserCheck: loser_check,
          loserVault: loser.vault,
          loserCheckSigner:loserCheckPDA,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        }
      }
    );
    console.log("transaction: ", tx);

    const gameData = await program.account.match.fetch(match.publicKey);
    assert.ok(gameData.player1Check.equals(check1.publicKey));
    assert.ok(gameData.player2Check.equals(check2.publicKey));
    assert.ok(gameData.matchAddress.equals(match.publicKey));
    assert.ok(gameData.wagerAmount.eq(new anchor.BN(2000)));
    assert.ok(gameData.winnerCheck.equals(check1.publicKey));
    assert.ok(gameData.prizeSettled == false);
    console.log("Match Concluded, on Address ", gameData.matchAddress);

    const winnerCheckData = await program.account.check.fetch(check2.publicKey);
    assert.ok(winnerCheckData.burned == true);
    assert.ok(winnerCheckData.amount.eq(new anchor.BN(0)));
  });

  it('Claim Prize', async () => {
    //Player 1 wins Player 1 claims
    let [VaultPDA, _nonce] = await anchor.web3.PublicKey.findProgramAddress(
      [check1.publicKey.toBuffer()],
      program.programId
    );
    
    await program.rpc.claimPrize({
      accounts: {
        game: match.publicKey,
        winnerCheck: check1.publicKey,
        winnerVault: vault1.publicKey,
        checkSigner: VaultPDA,
        to: receiver,
        owner: program.provider.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });

    //If claimed by player 1 then should fail
    const gameData = await program.account.match.fetch(match.publicKey);
    assert.ok(gameData.player1Check.equals(check1.publicKey));
    assert.ok(gameData.player2Check.equals(check2.publicKey));
    assert.ok(gameData.matchAddress.equals(match.publicKey));
    assert.ok(gameData.wagerAmount.eq(new anchor.BN(2000)));
    assert.ok(gameData.winnerCheck.equals(check1.publicKey));
    assert.ok(gameData.prizeSettled == true);
    console.log("Prize Claimed on Match ", gameData.matchAddress);
    
    const winnerCheckData = await program.account.check.fetch(check1.publicKey);
    assert.ok(winnerCheckData.burned == true);
    assert.ok(winnerCheckData.amount.eq(new anchor.BN(0)));
  });
});
