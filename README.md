# Anchor Program for Crypunks

## Install and Test

-   Clone the Repo 
-   Run `solana-test-validator` in a new terminal
-   Run `anchor build` followed by `anchor-deploy`
-   Now close the terminal running the testnet
-   Run `anchor test` and verify

## Flow of Program 

1. `Vaults` (Token Accounts) get created with a `Check` account using `registerPlayer` RPC.
2. Once Backend decides the Match (1v1),  `startMatch` RPC is called with both player `Check` Accounts.
3. After Unreal engine decides a winner, Backend sends the respective Check(*winner's check pubkey*) via  `concludeMatch` RPC with both players `Vaults` and losers `Check` account and its `PDA`. *Please refer testscript for reference*
4. **NOTE**: Transfer of funds happen from Loser Vault → Winner Vault in the `concludeMatch` RPC.
5. Winner gets the Claim Option which will trigger `claimPrize` RPC. This call will transfer funds from Winner Vault → Winner Wallet.   
## Status of Features

- [X] Vault Generation RPCs
- [X] Match Making RPCs
- [ ] Wallet ↔ Vault Transfers
- [ ] Cancel Button RPC

