use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};
use std::convert::Into;

declare_id!("GDtbF64JLEbuoGxRTrVob8wef28sByS8GZn4JzSBJTCW");

#[program]
pub mod prize_pool_contract {
    use super::*;

    pub fn initialize(ctx: Context<InitializeMatch>) -> ProgramResult {
        let game = &mut ctx.accounts.game;
        game.match_address = *game.to_account_info().key;
        Ok(())
    }

    pub fn register_player_one(
        ctx: Context<InitializePlayerVault>,
        amount: u64,
        nonce: u8,
    ) -> ProgramResult {
        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info().clone(),
            to: ctx.accounts.player_1_vault.to_account_info().clone(),
            authority: ctx.accounts.owner.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        let game = &mut ctx.accounts.game;
        game.player_1_token_account = *ctx.accounts.player_1_vault.to_account_info().key;
        game.wager_amount += amount;

        // Print the check.
        let check = &mut ctx.accounts.player_1_check;
        check.amount = amount;
        check.from = *ctx.accounts.from.to_account_info().key;
        check.vault = *ctx.accounts.player_1_vault.to_account_info().key;
        check.nonce = nonce;
        Ok(())
    }

    pub fn register_player_two(ctx: Context<StartMatch>, amount: u64, nonce: u8) -> ProgramResult {
        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info().clone(),
            to: ctx.accounts.player_2_vault.to_account_info().clone(),
            authority: ctx.accounts.owner.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        let game = &mut ctx.accounts.game;
        game.player_2_token_account = *ctx.accounts.player_2_vault.to_account_info().key;
        game.wager_amount += amount;

        // Print the check.
        let check = &mut ctx.accounts.player_2_check;
        check.amount = amount;
        check.from = *ctx.accounts.from.to_account_info().key;
        check.vault = *ctx.accounts.player_2_vault.to_account_info().key;
        check.nonce = nonce;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeMatch<'info> {
    #[account(zero)]
    game: Account<'info, Match>,
    #[account(signer)]
    authority: AccountInfo<'info>,
}

//Gets player 1 data and adds it to match account
#[derive(Accounts)]
pub struct InitializePlayerVault<'info> {
    #[account(mut)]
    game: Account<'info, Match>,
    #[account(zero)]
    player_1_check: Account<'info, Check>,
    // Check's token vault.
    #[account(mut, constraint = &player_1_vault.owner == check_signer.key)]
    player_1_vault: Account<'info, TokenAccount>,
    // Program derived address for the check.
    check_signer: AccountInfo<'info>,
    // Token account the check is made from.
    #[account(mut, has_one = owner)]
    from: Account<'info, TokenAccount>,
    // Owner of the `from` token account.
    #[account(signer)]
    owner: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
}

impl<'info> InitializePlayerVault<'info> {
    pub fn accounts(ctx: &Context<InitializePlayerVault>, nonce: u8) -> Result<()> {
        let signer = Pubkey::create_program_address(
            &[
                ctx.accounts.player_1_check.to_account_info().key.as_ref(),
                &[nonce],
            ],
            ctx.program_id,
        )
        .map_err(|_| ErrorCode::InvalidCheckNonce)?;
        if &signer != ctx.accounts.check_signer.to_account_info().key {
            return Err(ErrorCode::InvalidCheckSigner.into());
        }
        Ok(())
    }
}

//Adds data for player 2 and starts the match with given seed
#[derive(Accounts)]
pub struct StartMatch<'info> {
    #[account(zero)]
    player_2_check: Account<'info, Check>,
    #[account(mut, constraint = &player_2_vault.owner == check_signer.key)]
    player_2_vault: Account<'info, TokenAccount>,
    // Program derived address for the check.
    check_signer: AccountInfo<'info>,
    #[account(mut, has_one = owner)]
    from: Account<'info, TokenAccount>,
    // Owner of the `from` token account.
    #[account(signer)]
    owner: AccountInfo<'info>,
    #[account(mut, constraint = game.winner_address == Pubkey::default() && game.player_1_token_account != Pubkey::default())]
    game: Account<'info, Match>,
    token_program: AccountInfo<'info>,
}

impl<'info> StartMatch<'info> {
    pub fn accounts(ctx: &Context<StartMatch>, amount: u64, nonce: u8) -> Result<()> {
        let signer = Pubkey::create_program_address(
            &[
                ctx.accounts.player_2_check.to_account_info().key.as_ref(),
                &[nonce],
            ],
            ctx.program_id,
        )
        .map_err(|_| ErrorCode::InvalidCheckNonce)?;
        if &signer != ctx.accounts.check_signer.to_account_info().key {
            return Err(ErrorCode::InvalidCheckSigner.into());
        }
        //Wager Amount should be same as player 2
        if ctx.accounts.game.wager_amount != amount {
            return Err(ErrorCode::InvalidWager.into());
        }
        Ok(())
    }
}
// #[derive(Accounts)]
// pub struct ConcludeMatch<'info> {
//     game: Account<'info, Match>,
//     #[account(mut, has_one = authority)]
//     winner_address: Account<'info>,
//     authority: Signer<'info>,
// }

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

#[account]
#[derive(Default)]
pub struct Check {
    from: Pubkey,
    to: Pubkey,
    amount: u64,
    vault: Pubkey,
    nonce: u8,
    burned: bool,
}

#[error]
pub enum ErrorCode {
    #[msg("The given nonce does not create a valid program derived address.")]
    InvalidCheckNonce,
    #[msg("The derived check signer does not match that which was given.")]
    InvalidCheckSigner,
    #[msg("The given check has already been burned.")]
    AlreadyBurned,
    #[msg("Unauthorized Claim Request")]
    InvalidAuthClaim,
    #[msg("Incompatible Wager")]
    InvalidWager,
}
