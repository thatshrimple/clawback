use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked},
};

declare_id!("25MSUtyW1pnuw2QDBDnDkmu57w4VeKAngE4sPSGTbe4E");

/// Clawback Protocol: Reversible USDC payments for AI agents

#[program]
pub mod clawback {
    use super::*;

    /// Initialize a new vault for an owner
    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.owner = ctx.accounts.owner.key();
        vault.usdc_mint = ctx.accounts.usdc_mint.key();
        vault.vault_token_account = ctx.accounts.vault_token_account.key();
        vault.next_payment_id = 0;
        vault.bump = ctx.bumps.vault;
        
        msg!("Vault initialized for owner: {}", vault.owner);
        Ok(())
    }

    /// Set spending policy for an agent
    pub fn set_policy(
        ctx: Context<SetPolicy>,
        agent: Pubkey,
        max_per_tx: u64,
        daily_limit: u64,
    ) -> Result<()> {
        let policy = &mut ctx.accounts.policy;
        policy.vault = ctx.accounts.vault.key();
        policy.agent = agent;
        policy.max_per_tx = max_per_tx;
        policy.daily_limit = daily_limit;
        policy.daily_spent = 0;
        policy.daily_window_start = Clock::get()?.unix_timestamp;
        policy.paused = false;
        policy.bump = ctx.bumps.policy;
        
        msg!("Policy set for agent {}", agent);
        Ok(())
    }

    /// Pause an agent (emergency stop)
    pub fn pause_agent(ctx: Context<ModifyPolicy>) -> Result<()> {
        ctx.accounts.policy.paused = true;
        msg!("Agent {} paused", ctx.accounts.policy.agent);
        Ok(())
    }

    /// Unpause an agent
    pub fn unpause_agent(ctx: Context<ModifyPolicy>) -> Result<()> {
        ctx.accounts.policy.paused = false;
        msg!("Agent {} unpaused", ctx.accounts.policy.agent);
        Ok(())
    }

    /// Set a recipient as trusted (skips cooling off)
    pub fn set_trusted_recipient(
        ctx: Context<SetTrustedRecipient>,
        recipient: Pubkey,
        trusted: bool,
    ) -> Result<()> {
        let trust = &mut ctx.accounts.trusted_recipient;
        trust.vault = ctx.accounts.vault.key();
        trust.recipient = recipient;
        trust.trusted = trusted;
        trust.bump = ctx.bumps.trusted_recipient;
        
        msg!("Recipient {} trusted: {}", recipient, trusted);
        Ok(())
    }

    /// Agent initiates a payment (enters cooling off period)
    pub fn initiate_payment(
        ctx: Context<InitiatePayment>,
        amount: u64,
        intent_hash: [u8; 32],
        intent_uri: String,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let policy = &mut ctx.accounts.policy;
        let payment = &mut ctx.accounts.payment;
        let clock = Clock::get()?;
        
        // Check policy not paused
        require!(!policy.paused, ClawbackError::AgentPaused);
        
        // Check max per tx
        if policy.max_per_tx > 0 {
            require!(amount <= policy.max_per_tx, ClawbackError::ExceedsMaxPerTx);
        }
        
        // Check and update daily limit
        if policy.daily_limit > 0 {
            if clock.unix_timestamp >= policy.daily_window_start + 86400 {
                policy.daily_spent = 0;
                policy.daily_window_start = clock.unix_timestamp;
            }
            require!(
                policy.daily_spent + amount <= policy.daily_limit,
                ClawbackError::ExceedsDailyLimit
            );
            policy.daily_spent += amount;
        }
        
        // Check if trusted recipient
        let is_trusted = ctx.accounts.trusted_recipient
            .as_ref()
            .map(|t| t.trusted)
            .unwrap_or(false);
        
        let cooling_off = if is_trusted { 0 } else { get_cooling_off_period(amount) };
        
        // Create payment record
        let payment_id = vault.next_payment_id;
        payment.id = payment_id;
        payment.vault = vault.key();
        payment.agent = ctx.accounts.agent.key();
        payment.recipient = ctx.accounts.recipient.key();
        payment.amount = amount;
        payment.initiated_at = clock.unix_timestamp;
        payment.finalize_at = clock.unix_timestamp + cooling_off;
        payment.intent_hash = intent_hash;
        payment.intent_uri = intent_uri;
        payment.status = PaymentStatus::Pending;
        payment.bump = ctx.bumps.payment;
        
        vault.next_payment_id += 1;
        
        // Transfer USDC from vault to escrow
        let owner_key = vault.owner;
        let bump = vault.bump;
        let seeds: &[&[u8]] = &[b"vault", owner_key.as_ref(), &[bump]];
        let signer_seeds = &[seeds];
        
        let decimals = ctx.accounts.usdc_mint.decimals;
        
        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.vault_token_account.to_account_info(),
                    mint: ctx.accounts.usdc_mint.to_account_info(),
                    to: ctx.accounts.payment_token_account.to_account_info(),
                    authority: vault.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
            decimals,
        )?;
        
        msg!("Payment {} initiated: {} to {}", payment_id, amount, payment.recipient);
        
        // If trusted, finalize immediately
        if cooling_off == 0 {
            payment.status = PaymentStatus::Finalized;
            
            let vault_key = vault.key();
            let payment_id_bytes = payment_id.to_le_bytes();
            let payment_bump = payment.bump;
            let payment_seeds: &[&[u8]] = &[b"payment", vault_key.as_ref(), &payment_id_bytes, &[payment_bump]];
            let payment_signer = &[payment_seeds];
            
            transfer_checked(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    TransferChecked {
                        from: ctx.accounts.payment_token_account.to_account_info(),
                        mint: ctx.accounts.usdc_mint.to_account_info(),
                        to: ctx.accounts.recipient_token_account.to_account_info(),
                        authority: payment.to_account_info(),
                    },
                    payment_signer,
                ),
                amount,
                decimals,
            )?;
            
            msg!("Payment {} finalized (trusted)", payment_id);
        }
        
        Ok(())
    }

    /// Owner claws back a pending payment
    pub fn clawback(ctx: Context<Clawback>) -> Result<()> {
        let payment = &mut ctx.accounts.payment;
        
        require!(payment.status == PaymentStatus::Pending, ClawbackError::PaymentNotPending);
        
        payment.status = PaymentStatus::ClawedBack;
        
        let vault_key = ctx.accounts.vault.key();
        let payment_id_bytes = payment.id.to_le_bytes();
        let payment_bump = payment.bump;
        let payment_seeds: &[&[u8]] = &[b"payment", vault_key.as_ref(), &payment_id_bytes, &[payment_bump]];
        let signer = &[payment_seeds];
        
        let decimals = ctx.accounts.usdc_mint.decimals;
        
        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.payment_token_account.to_account_info(),
                    mint: ctx.accounts.usdc_mint.to_account_info(),
                    to: ctx.accounts.vault_token_account.to_account_info(),
                    authority: payment.to_account_info(),
                },
                signer,
            ),
            payment.amount,
            decimals,
        )?;
        
        msg!("Payment {} clawed back", payment.id);
        Ok(())
    }

    /// Finalize a payment after cooling off expires
    pub fn finalize(ctx: Context<Finalize>) -> Result<()> {
        let payment = &mut ctx.accounts.payment;
        let clock = Clock::get()?;
        
        require!(payment.status == PaymentStatus::Pending, ClawbackError::PaymentNotPending);
        require!(clock.unix_timestamp >= payment.finalize_at, ClawbackError::CoolingOffNotExpired);
        
        payment.status = PaymentStatus::Finalized;
        
        let vault_key = ctx.accounts.vault.key();
        let payment_id_bytes = payment.id.to_le_bytes();
        let payment_bump = payment.bump;
        let payment_seeds: &[&[u8]] = &[b"payment", vault_key.as_ref(), &payment_id_bytes, &[payment_bump]];
        let signer = &[payment_seeds];
        
        let decimals = ctx.accounts.usdc_mint.decimals;
        
        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.payment_token_account.to_account_info(),
                    mint: ctx.accounts.usdc_mint.to_account_info(),
                    to: ctx.accounts.recipient_token_account.to_account_info(),
                    authority: payment.to_account_info(),
                },
                signer,
            ),
            payment.amount,
            decimals,
        )?;
        
        msg!("Payment {} finalized", payment.id);
        Ok(())
    }
}

