#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

declare_id!("37rseuTV1GSd7StJsevGwZ8VmvbvcAXw6jM6WB8kUndY");

#[program]
pub mod counter {
    use super::*;

    pub fn create_journal_entry(ctx: Context<CreateEntry>, title: String, message: String) -> Result<()> {
        msg!("Journal Entry Created");
        msg!("Title: {}", title);
        msg!("Message: {}", message);

        let journey_entry = &mut ctx.accounts.journal_entry;
        journey_entry.owner = *ctx.accounts.owner.key;
        journey_entry.title = title;
        journey_entry.message = message;
        Ok(())
    }

    pub fn update_journey_entry(ctx: Context<UpdateEntry>, title: String, message: String) -> Result<()> {
        msg!("Journal Entry Updated");
        msg!("Title: {}", title);
        msg!("Message: {}", message);

        let journey_entry = &mut ctx.accounts.journal_entry;
        journey_entry.message = message;
        Ok(())
    }

    pub fn delete_journey_entry(ctx: Context<DeleteEntry>, title: String) -> Result<()> {
        msg!("Journal entry titled {} deleted", title);
        Ok(())
    }


}

#[derive(Accounts)]
#[instruction(title: String, message: String)]
pub struct CreateEntry<'info> {
    #[account(
        init_if_needed,
        seeds = [title.as_bytes(), owner.key().as_ref()],
        bump,
        payer = owner,
        space = 8 + JournalEntryState::INIT_SPACE
    )]
    pub journal_entry: Account<'info, JournalEntryState>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(title: String, message: String)]
pub struct UpdateEntry<'info> {
    #[account(
        mut,
        seeds = [title.as_bytes(), owner.key().as_ref()],
        bump,
        realloc = 8 + JournalEntryState::INIT_SPACE + message.len(),
        realloc::payer = owner,
        realloc::zero = true,
    )]
    pub journal_entry: Account<'info, JournalEntryState>,

    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(title: String)]
pub struct DeleteEntry<'info> {
    #[account(
        mut,
        seeds = [title.as_bytes(), owner.key().as_ref()],
        bump,
        close = owner,
    )]

    pub journal_entry: Account<'info, JournalEntryState>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct JournalEntryState {
    pub owner: Pubkey,
    #[max_len(50)]
    pub title: String,
    #[max_len(1080)]
    pub message: String
}

