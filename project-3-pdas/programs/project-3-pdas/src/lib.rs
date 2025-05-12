use anchor_lang::prelude::*;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("94L2mJxVu6ZMmHaGsCHRQ65Kk2mea6aTnwWjSdfSsmBC");

#[program]
mod journal {
    use super::*;

    pub fn create_journal_entry(
        ctx: Context<CreateEntry>,
        title: String,
        message: String,
    ) -> Result<()> {
        let journal_entry = &mut ctx.accounts.journal_entry;
        journal_entry.owner = ctx.accounts.owner.key();
        journal_entry.title = title;
        journal_entry.message = message;
        journal_entry.timestamp = Clock::get()?.unix_timestamp as u64;
        Ok(())
    }

    pub fn update_journal_entry(
        ctx: Context<UpdateEntry>,
        title: String,
        message: String,
        entry_time: u8,
    ) -> Result<()> {
        msg!("Journal Entry Updated");
        msg!("Title: {}", title);
        msg!("Message: {}", message);

        let journal_entry = &mut ctx.accounts.journal_entry;
        journal_entry.message = message;

        Ok(())
    }

    pub fn delete_journal_entry(_ctx: Context<DeleteEntry>, title: String, entry_time: u8) -> Result<()> {
        msg!("Journal entry titled {} deleted", title);
        Ok(())
    }
}

#[account]
pub struct JournalEntryState {
    pub owner: Pubkey,
    pub title: String,
    pub message: String,
    pub timestamp: u64,
}

#[derive(Accounts)]
#[instruction(title: String, message: String)]
pub struct CreateEntry<'info> {
    #[account(
        init,
        seeds = [
            b"journal_entry", 
            title.as_bytes(), 
            owner.key().as_ref(),
            &Clock::get().unwrap().unix_timestamp.to_le_bytes()[0..4]
        ], 
        bump,
        payer = owner,
        space = 8 + 32 + 4 + title.len() + 4 + message.len() + 8
    )]
    pub journal_entry: Account<'info, JournalEntryState>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(title: String, message: String, entry_time: u8)]
pub struct UpdateEntry<'info> {
    #[account(
        mut,
        seeds = [
            b"journal_entry", 
            title.as_bytes(), 
            owner.key().as_ref(),
            &[entry_time]
        ], 
        bump,
        realloc = 8 + 32 + 4 + title.len() + 4 + message.len() + 8,
        realloc::payer = owner,
        realloc::zero = true,
    )]
    pub journal_entry: Account<'info, JournalEntryState>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(title: String, entry_time: u8)]
pub struct DeleteEntry<'info> {
    #[account( 
        mut, 
        seeds = [
            b"journal_entry", 
            title.as_bytes(), 
            owner.key().as_ref(),
            &[entry_time]
        ], 
        bump,
        close = owner,
    )]
    pub journal_entry: Account<'info, JournalEntryState>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}
