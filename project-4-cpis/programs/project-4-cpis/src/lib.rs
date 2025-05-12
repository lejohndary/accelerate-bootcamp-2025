use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{ self, Mint, TokenAccount, TokenInterface, TransferChecked, MintTo };

declare_id!("7qiThtk83oZCPeDQuooiUajsxrxeBs8DHQdA6kzqQJZs");

#[program]
pub mod project_4_cpis {
    use super::*;

    pub fn create_token_account(ctx: Context<CreateTokenAccount>) -> Result<()> {
        anchor_spl::associated_token::create(
            CpiContext::new(
                ctx.accounts.associated_token_program.to_account_info(),
                anchor_spl::associated_token::Create {
                    payer: ctx.accounts.payer.to_account_info(),
                    associated_token: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
            )
        )?;

        Ok(())
    }

    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
        let mint_cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_context = CpiContext::new(cpi_program, mint_cpi_accounts);

        let decimals = ctx.accounts.mint.decimals;
        token_interface::mint_to(cpi_context, amount)?;

        Ok(())
    }

    pub fn token_transfer(ctx: Context<TokenTransfer>, amount: u64) -> Result<()> {
        let transfer_cpi_accounts = TransferChecked {
            from: ctx.accounts.from.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();

        let cpi_context = CpiContext::new(cpi_program, transfer_cpi_accounts);

        let decimals = ctx.accounts.mint.decimals;
        token_interface::transfer_checked(cpi_context, amount, decimals)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateTokenAccount<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: This is the account that will own the token account
    pub owner: UncheckedAccount<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    
    #[account(mut)]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(mut)]
    pub mint_authority: Signer<'info>,
    
    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,
    
    #[account(mut)]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TokenTransfer<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub to: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub from: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}
