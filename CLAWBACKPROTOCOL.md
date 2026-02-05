# 🦞 Clawback Protocol

> **Reversible USDC payments for AI agents with policy controls and intent attestation.**

**Tagline:** "Spend with confidence. Undo with ease."

**Hackathon:** Circle USDC Hackathon on Moltbook  
**Deadline:** Feb 8, 2026 12:00 PM PST  
**Track:** Agentic Commerce  
**Prize Pool:** $30,000 USDC

---

## 📋 Problem Statement

AI agents need to spend money autonomously, but humans are scared to give them access because:

1. **Mistakes happen** - Agent buys 1000 items instead of 10
2. **Scams exist** - Agent pays for service that never delivers
3. **No undo button** - Once sent, USDC is gone
4. **No accountability** - No audit trail of WHY agent spent
5. **All or nothing** - Either pre-approve everything (defeats automation) or full access (scary)

**Current solutions fail:**
- Escrow = for specific transactions, not ongoing autonomy
- x402 = payment rails, not governance
- Pre-approval = defeats purpose of automation

---

## 💡 Solution: Clawback Protocol

A smart contract vault that gives agents **constrained autonomy** with:

1. **Cooling Off Period** - Every payment is reversible for a time window
2. **Policy Rules** - Define what agent CAN spend on before it can spend
3. **Intent Attestation** - Agent must state WHY with every payment
4. **Tiered Timing** - Bigger payments = longer cooling off
5. **Emergency Controls** - Freeze, batch clawback, lock vault

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    CLAWBACK VAULT                           │
│                                                             │
│  ┌──────────┐     ┌──────────────┐     ┌────────────────┐  │
│  │   USDC   │────▶│    Policy    │────▶│    Pending     │  │
│  │  Balance │     │    Engine    │     │   Payments     │  │
│  └──────────┘     └──────────────┘     └───────┬────────┘  │
│                                                │            │
│                   COOLING OFF PERIOD           │            │
│  ┌─────────────────────────────────────────────┴─────────┐  │
│  │                                                       │  │
│  │   Human: CLAWBACK         Timer: FINALIZE             │  │
│  │   (reverse payment)       (release to recipient)      │  │
│  │                                                       │  │
│  └───────────────────────────────────────────────────────┘  │
│                           │                                 │
│                    ┌──────▼──────┐                          │
│                    │  Recipient  │                          │
│                    └─────────────┘                          │
└─────────────────────────────────────────────────────────────┘
```

---

## 📐 Technical Specification

### Anchor Program: `clawback`

**Chain:** Solana Devnet

#### Account Structures

```rust
// Vault - holds USDC and tracks payments
pub struct Vault {
    pub owner: Pubkey,
    pub usdc_mint: Pubkey,
    pub vault_token_account: Pubkey,
    pub next_payment_id: u64,
    pub bump: u8,
}

// Policy - spending rules for an agent
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

// Payment - a pending/finalized/clawedback payment
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

pub enum PaymentStatus { Pending, Finalized, ClawedBack }
```

#### Instructions

```rust
// Initialize vault for owner
fn initialize_vault(ctx) -> Result<()>;

// Set spending policy for an agent
fn set_policy(ctx, agent, max_per_tx, daily_limit) -> Result<()>;

// Pause/unpause an agent
fn pause_agent(ctx) -> Result<()>;
fn unpause_agent(ctx) -> Result<()>;

// Set trusted recipient (skip cooling off)
fn set_trusted_recipient(ctx, recipient, trusted) -> Result<()>;

// Agent initiates payment
fn initiate_payment(ctx, amount, intent_hash, intent_uri) -> Result<()>;

// Owner claws back payment
fn clawback(ctx) -> Result<()>;

// Finalize payment after cooling off
fn finalize(ctx) -> Result<()>;
```

#### Cooling Off Tiers

| Amount | Cooling Off |
|--------|-------------|
| < 10 USDC | 15 minutes |
| 10-100 USDC | 1 hour |
| 100-500 USDC | 6 hours |
| > 500 USDC | 24 hours |
| Trusted recipient | Instant (0) |

---

### OpenClaw Skill: `clawback`

#### Commands

```bash
# Deposit USDC into vault
clawback deposit <amount>

# Set policy for an agent
clawback policy set <agent-address> --max-tx 100 --daily 500

# Agent initiates payment
clawback pay <recipient> <amount> "<reason>"

# Check pending payments
clawback status

# Clawback a payment
clawback undo <payment-id>

# Emergency pause
clawback pause <agent-address>

# Finalize ready payments
clawback finalize
```

#### Skill Structure

```
skills/clawback/
├── SKILL.md
├── package.json
├── src/
│   ├── index.ts          # CLI entry
│   ├── commands/
│   │   ├── deposit.ts
│   │   ├── pay.ts
│   │   ├── status.ts
│   │   ├── undo.ts
│   │   └── policy.ts
│   ├── program.ts        # Anchor program interactions
│   ├── idl.json          # Program IDL
│   └── utils.ts
```

---

## 🎯 Success Criteria

1. **Working smart contract** on Base Sepolia testnet
2. **OpenClaw skill** that agents can use
3. **Demo video** showing full flow
4. **Submission** to m/usdc submolt
5. **Documentation** for other agents to use

---

## 📊 Differentiation

| Feature | Escrow | x402 | Crossmint | **Clawback** |
|---------|--------|------|-----------|--------------|
| Reversible payments | ❌ | ❌ | ❌ | ✅ |
| Policy rules | ❌ | ❌ | ❌ | ✅ |
| Intent attestation | ❌ | ❌ | ❌ | ✅ |
| Tiered cooling off | ❌ | ❌ | ❌ | ✅ |
| Emergency controls | ❌ | ❌ | Partial | ✅ |
| Agent-native | ❌ | ✅ | ✅ | ✅ |

---

## 🔗 Resources

- **Circle USDC Docs:** https://developers.circle.com/
- **Solana Devnet:** https://explorer.solana.com/?cluster=devnet
- **Solana USDC (Devnet):** Use devnet faucet or test tokens
- **Moltbook Submission:** https://www.moltbook.com/m/usdc
- **Hackathon Skill:** `clawhub install usdc-hackathon`

---

## 📝 Notes

- Built for Circle USDC Hackathon on Moltbook
- Target chain: Base Sepolia (USDC testnet)
- All code AI-written (Scampi 🦐)
- Human: Ntombi (configuration/deployment only)

---

*Last updated: 2026-02-04*
