use anchor_lang::prelude::*;
use anchor_lang::solana_program::account_info::Account;
use anchor_spl::token::{self, SetAuthority, TokenAccount, Transfer};
use check_creator::check_creator;
use check_creator::cpi::accounts::{CancelCheck, CashCheck, CreateEmptyCheck};
use check_creator::program::check_creator;
use check_creator::{self, Check};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod prize_pool_contract {
    use super::*;

    pub fn initialize(ctx: Context<InitializePlayerVault>, amount:u64, nonce:u8) -> ProgramResult {
        let game = &mut ctx.accounts.game;
        game.match_address = *game.to_account_info().key;
        game.player_1_token_account = *ctx.accounts.player_x.key;
        Ok(())
        //Create Empty Check for the first player
        let cpi_program = ctx.accounts.match_manager.to_account_info();
        let cpi_accounts = ctx.check;
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        check_creator::cpi::create_empty_check(cpi_ctx, amount, nonce);
    }
}

//Adds data for player 2 and starts the match with given seed 
#[derive(Accounts)]
pub struct InitializeMatch<'info> {
    #[account(signer)]
    player_2_vault: AccountInfo<'info>,
    #[account(mut, constraint = game.winner_address != Pubkey::default() && game.player_1_token_account != Pubkey::default())]
    game: Account<'info, Match>,
}

//Gets player 1 data and adds it to match account
#[derive(Accounts)]
pub struct InitializePlayerVault<'info> {
    #[account(zero)]
    game: Account<'info, Match>,
    #[account(zero)]
    player_1_check: Account<'info, Check>,
    //Vault made for storing wager amount
    #[account(mut, has_one = player_wallet)]
    player_1_vault: Account<'info, TokenAccount>,,
    player_1_wallet: AccountInfo<'info>,
    match_manager: Program<'info, check_creator>,
}
    

#[derive(Accounts)]
pub struct ConcludeMatch<'info> {
    game: ProgramAccount<'info, Match>,
    #[account(mut, has_one = authority)]
    winner_address: Pubkey,
    authority: Signer<'info>,
}

#[account]
#[derive(Default)]
pub struct Match {
    player_1_token_account: Pubkey,
    player_2_token_account: Pubkey,
    wager_amount: u64,
    prize_settled: bool,
    match_address: Pubkey,
    winner_address: Pubkey,
}
