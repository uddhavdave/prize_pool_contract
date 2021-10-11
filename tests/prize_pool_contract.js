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

  it("Sets up initial test state", async () => {
    const [_mint, _god] = await serumCmn.createMintAndVault(
      program.provider,
      new anchor.BN(1000000)
    );
    mint = _mint;
    god = _god;
  });

  let check1Signer = null;
  let check2Signer = null;

  it('Initialize Match', async () => {
    const tx = await program.rpc.initialize({
      accounts: {
        authority: program.provider.wallet.publicKey,
        game: match.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [match],
      instructions: [await program.account.match.createInstruction(match)]
    })

    console.log("transaction: ", tx)
  });

  it('Ready Player one!', async () => {
    // Add your test here.
    let [_checkSigner, nonce] = await anchor.web3.PublicKey.findProgramAddress(
      [check1.publicKey.toBuffer()],
      program.programId
    );
    check1Signer = _checkSigner;
    
    const tx = await program.rpc.registerPlayerOne(
      new anchor.BN(1000),
      nonce,
      {
        accounts: {
          game: match.publicKey,
          player1Check: check1.publicKey,
          player1Vault: vault1.publicKey,
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

  it('Ready Player two and start match!', async () => {
    // Add your test here.
    let [_checkSigner, nonce] = await anchor.web3.PublicKey.findProgramAddress(
      [check2.publicKey.toBuffer()],
      program.programId
    );
    check2Signer = _checkSigner;
    
    const tx = await program.rpc.registerPlayerTwo(
      new anchor.BN(1000),
      nonce,
      {
        accounts: {
          game: match.publicKey,
          player2Check: check2.publicKey,
          player2Vault: vault2.publicKey,
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
    
    
    const gameData = await program.account.match.fetch(match.publicKey);
    assert.ok(gameData.player1TokenAccount.equals(vault1.publicKey));
    assert.ok(gameData.player2TokenAccount.equals(vault2.publicKey));
    assert.ok(gameData.matchAddress.equals(match.publicKey));
    assert.ok(gameData.wagerAmount.eq(new anchor.BN(2000)));
    assert.ok(gameData.prizeSettled == false);
    console.log("Match is set successfully, on Address ", gameData.matchAddress);
      
  });
});