// ============ Helpers ============

fn get_cooling_off_period(amount: u64) -> i64 {
    if amount < 10_000_000 {           // < $10
        15 * 60                         // 15 minutes
    } else if amount < 100_000_000 {   // $10 - $100
        60 * 60                         // 1 hour
    } else if amount < 500_000_000 {   // $100 - $500
        6 * 60 * 60                     // 6 hours
    } else {                            // > $500
        24 * 60 * 60                    // 24 hours
    }
}

// ============ Accounts ============

#[account]
pub struct Vault {
    pub owner: Pubkey,
    pub usdc_mint: Pubkey,
    pub vault_token_account: Pubkey,
    pub next_payment_id: u64,
    pub bump: u8,
}

#[account]
pub struct Policy {
    pub vault: Pubkey,
    pub agent: Pubkey,
    pub max_per_tx: u64,
    pub daily_limit: u64,
    pub daily_spent: u64,
    pub daily_window_start: i64,
    pub paused: bool,
    pub bump: u8,
}

#[account]
pub struct TrustedRecipient {
    pub vault: Pubkey,
    pub recipient: Pubkey,
    pub trusted: bool,
    pub bump: u8,
}

#[account]
pub struct Payment {
    pub id: u64,
    pub vault: Pubkey,
    pub agent: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub initiated_at: i64,
    pub finalize_at: i64,
    pub intent_hash: [u8; 32],
    pub intent_uri: String,
    pub status: PaymentStatus,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum PaymentStatus {
    Pending,
    Finalized,
    ClawedBack,
}

// ============ Contexts ============

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 32 + 32 + 8 + 1,
        seeds = [b"vault", owner.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,
    
    pub usdc_mint: InterfaceAccount<'info, Mint>,
    
    #[account(
        init,
        payer = owner,
        associated_token::mint = usdc_mint,
        associated_token::authority = vault,
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(agent: Pubkey)]
pub struct SetPolicy<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        seeds = [b"vault", owner.key().as_ref()],
        bump = vault.bump,
        has_one = owner,
    )]
    pub vault: Account<'info, Vault>,
    
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 1 + 1,
        seeds = [b"policy", vault.key().as_ref(), agent.as_ref()],
        bump
    )]
    pub policy: Account<'info, Policy>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ModifyPolicy<'info> {
    pub owner: Signer<'info>,
    
    #[account(
        seeds = [b"vault", owner.key().as_ref()],
        bump = vault.bump,
        has_one = owner,
    )]
    pub vault: Account<'info, Vault>,
    
    #[account(
        mut,
        seeds = [b"policy", vault.key().as_ref(), policy.agent.as_ref()],
        bump = policy.bump,
    )]
    pub policy: Account<'info, Policy>,
}

