use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};
use std::convert::Into;

declare_id!("GDtbF64JLEbuoGxRTrVob8wef28sByS8GZn4JzSBJTCW");

#[program]
pub mod prize_pool_contract {
    use super::*;

    pub fn register_player(
        ctx: Context<InitializePlayerVault>,
        amount: u64,
        nonce: u8,
    ) -> ProgramResult {
        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info().clone(),
            to: ctx.accounts.player_vault.to_account_info().clone(),
            authority: ctx.accounts.owner.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Print the check.
        let check = &mut ctx.accounts.player_check;
        check.amount = amount;
        check.from = *ctx.accounts.from.to_account_info().key;
        check.vault = *ctx.accounts.player_vault.to_account_info().key;
        check.nonce = nonce;
        Ok(())
    }

    pub fn start_match(ctx: Context<StartMatch>) -> ProgramResult {
        let game = &mut ctx.accounts.game;
        game.match_address = *game.to_account_info().key;
        game.player_2_check = *ctx.accounts.player_2_check.to_account_info().key;
        game.player_1_check = *ctx.accounts.player_1_check.to_account_info().key;
        game.wager_amount = ctx.accounts.player_1_check.amount + ctx.accounts.player_2_check.amount;
        game.prize_settled = false;
        Ok(())
    }

    #[access_control(not_burned(&ctx.accounts.loser_check))]
    #[access_control(not_burned(&ctx.accounts.winner_check))]
    pub fn conclude_match(ctx: Context<ConcludeMatch>) -> ProgramResult {
        let game = &mut ctx.accounts.game;
        game.winner_check = *ctx.accounts.winner_check.to_account_info().key;

        let loser = &mut ctx.accounts.loser_check;
        let winner = &mut ctx.accounts.winner_check;
        loser.to = winner.vault;

        //Transfer the losers amount
        let cpi_accounts = Transfer {
            from: ctx.accounts.loser_vault.to_account_info().clone(),
            to: ctx.accounts.winner_vault.to_account_info().clone(),
            authority: ctx.accounts.loser_check_signer.clone(),
        };
        let seeds = &[loser.to_account_info().key.as_ref(), &[loser.nonce]];
        let signer = &[&seeds[..]];
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, loser.amount)?;

        // Burn the check for one time use.
        loser.burned = true;
        loser.amount = 0;
        winner.amount += loser.amount;
        Ok(())
    }

    #[access_control(not_burned(&ctx.accounts.winner_check))]
    pub fn claim_prize(ctx: Context<ClaimPrize>) -> ProgramResult {
        let game = &mut ctx.accounts.game;
        let winner = &mut ctx.accounts.winner_check;
        //Transfer the losers amount
        let cpi_accounts = Transfer {
            from: ctx.accounts.winner_vault.to_account_info().clone(),
            to: ctx.accounts.to.to_account_info().clone(),
            authority: ctx.accounts.check_signer.clone(),
        };
        let seeds = &[winner.to_account_info().key.as_ref(), &[winner.nonce]];
        let signer = &[&seeds[..]];
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, winner.amount)?;
        // Burn the check for one time use.
        winner.burned = true;
        winner.amount = 0;

        //TODO: discuss this flag
        game.prize_settled = true;
        Ok(())
    }
}

//Gets player 1 data and adds it to match account
#[derive(Accounts)]
pub struct InitializePlayerVault<'info> {
    #[account(zero)]
    player_check: Account<'info, Check>,
    // Check's token vault.
    #[account(mut, constraint = &player_vault.owner == check_signer.key)]
    player_vault: Account<'info, TokenAccount>,
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
                ctx.accounts.player_check.to_account_info().key.as_ref(),
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
    game: Account<'info, Match>,
    #[account(signer)]
    authority: AccountInfo<'info>,
    // #[account(mut, has_one = vault)
    // Add checks to check whether they have vaults
    player_1_check: Account<'info, Check>,
    player_2_check: Account<'info, Check>,
}

#[derive(Accounts)]
pub struct ConcludeMatch<'info> {
    #[account(mut)]
    game: Account<'info, Match>,
    #[account(signer)]
    authority: AccountInfo<'info>,
    #[account(mut, constraint = (winner_check.to_account_info().key == &game.player_1_check || winner_check.to_account_info().key == &game.player_2_check) && loser_check.to_account_info().key!= winner_check.to_account_info().key)]
    winner_check: Account<'info, Check>,
    #[account(mut, constraint = (loser_check.to_account_info().key == &game.player_1_check || loser_check.to_account_info().key == &game.player_2_check) && loser_check.to_account_info().key!= winner_check.to_account_info().key)]
    loser_check: Account<'info, Check>,

    #[account(mut, constraint = &loser_vault.owner == loser_check_signer.key)]
    loser_vault: Account<'info, TokenAccount>,
    // Program derived address for the check.
    #[account(
        seeds = [loser_check.to_account_info().key.as_ref()],
        bump = loser_check.nonce,
    )]
    loser_check_signer: AccountInfo<'info>,
    #[account(mut, constraint = winner_vault.to_account_info().key == &winner_check.vault)]
    winner_vault: Account<'info, TokenAccount>,
    token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ClaimPrize<'info> {
    #[account(mut)]
    game: Account<'info, Match>,
    #[account(mut, constraint = winner_check.to_account_info().key == &game.winner_check)]
    winner_check: Account<'info, Check>,
    #[account(mut)]
    winner_vault: AccountInfo<'info>,
    #[account(
        seeds = [winner_check.to_account_info().key.as_ref()],
        bump = winner_check.nonce,
    )]
    check_signer: AccountInfo<'info>,

    #[account(mut, has_one = owner)]
    to: Account<'info, TokenAccount>,
    #[account(signer)]
    owner: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
}

#[account]
#[derive(Default)]
pub struct Match {
    player_1_check: Pubkey,
    player_2_check: Pubkey,
    wager_amount: u64,
    prize_settled: bool,
    match_address: Pubkey,
    winner_check: Pubkey,
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
}

fn not_burned(check: &Check) -> Result<()> {
    if check.burned {
        return Err(ErrorCode::AlreadyBurned.into());
    }
    Ok(())
}
