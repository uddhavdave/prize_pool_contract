use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};
use std::convert::Into;

declare_id!("HmbTLCmaGvZhKnn1Zfa1JVnp7vkMV4DYVxPLWBVoN65L");

#[program]
pub mod check_creator {
    use super::*;
    pub fn create_empty_check(
        ctx: Context<CreateEmptyCheck>,
        amount: u64,
        nonce: u8,
    ) -> Result<()> {
        // Transfer funds to the check.
        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info().clone(),
            to: ctx.accounts.vault.to_account_info().clone(),
            authority: ctx.accounts.owner.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Print the check.
        let check = &mut ctx.accounts.check;
        check.amount = amount;
        check.from = *ctx.accounts.from.to_account_info().key;
        check.vault = *ctx.accounts.vault.to_account_info().key;
        check.nonce = nonce;
        Ok(())
    }

    //Should only be called via Master
    #[access_control(not_burned(&ctx.accounts.check))]
    #[access_control(is_authorized(&ctx.accounts.check))]
    pub fn cash_check(ctx: Context<CashCheck>, claimer: Pubkey) -> Result<()> {
        let seeds = &[
            ctx.accounts.check.to_account_info().key.as_ref(),
            &[ctx.accounts.check.nonce],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault.to_account_info().clone(),
            to: claimer.to_account_info().clone(),
            authority: ctx.accounts.check_signer.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, ctx.accounts.check.amount)?;
        // Burn the check for one time use.
        ctx.accounts.to = Some(claimer);
        ctx.accounts.check.burned = true;
        Ok(())
    }

    //Should only be called via Master
    #[access_control(not_burned(&ctx.accounts.check))]
    pub fn cancel_check(ctx: Context<CancelCheck>) -> Result<()> {
        let seeds = &[
            ctx.accounts.check.to_account_info().key.as_ref(),
            &[ctx.accounts.check.nonce],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault.to_account_info().clone(),
            to: ctx.accounts.from.to_account_info().clone(),
            authority: ctx.accounts.check_signer.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, ctx.accounts.check.amount)?;
        ctx.accounts.check.burned = true;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateEmptyCheck<'info> {
    // Check being created.
    #[account(zero)]
    check: Account<'info, Check>,
    // Check's token vault.
    #[account(mut, constraint = &vault.owner == check_signer.key)]
    vault: Account<'info, TokenAccount>,
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

impl<'info> CreateEmptyCheck<'info> {
    pub fn accounts(ctx: &Context<CreateEmptyCheck>, nonce: u8) -> Result<()> {
        let signer = Pubkey::create_program_address(
            &[ctx.accounts.check.to_account_info().key.as_ref(), &[nonce]],
            ctx.program_id,
        )
        .map_err(|_| ErrorCode::InvalidCheckNonce)?;
        if &signer != ctx.accounts.check_signer.to_account_info().key {
            return Err(ErrorCode::InvalidCheckSigner.into());
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CashCheck<'info> {
    #[account(mut, has_one = vault, has_one = to)]
    check: Account<'info, Check>,
    #[account(mut)]
    vault: AccountInfo<'info>,
    #[account(
        seeds = [check.to_account_info().key.as_ref()],
        bump = check.nonce,
    )]
    check_signer: AccountInfo<'info>,
    #[account(mut, has_one = owner)]
    to: Account<'info, TokenAccount>,
    #[account(signer)]
    owner: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CancelCheck<'info> {
    #[account(mut, has_one = vault, has_one = from)]
    check: Account<'info, Check>,
    #[account(mut)]
    vault: AccountInfo<'info>,
    #[account(
        seeds = [check.to_account_info().key.as_ref()],
        bump = check.nonce,
    )]
    check_signer: AccountInfo<'info>,
    #[account(mut, has_one = owner)]
    from: Account<'info, TokenAccount>,
    #[account(signer)]
    owner: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
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

fn is_authorized(check: &Check) -> Result<()> {
    if check.to == Pubkey::default() {
        return Err(ErrorCode::InvalidAuthClaim.into());
    }
    Ok(())
}
