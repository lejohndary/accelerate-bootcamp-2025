use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Burn, MintTo},
    token_interface::{Mint, TokenAccount, Token2022},
};

declare_id!("6y2JmXvbBisg2pS4p388BjiGeP1xWoEE1xgJjrENGYMq");

#[program]
pub mod project_5_capstone {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        dispute_period_seconds: i64,
        dispute_threshold: u64,
        pool_name: String,
        pool_description: String,
        end_time: i64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let bump = ctx.bumps.pool;

        // Initialize pool data
        pool.authority = ctx.accounts.authority.key();
        pool.yes_mint = ctx.accounts.yes_mint.key();
        pool.no_mint = ctx.accounts.no_mint.key();
        pool.total_yes_tokens = 0;
        pool.total_no_tokens = 0;
        pool.solution_proposed = false;
        pool.solution_winner = None;
        pool.dispute_period_seconds = dispute_period_seconds;
        pool.dispute_threshold = dispute_threshold;
        pool.is_disputed = false;
        pool.is_finalized = false;
        pool.bump = bump;
        pool.name = pool_name;
        pool.description = pool_description;
        pool.end_time = end_time;
        pool.created_at = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn mint_prediction_tokens(
        ctx: Context<MintPredictionTokens>,
        amount: u64,
        prediction: bool, // true for YES, false for NO
    ) -> Result<()> {
        // Check if predictions are still allowed
        let current_time = Clock::get()?.unix_timestamp;
        require!(current_time < ctx.accounts.pool.end_time, BettingPoolError::BettingPeriodEnded);
        
        // Create pool seeds for signing
        let pool = &ctx.accounts.pool;
        let pool_seeds = &[
            b"pool".as_ref(),
            pool.authority.as_ref(),
            &[pool.bump],
        ];
        let signer = &[&pool_seeds[..]];
        
        // Get mint and token account based on prediction
        let mint = if prediction {
            ctx.accounts.yes_mint.to_account_info()
        } else {
            ctx.accounts.no_mint.to_account_info()
        };
        
        let token_account = if prediction {
            ctx.accounts.user_yes_token.to_account_info()
        } else {
            ctx.accounts.user_no_token.to_account_info()
        };
        
        // Create CPI context for minting
        let cpi_accounts = MintTo {
            mint,
            to: token_account,
            authority: ctx.accounts.pool.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        
        // Mint the tokens
        token::mint_to(cpi_ctx, amount)?;
        
        // Update pool token counters
        let pool = &mut ctx.accounts.pool;
        if prediction {
            pool.total_yes_tokens = pool.total_yes_tokens.checked_add(amount).unwrap();
        } else {
            pool.total_no_tokens = pool.total_no_tokens.checked_add(amount).unwrap();
        }
        
        Ok(())
    }

    pub fn burn_prediction_tokens(
        ctx: Context<BurnPredictionTokens>,
        amount: u64,
        prediction: bool, // true for YES, false for NO
    ) -> Result<()> {
        // Check if predictions are still allowed
        let current_time = Clock::get()?.unix_timestamp;
        require!(current_time < ctx.accounts.pool.end_time, BettingPoolError::BettingPeriodEnded);
        
        // Get mint and token account based on prediction
        let mint = if prediction {
            ctx.accounts.yes_mint.to_account_info()
        } else {
            ctx.accounts.no_mint.to_account_info()
        };
        
        let token_account = if prediction {
            ctx.accounts.user_yes_token.to_account_info()
        } else {
            ctx.accounts.user_no_token.to_account_info()
        };
        
        // Create CPI context for burning
        let cpi_accounts = Burn {
            mint,
            from: token_account,
            authority: ctx.accounts.user.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        // Burn the tokens
        token::burn(cpi_ctx, amount)?;
        
        // Update pool token counters
        let pool = &mut ctx.accounts.pool;
        if prediction {
            pool.total_yes_tokens = pool.total_yes_tokens.checked_sub(amount).unwrap();
        } else {
            pool.total_no_tokens = pool.total_no_tokens.checked_sub(amount).unwrap();
        }
        
        Ok(())
    }

    pub fn propose_solution(
        ctx: Context<ProposeSolution>,
        winner: bool, // true for YES, false for NO
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require!(!pool.solution_proposed, BettingPoolError::SolutionAlreadyProposed);
        
        // Check if proposing solution is allowed (only after end_time)
        let current_time = Clock::get()?.unix_timestamp;
        require!(current_time >= pool.end_time, BettingPoolError::BettingPeriodNotEnded);
        
        pool.solution_proposed = true;
        pool.solution_winner = Some(winner);
        pool.dispute_period_start = Clock::get()?.unix_timestamp;
        pool.dispute_period_end = pool.dispute_period_start + pool.dispute_period_seconds;
        
        Ok(())
    }

    pub fn dispute_solution(
        ctx: Context<DisputeSolution>
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;
        
        require!(pool.solution_proposed, BettingPoolError::NoSolutionProposed);
        require!(!pool.is_disputed, BettingPoolError::AlreadyDisputed);
        require!(clock.unix_timestamp <= pool.dispute_period_end, BettingPoolError::DisputePeriodEnded);
        
        // Check if disputer has enough tokens of the losing side
        let winner = pool.solution_winner.unwrap();
        let disputer_tokens = if winner {
            // If YES won, then NO is disputing
            ctx.accounts.user_no_token.amount
        } else {
            // If NO won, then YES is disputing
            ctx.accounts.user_yes_token.amount
        };
        
        require!(disputer_tokens >= pool.dispute_threshold, BettingPoolError::InsufficientTokensForDispute);
        
        pool.is_disputed = true;
        pool.disputer = Some(ctx.accounts.user.key());
        
        Ok(())
    }

    pub fn resolve_dispute(
        ctx: Context<ResolveDispute>,
        new_winner: bool, // true for YES, false for NO
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        
        require!(pool.is_disputed, BettingPoolError::NotDisputed);
        require!(!pool.is_finalized, BettingPoolError::AlreadyFinalized);
        
        // Only the authority can resolve disputes
        require!(ctx.accounts.authority.key() == pool.authority, BettingPoolError::Unauthorized);
        
        // Set the new winner
        pool.solution_winner = Some(new_winner);
        pool.is_disputed = false;
        
        // Reset dispute period to allow for another round of disputes
        pool.dispute_period_start = Clock::get()?.unix_timestamp;
        pool.dispute_period_end = pool.dispute_period_start + pool.dispute_period_seconds;
        
        Ok(())
    }

    pub fn finalize_pool(ctx: Context<FinalizePool>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;
        
        require!(pool.solution_proposed, BettingPoolError::NoSolutionProposed);
        require!(!pool.is_finalized, BettingPoolError::AlreadyFinalized);
        require!(clock.unix_timestamp > pool.dispute_period_end, BettingPoolError::DisputePeriodNotEnded);
        require!(!pool.is_disputed, BettingPoolError::PoolIsDisputed);
        
        // Set the pool as finalized
        pool.is_finalized = true;
        
        Ok(())
    }

    pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        
        require!(pool.is_finalized, BettingPoolError::PoolNotFinalized);
        
        let winner = pool.solution_winner.unwrap();
        
        // Check if user holds winning tokens
        let winning_token_account = if winner {
            &ctx.accounts.user_yes_token
        } else {
            &ctx.accounts.user_no_token
        };
        
        require!(winning_token_account.amount > 0, BettingPoolError::NoWinningTokens);
        
        // Calculate proportion of winnings
        let total_winning_tokens = if winner {
            pool.total_yes_tokens
        } else {
            pool.total_no_tokens
        };
        
        let user_winning_proportion = winning_token_account.amount as f64 / total_winning_tokens as f64;
        
        // This is where you would implement token distribution logic
        // For example, transferring a share of the pool funds to the winner
        
        // For now, we'll just burn the winning tokens
        let mint = if winner {
            ctx.accounts.yes_mint.to_account_info()
        } else {
            ctx.accounts.no_mint.to_account_info()
        };
        
        let token_account = if winner {
            ctx.accounts.user_yes_token.to_account_info()
        } else {
            ctx.accounts.user_no_token.to_account_info()
        };
        
        let cpi_accounts = Burn {
            mint,
            from: token_account,
            authority: ctx.accounts.user.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::burn(cpi_ctx, winning_token_account.amount)?;
        
        msg!("User claimed winnings: {:.2}% of the pool", user_winning_proportion * 100.0);
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        seeds = [b"pool", authority.key().as_ref()],
        bump,
        space = BettingPool::space()
    )]
    pub pool: Account<'info, BettingPool>,
    
    #[account(
        init,
        payer = authority,
        mint::decimals = 6,
        mint::authority = pool,
        mint::token_program = token_program
    )]
    pub yes_mint: InterfaceAccount<'info, Mint>,
    
    #[account(
        init,
        payer = authority,
        mint::decimals = 6,
        mint::authority = pool,
        mint::token_program = token_program
    )]
    pub no_mint: InterfaceAccount<'info, Mint>,
    
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(amount: u64, prediction: bool)]
pub struct MintPredictionTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"pool", pool.authority.as_ref()],
        bump = pool.bump
    )]
    pub pool: Account<'info, BettingPool>,
    
    #[account(
        mut,
        constraint = yes_mint.key() == pool.yes_mint
    )]
    pub yes_mint: InterfaceAccount<'info, Mint>,
    
    #[account(
        mut,
        constraint = no_mint.key() == pool.no_mint
    )]
    pub no_mint: InterfaceAccount<'info, Mint>,
    
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = yes_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_yes_token: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = no_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_no_token: InterfaceAccount<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(amount: u64, prediction: bool)]
pub struct BurnPredictionTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"pool", pool.authority.as_ref()],
        bump = pool.bump
    )]
    pub pool: Account<'info, BettingPool>,
    
    #[account(
        mut,
        constraint = yes_mint.key() == pool.yes_mint
    )]
    pub yes_mint: InterfaceAccount<'info, Mint>,
    
    #[account(
        mut,
        constraint = no_mint.key() == pool.no_mint
    )]
    pub no_mint: InterfaceAccount<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = yes_mint,
        associated_token::authority = user
    )]
    pub user_yes_token: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = no_mint,
        associated_token::authority = user
    )]
    pub user_no_token: InterfaceAccount<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(winner: bool)]
