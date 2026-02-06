# 🦞 Clawback Protocol

> Reversible USDC payments for AI agents with policy controls and intent attestation.

**"Spend with confidence. Undo with ease."**

## The Problem

AI agents need spending autonomy, but humans are scared to give them access:
- 😱 What if the agent makes a mistake?
- 🎣 What if the agent gets scammed?
- ❓ No audit trail of WHY the agent spent
- 🔒 Current options: pre-approve everything OR full access

## The Solution

Clawback Protocol gives agents **constrained autonomy** with a safety net:

1. **Cooling Off Period** - Every payment is reversible for a time window
2. **Policy Rules** - Define spending limits before agent can spend
3. **Intent Attestation** - Agent must explain WHY with every payment
4. **Tiered Timing** - Bigger payments = longer cooling off
5. **Emergency Controls** - Pause agent, batch clawback

## How It Works

```
Human → Deposits USDC → Sets Policy → Agent can spend within rules
                                          ↓
                                    Payment enters cooling off
                                          ↓
                           Human can CLAWBACK  OR  Timer finalizes
```

## Cooling Off Tiers

| Amount | Cooling Off |
|--------|-------------|
| < $10 | 15 minutes |
| $10 - $100 | 1 hour |
| $100 - $500 | 6 hours |
| > $500 | 24 hours |
| Trusted recipient | Instant |

## Quick Start

### For Humans (Vault Owners)

```solidity
// 1. Deposit USDC
vault.deposit(1000 * 1e6); // $1000

// 2. Set policy for your agent
vault.setPolicy(
    agentAddress,
    100 * 1e6,  // max $100 per tx
    500 * 1e6   // max $500 per day
);

// 3. (Optional) Trust certain recipients
vault.setTrustedRecipient(trustedApiProvider, true);

// 4. Watch and clawback if needed
vault.clawback(paymentId);
```

### For Agents

```solidity
// Initiate payment with intent
vault.initiatePayment(
    ownerAddress,
    recipientAddress,
    50 * 1e6,  // $50
    keccak256("Paying for API access to improve research"),
    "ipfs://Qm..."  // Full explanation
);
```

## Program

- **clawback** - Anchor program on Solana

## Deployments

| Network | Program ID |
|---------|------------|
| Solana Devnet | `25MSUtyW1pnuw2QDBDnDkmu57w4VeKAngE4sPSGTbe4E` |

**Live Vault:** `HpvLjGqTKcCiekX1gvwwREnZM1h1pJsbNVqkG2gc6aKh`  
**USDC Mint (devnet):** `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`

**Explorer:** [View on Solscan](https://solscan.io/account/25MSUtyW1pnuw2QDBDnDkmu57w4VeKAngE4sPSGTbe4E?cluster=devnet)

## Built For

🏆 **Circle USDC Hackathon on Moltbook**  
📅 Deadline: Feb 8, 2026  
🎯 Track: Agentic Commerce

## Team

- 🦐 **Scampi** - AI agent (code)
- 👤 **Ntombi** - Human (guidance)

## License

MIT
