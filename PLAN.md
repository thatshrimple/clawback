# 🦞 Clawback Protocol - Project Plan

**Start:** Feb 4, 2026  
**Deadline:** Feb 8, 2026 12:00 PM PST  
**Days remaining:** 4

---

## 📅 Timeline

### Day 1 (Feb 4-5) - Foundation
- [x] Research and concept development
- [x] Create project spec (CLAWBACKPROTOCOL.md)
- [x] Create project plan (this file)
- [x] Set up project structure
- [x] Write smart contract v1 (core functionality)
- [ ] Deploy to Base Sepolia testnet
- [ ] Test basic deposit/withdraw/pay/clawback

### Day 2 (Feb 5-6) - Smart Contract Complete
- [ ] Add policy engine to contract
- [ ] Add tiered cooling off logic
- [ ] Add intent attestation
- [ ] Add emergency controls (pause, batch clawback)
- [ ] Write comprehensive tests
- [ ] Audit for vulnerabilities
- [ ] Redeploy final contract

### Day 3 (Feb 6-7) - OpenClaw Skill
- [ ] Initialize skill package
- [ ] Implement `clawback deposit` command
- [ ] Implement `clawback pay` command
- [ ] Implement `clawback status` command
- [ ] Implement `clawback undo` command
- [ ] Implement `clawback policy` command
- [ ] Implement `clawback pause/finalize` commands
- [ ] Test skill end-to-end
- [ ] Write SKILL.md documentation

### Day 4 (Feb 7-8) - Polish & Submit
- [ ] Record demo video
- [ ] Write submission post for m/usdc
- [ ] Create README with examples
- [ ] Final testing
- [ ] Submit to Moltbook (before 12:00 PM PST!)
- [ ] Share on X/Twitter
- [ ] Celebrate 🦞🎉

---

## ✅ Task Checklist

### Anchor Program Tasks

```
[x] Create Anchor project structure
[x] Implement clawback program (lib.rs)
    [x] Account structs (Vault, Policy, Payment, TrustedRecipient)
    [x] initialize_vault instruction
    [x] set_policy instruction
    [x] pause_agent / unpause_agent instructions
    [x] set_trusted_recipient instruction
    [x] initiate_payment instruction
    [x] clawback instruction
    [x] finalize instruction
    [x] get_cooling_off_period helper
    [x] Error handling
[ ] Build program (anchor build)
[ ] Deploy to Solana devnet
[ ] Write TypeScript tests
    [ ] Test deposit/withdraw
    [ ] Test policy enforcement
    [ ] Test payment initiation
    [ ] Test clawback within window
    [ ] Test finalize after window
    [ ] Test trusted recipients (instant)
    [ ] Test blocked recipients (revert)
    [ ] Test daily limit enforcement
    [ ] Test emergency pause
[ ] Deploy to Base Sepolia
[ ] Verify contract on BaseScan
```

### Skill Tasks

```
[x] npm init clawback-skill
[x] Set up TypeScript
[x] Install dependencies (@coral-xyz/anchor, commander)
[x] Create src/types.ts (types + helpers)
[x] Create src/client.ts (Anchor program interactions)
[x] Implement commands in src/cli.ts:
    [x] init-vault
    [x] set-policy
    [x] pay
    [x] status
    [x] undo (clawback)
    [x] finalize
    [x] pause / unpause
    [x] trust
[x] Create SKILL.md
[ ] Test with OpenClaw (needs deployment)
[ ] Publish to ClawHub (optional)
```

### Submission Tasks

```
[ ] Write compelling post title
[ ] Write post content explaining:
    [ ] Problem being solved
    [ ] How Clawback works
    [ ] Demo/video link
    [ ] Contract address
    [ ] How to try it
[ ] Record 2-3 min demo video
[ ] Post to m/usdc submolt
[ ] Cross-post to X (@BotScampi)
```

---

## 🔧 Technical Decisions

| Decision | Choice | Reason |
|----------|--------|--------|
| Chain | **Solana Devnet** | Ntombi prefers Solana, aligns with Colosseum |
| Framework | **Anchor** | Modern Solana development |
| Skill Language | TypeScript | OpenClaw native |
| Storage | On-chain only | Simplicity, auditability |
| Intent Storage | IPFS via URI | Off-chain but verifiable |

---

## 📦 Dependencies

### Anchor Program
- anchor-lang 0.30.1
- anchor-spl 0.30.1
- Solana CLI tools

### Skill
- @solana/web3.js
- @coral-xyz/anchor
- commander (CLI)
- dotenv

---

## 🚨 Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Contract bug | High | Thorough testing, keep it simple |
| Time crunch | High | Focus on MVP, cut features if needed |
| Base testnet issues | Medium | Have backup plan (Sepolia ETH) |
| Skill complexity | Medium | Start with minimal commands |

---

## 📍 Current Status

**Phase:** Day 1-2 - Skill Development  
**Last Updated:** 2026-02-04 15:25 PST  
**Next Task:** Deploy Anchor program (blocked on devnet SOL), then test skill

**Completed Today:**
- ✅ Project spec and plan
- ✅ **Anchor program v1 built** (`target/deploy/clawback.so`)
- ✅ IDL generated (`target/idl/clawback.json`)
- ✅ **OpenClaw Skill created** (`skills/clawback/`)
  - ✅ SKILL.md with full documentation
  - ✅ TypeScript client (`src/client.ts`)
  - ✅ CLI with all commands (`src/cli.ts`)
  - ✅ Types and helpers (`src/types.ts`)

---

## 💬 Notes

- Keep scope tight - MVP first, polish later
- Document as we go
- Test early and often
- Submission > perfection

---

*Scampi 🦞 building for Ntombi*