pub struct ProposeSolution<'info> {
    #[account(
        constraint = authority.key() == pool.authority @ BettingPoolError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"pool", pool.authority.as_ref()],
        bump = pool.bump
    )]
    pub pool: Account<'info, BettingPool>,
}

#[derive(Accounts)]
pub struct DisputeSolution<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"pool", pool.authority.as_ref()],
        bump = pool.bump
    )]
    pub pool: Account<'info, BettingPool>,
    
    #[account(
        mut,
        associated_token::mint = yes_mint,
        associated_token::authority = user
    )]
    pub user_yes_token: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = no_mint,
        associated_token::authority = user
    )]
    pub user_no_token: InterfaceAccount<'info, TokenAccount>,
    
    #[account(constraint = yes_mint.key() == pool.yes_mint)]
    pub yes_mint: InterfaceAccount<'info, Mint>,
    
    #[account(constraint = no_mint.key() == pool.no_mint)]
    pub no_mint: InterfaceAccount<'info, Mint>,
}

#[derive(Accounts)]
#[instruction(new_winner: bool)]
pub struct ResolveDispute<'info> {
    #[account(
        constraint = authority.key() == pool.authority @ BettingPoolError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"pool", pool.authority.as_ref()],
        bump = pool.bump
    )]
    pub pool: Account<'info, BettingPool>,
}