#[derive(Accounts)]
#[instruction(recipient: Pubkey)]
pub struct SetTrustedRecipient<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        seeds = [b"vault", owner.key().as_ref()],
        bump = vault.bump,
        has_one = owner,
    )]
    pub vault: Account<'info, Vault>,
    
    #[account(
        init_if_needed,
        payer = owner,
        space = 8 + 32 + 32 + 1 + 1,
        seeds = [b"trusted", vault.key().as_ref(), recipient.as_ref()],
        bump
    )]
    pub trusted_recipient: Account<'info, TrustedRecipient>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(amount: u64, intent_hash: [u8; 32], intent_uri: String)]
pub struct InitiatePayment<'info> {
    #[account(mut)]
    pub agent: Signer<'info>,
    
    /// CHECK: Recipient address
    pub recipient: UncheckedAccount<'info>,
    
    #[account(
        mut,
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,
    
    #[account(
        mut,
        seeds = [b"policy", vault.key().as_ref(), agent.key().as_ref()],
        bump = policy.bump,
    )]
    pub policy: Account<'info, Policy>,
    
    #[account(
        seeds = [b"trusted", vault.key().as_ref(), recipient.key().as_ref()],
        bump,
    )]
    pub trusted_recipient: Option<Account<'info, TrustedRecipient>>,
    
    #[account(
        init,
        payer = agent,
        space = 8 + 8 + 32 + 32 + 32 + 8 + 8 + 8 + 32 + 4 + 200 + 1 + 1,
        seeds = [b"payment", vault.key().as_ref(), &vault.next_payment_id.to_le_bytes()],
        bump
    )]
    pub payment: Account<'info, Payment>,
    
    #[account(mut)]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        init,
        payer = agent,
        associated_token::mint = usdc_mint,
        associated_token::authority = payment,
    )]
    pub payment_token_account: InterfaceAccount<'info, TokenAccount>,
    
    #[account(mut)]
    pub recipient_token_account: InterfaceAccount<'info, TokenAccount>,
    
    pub usdc_mint: InterfaceAccount<'info, Mint>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct Clawback<'info> {
    pub owner: Signer<'info>,
    
    #[account(
        seeds = [b"vault", owner.key().as_ref()],
        bump = vault.bump,
        has_one = owner,
    )]
    pub vault: Account<'info, Vault>,
    
    #[account(
        mut,
        seeds = [b"payment", vault.key().as_ref(), &payment.id.to_le_bytes()],
        bump = payment.bump,
        constraint = payment.vault == vault.key(),
    )]
    pub payment: Account<'info, Payment>,
    
    #[account(mut)]
    pub payment_token_account: InterfaceAccount<'info, TokenAccount>,
    
    #[account(mut)]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,
    
    pub usdc_mint: InterfaceAccount<'info, Mint>,
    
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct Finalize<'info> {
    /// CHECK: Anyone can finalize
    pub caller: UncheckedAccount<'info>,
    
    #[account(
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,
    
    #[account(
        mut,
        seeds = [b"payment", vault.key().as_ref(), &payment.id.to_le_bytes()],
        bump = payment.bump,
        constraint = payment.vault == vault.key(),
    )]
    pub payment: Account<'info, Payment>,
    
    #[account(mut)]
    pub payment_token_account: InterfaceAccount<'info, TokenAccount>,
    
    #[account(mut)]
    pub recipient_token_account: InterfaceAccount<'info, TokenAccount>,
    
    pub usdc_mint: InterfaceAccount<'info, Mint>,
    
    pub token_program: Interface<'info, TokenInterface>,
}

// ============ Errors ============

#[error_code]
pub enum ClawbackError {
    #[msg("Agent is paused")]
    AgentPaused,
    #[msg("Amount exceeds max per transaction")]
    ExceedsMaxPerTx,
    #[msg("Amount exceeds daily limit")]
    ExceedsDailyLimit,
    #[msg("Payment is not pending")]
    PaymentNotPending,
    #[msg("Cooling off period has not expired")]
    CoolingOffNotExpired,
}