#[derive(Accounts)]
pub struct FinalizePool<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"pool", pool.authority.as_ref()],
        bump = pool.bump
    )]
    pub pool: Account<'info, BettingPool>,
}

#[derive(Accounts)]
pub struct ClaimWinnings<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"pool", pool.authority.as_ref()],
        bump = pool.bump
    )]
    pub pool: Account<'info, BettingPool>,
    
    #[account(mut, constraint = yes_mint.key() == pool.yes_mint)]
    pub yes_mint: InterfaceAccount<'info, Mint>,
    
    #[account(mut, constraint = no_mint.key() == pool.no_mint)]
    pub no_mint: InterfaceAccount<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = yes_mint,
        associated_token::authority = user
    )]
    pub user_yes_token: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = no_mint,
        associated_token::authority = user
    )]
    pub user_no_token: InterfaceAccount<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token2022>,
}

#[account]
pub struct BettingPool {
    pub authority: Pubkey,
    pub yes_mint: Pubkey,
    pub no_mint: Pubkey,
    pub total_yes_tokens: u64,
    pub total_no_tokens: u64,
    pub solution_proposed: bool,
    pub solution_winner: Option<bool>, // true for YES, false for NO
    pub dispute_period_start: i64,
    pub dispute_period_end: i64,
    pub dispute_period_seconds: i64,
    pub dispute_threshold: u64,
    pub is_disputed: bool,
    pub is_finalized: bool,
    pub bump: u8,
    pub name: String,
    pub description: String,
    pub end_time: i64,
    pub created_at: i64,
    pub disputer: Option<Pubkey>,
}

impl BettingPool {
    pub fn space() -> usize {
        8 +  // discriminator
        32 + // authority: Pubkey
        32 + // yes_mint: Pubkey
        32 + // no_mint: Pubkey
        8 +  // total_yes_tokens: u64
        8 +  // total_no_tokens: u64
        1 +  // solution_proposed: bool
        1 + 1 + // solution_winner: Option<bool>
        8 +  // dispute_period_start: i64
        8 +  // dispute_period_end: i64
        8 +  // dispute_period_seconds: i64
        8 +  // dispute_threshold: u64
        1 +  // is_disputed: bool
        1 +  // is_finalized: bool
        1 +  // bump: u8
        4 + 32 + // name: String (max 32 chars)
        4 + 256 + // description: String (max 256 chars)
        8 +  // end_time: i64
        8 +  // created_at: i64
        1 + 32  // disputer: Option<Pubkey>
    }
}

#[error_code]
pub enum BettingPoolError {
    #[msg("Solution has already been proposed")]
    SolutionAlreadyProposed,
    #[msg("No solution has been proposed yet")]
    NoSolutionProposed,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Dispute period has ended")]
    DisputePeriodEnded,
    #[msg("Dispute period has not ended yet")]
    DisputePeriodNotEnded,
    #[msg("Pool is already disputed")]
    AlreadyDisputed,
    #[msg("Pool is already finalized")]
    AlreadyFinalized,
    #[msg("Pool is currently disputed")]
    PoolIsDisputed,
    #[msg("Pool has not been finalized yet")]
    PoolNotFinalized,
    #[msg("Insufficient tokens for dispute")]
    InsufficientTokensForDispute,
    #[msg("The pool is not disputed")]
    NotDisputed,
    #[msg("Betting period has ended")]
    BettingPeriodEnded,
    #[msg("Betting period has not ended yet")]
    BettingPeriodNotEnded,
    #[msg("User holds no winning tokens")]
    NoWinningTokens,
}
