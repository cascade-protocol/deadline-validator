# RFC: Extensions to scheme_exact_svm.md - Deadline Validation, Smart Wallet Support, and Durable Nonces

**Status:** Draft<br>
**Author:** Misha Kolesnik (Cascade)<br>
**Created:** 2025-11-14<br>
**Target:** Amendments to `/specs/schemes/exact/scheme_exact_svm.md` (lines 95-100)<br>
**GitHub:** https://github.com/cascade-protocol/deadline-validator<br>
**Contact:** @tenequm (GitHub), @opwizardx (Twitter)

---

## Abstract

This RFC proposes amendments to the x402 SVM exact payment scheme specification to enable critical ecosystem features currently blocked by the rigid 3-4 instruction layout:

1. **Expand instruction count** - From 3-4 to 3-6 instructions to accommodate optional features
2. **Deadline validation instruction** - Enable `maxTimeoutSeconds` enforcement beyond Solana's ~80-90 second blockhash limit
3. **Durable nonce support** - Enable timeouts beyond 90 seconds for long-running operations
4. **Flexible transfer verification** - Support both static verification (direct TransferChecked) and simulation verification (CPI transfers)

These amendments enable smart wallets (Squads, SWIG, SPL Governance) and precise timeout control through opt-in features, while maintaining all existing security guarantees and preserving 100% backward compatibility with current implementations.

**Key architectural insight**: Smart wallets embed TransferChecked in account data or instruction payloads and execute via `invoke_signed()`. The transfer occurs during Cross-Program Invocation (CPI), not as a top-level instruction, requiring simulation-based verification instead of static analysis.

---

## Motivation

### Current Limitation

The x402 SVM exact scheme (as specified in `scheme_exact_svm.md` lines 95-100) requires:

```
The decompiled transaction MUST contain either 3 or 4 instructions:
1. Compute Budget: Set Compute Unit Limit
2. Compute Budget: Set Compute Unit Price
3. Optional: Associated Token Account Create
4. SPL Token or Token-2022 TransferChecked
```

This rigid structure blocks two critical use cases identified by the ecosystem.

### Use Case 1: Deadline Validation - Non-functional `maxTimeoutSeconds`

**Critical Issue: `maxTimeoutSeconds` is completely non-functional on Solana**

This is a **spec gap**, not an implementation bug. The x402 SVM spec defines `maxTimeoutSeconds` in PaymentRequirements but provides no mechanism to enforce it on Solana.

**Problem Report** (x402-solana Telegram, Nov 12, 2025):
> "maxTimeoutSeconds isn't honoured on Solana. Settlements consistently fail after ~1 or 2 minutes regardless of maxTimeoutSeconds being higher."

**Root Cause Analysis** (Carson, x402 core team):
> "The blockhash-based expiry has a hard ~80-90 second limit. Our remaining option would be to create a custom on-chain program that checks the Clock sysvar to enforce an actual expiry time."

**The Spec Gap:**
- EVM spec uses `validBefore` for timeout enforcement
- Solana spec never defined how to enforce `maxTimeoutSeconds`
- Result: Transactions ALWAYS expire after ~90 seconds when the blockhash expires
- Setting `maxTimeoutSeconds: 10` gives you ~90 seconds (not 10)
- Setting `maxTimeoutSeconds: 300` also gives you ~90 seconds (not 300)

**Impact:**
- The parameter exists in the spec but has zero effect
- Breaks user expectations and EVM parity
- Long-running AI operations (5+ minute LLM generations) cannot accept payment
- Clients specifying short timeouts (30s) get unexpectedly long windows (90s)
- Carson confirmed this affects ALL clients and facilitators

**Solution Requirements:**
- For `maxTimeoutSeconds ≤ 90`: Deadline validator alone provides precise enforcement
- For `maxTimeoutSeconds > 90`: BOTH deadline validator AND durable nonces required
  - Deadline validator enforces the actual timeout
  - Durable nonces bypass the 90-second blockhash expiry limit
  - Without both components, timeouts beyond 90s are impossible

### Use Case 2: Smart Wallet Support

**Problem Report** (Pontus, Corbits/ABK Labs, Nov 13, 2025):
> "Every major business in the world needs smart wallets to work. The reason [x-solana-settlement] exists is because transactions were failing for third-party wallets, smart wallets."

**Root Cause - Architectural Mismatch:**
Smart wallets execute token transfers via **Cross-Program Invocation (CPI)**, not as separate top-level instructions. The actual transaction structure:

```
1. Compute Budget: Set Compute Unit Limit
2. Compute Budget: Set Compute Unit Price
3. SmartWallet::Execute
   └─> [CPI] SPL Token::TransferChecked (embedded, executed via invoke_signed)
```

**TransferChecked is NOT a 4th instruction** - it's embedded within the Execute instruction and executed via CPI with the smart wallet PDA as signer. This is fundamentally different from EOA wallets where TransferChecked is a direct top-level instruction.

**Impact:**
- Squads multisig incompatible
- SPL Governance DAOs cannot make x402 payments
- **Ecosystem fragmentation:** Corbits team built separate `@faremeter/x-solana-settlement` scheme as a workaround, proving the business case for this RFC.

---

## Design Philosophy: Opt-in Complexity

### Critical Context: Facilitator Simplicity

The x402 protocol's success relies heavily on facilitator adoption. The current spec is intentionally lightweight:
- 3-4 instruction validation
- Simple static analysis
- No complex state management
- Minimal operational overhead

**This simplicity is a feature, not a limitation.** It enables:
- Easy facilitator onboarding
- Low operational costs
- High reliability
- Wide ecosystem adoption

### Why Opt-in, Not Required

Making advanced features (nonces, extended deadlines, complex verification) REQUIRED would:
- ❌ Increase barrier to entry for new facilitators
- ❌ Raise operational costs for all participants
- ❌ Reduce protocol adoption
- ❌ Force complexity on simple use cases

Instead, this RFC proposes **opt-in complexity**:
- ✅ Basic case remains simple (3-4 instructions, static verification)
- ✅ Power facilitators can opt into advanced features
- ✅ Complexity costs borne only by those who need it
- ✅ Progressive enhancement model

### Architectural Differences: EVM vs Solana

| Aspect | EVM | Solana |
|--------|-----|--------|
| Signature Model | Client signs **message** with `validBefore` | Client signs **transaction** with blockhash |
| Facilitator Role | Builds fresh transaction | Adds fee payer signature |
| Deadline Enforcement | Contract checks on-chain | Blockhash expires ~80-90s |
| Result | ✅ Any future timestamp | ❌ Hard limit at 90s |

This architectural difference is why deadline validation requires an additional on-chain program on Solana.

---

## Proposed Solution

### Overview - Unified Instruction Layout with Conditional Verification

Amend `scheme_exact_svm.md` to support 3-6 instructions (up from current 3-4) with conditional verification based on transaction structure:

**Core Changes:**
1. **Instruction count**: 3-6 instructions (vs current 3-4)
2. **Optional deadline validator**: Enables `maxTimeoutSeconds` enforcement via Clock sysvar
3. **Optional durable nonce**: Enables timeouts >90 seconds by bypassing blockhash expiry
4. **Flexible transfer verification**: Static verification OR simulation-based verification
5. **Smart wallet support**: Works with any program executing TransferChecked via CPI

**Verification Approach:**
- **If last instruction is TransferChecked**: Verify statically (current approach, unchanged)
- **If last instruction is NOT TransferChecked**: Verify via simulation (new, enables smart wallets)

This unified approach maintains all existing security guarantees while enabling both deadline enforcement and smart wallet compatibility through opt-in features.

### Transaction Structures

All transactions follow a unified instruction layout of 3-6 instructions. The last instruction determines verification method.

#### Unified Instruction Layout (3-6 instructions in order)
```
[Optional] 1st if present: System Program: Advance Nonce Account
[REQUIRED] Next two:     Compute Budget: Set Compute Unit Limit
                         Compute Budget: Set Compute Unit Price
[Optional] After budgets: Deadline Validator: Check Clock Sysvar
[Optional] 2nd-to-last:  Associated Token Account Create
[REQUIRED] Last:         Transfer Instruction
```

With all optional features present, the transaction contains 6 instructions:
1. Nonce Advance
2. Compute Budget Limit
3. Compute Budget Price
4. Deadline Validator
5. ATA Create
6. Transfer

#### Transfer Instruction (Last Position) - Two Possibilities

**Direct Transfer (EOA Wallets):**
```
SPL Token or Token-2022: TransferChecked
```
- TransferChecked is visible as top-level instruction
- Facilitator verifies statically
- Typical for standard wallets

**CPI Transfer (Smart Wallets):**
```
SmartWallet::Execute
  └─> TransferChecked (executed via invoke_signed() CPI)
```
- TransferChecked is NOT visible as top-level instruction
- Embedded in account data or instruction payload
- Executed at runtime via `invoke_signed()` with smart wallet PDA as signer
- Facilitator MUST verify via simulation

**How Smart Wallets Work with Facilitators:**

Smart wallets vary in architecture, but all support facilitators paying transaction fees:

**Multisig Wallets (Squads v4, Squads Smart Account):**
1. **Setup Phase** (happens before x402 payment):
   - Vault/transaction created on-chain
   - Members approve on-chain via voting
   - Transaction reaches "Ready" state

2. **x402 Payment Phase** (smart wallet CPI transfer):
   - Member builds execution transaction with facilitator as fee payer:
     ```typescript
     const tx = await vaultTransactionExecute({
       feePayer: facilitator.publicKey,  // Facilitator pays fees
       member: member.publicKey,         // Member with Execute permission
       ...
     })
     ```
   - Member signs their part (partial sign)
   - Sends partially-signed transaction to facilitator
   - Facilitator adds fee payer signature and submits

**Session/Permission Wallets (SWIG):**
- No on-chain approval phase
- SignV1/SignV2 instruction requires authority signature
- Can work with external fee payer via multi-signer transaction construction
- Authority signs; facilitator adds fee payer signature

**DAO Governance (SPL Governance):**
- Proposals approved on-chain by DAO voting
- ExecuteTransaction instruction requires NO specific signer
- Anyone (including facilitator) can execute approved transactions
- Facilitator can execute directly without client signature in execution phase

**Key insight:** All smart wallets support external fee payers, but the specific flow varies. The facilitator never gains custody - they only pay fees for operations that are either pre-approved (multisig/DAO) or explicitly authorized (session wallets).

---

## Technical Specification

### Amendment to scheme_exact_svm.md

Replace lines 95-100 of `scheme_exact_svm.md` with:

```markdown
### 1. Instruction Layout

The decompiled transaction MUST contain 3 to 6 instructions in the following order:

**[Optional] First instruction if present:** System Program: Advance Nonce Account
- If present, MUST be first instruction in the transaction
- Nonce authority MUST NOT equal fee payer (critical security check)
- Nonce account MUST be initialized and rent-exempt
- Current nonce value MUST match `extra.nonce.value` from PaymentRequirements

**[REQUIRED] Next two instructions:** Compute Budget Instructions
- Set Compute Unit Limit (program: ComputeBudget, discriminator: 2)
- Set Compute Unit Price (program: ComputeBudget, discriminator: 3)
- Compute unit price MUST be ≤ 5 lamports per compute unit
- These always follow nonce (if present) or appear first (if no nonce)

**[Optional]:** Deadline Validator
- If present, MUST verify Clock.unix_timestamp ≤ deadline
- Program MUST be specified in `extra.validatorProgram`
- Deadline value MUST be specified in `extra.deadline`
- Appears after compute budget instructions

**Validator Program Requirements:**
- MUST check Clock sysvar and fail transaction if deadline exceeded
- MUST have no upgrade authority (immutable deployment)
- Facilitators SHOULD verify program matches expected behavior before accepting

**[Optional]:** Associated Token Account Create
- Only when destination ATA does not exist
- MUST create ATA for (payTo, asset) under selected token program
- If present, MUST be second-to-last instruction (immediately before transfer)
- Facilitators MAY pay ATA creation rent (~0.002 SOL) or reject transactions requiring new ATAs

**[REQUIRED] Last instruction:** Transfer Instruction
MUST be one of:
a) SPL Token or Token-2022: TransferChecked (direct transfer), OR
b) Any program that executes TransferChecked via Cross-Program Invocation (CPI)

**Validation Requirements:**

Facilitators MUST verify that EVERY instruction matches one of the allowed types above:
- System Program: Advance Nonce (discriminator 4)
- ComputeBudget: Set Compute Unit Limit (discriminator 2)
- ComputeBudget: Set Compute Unit Price (discriminator 3)
- Deadline Validator: As specified in `extra.validatorProgram`
- Associated Token Program: Create ATA (discriminator 0 or 1)
- Token Program: TransferChecked (discriminator 12) OR Smart Wallet execution

Any instruction not matching these exact types MUST cause validation failure. No additional instructions are permitted.

**Fee Payer Isolation:**

The transaction fee payer MUST NOT appear in instruction accounts except as the funding account for ATA creation. This prevents facilitator account drainage through malicious instruction construction.

### 2. Transfer Verification

Facilitators MUST verify the transfer using one of two methods, determined by transaction structure:

**Static Verification** (when last instruction is TransferChecked):
- Program MUST be spl-token or token-2022
- Amount MUST equal maxAmountRequired exactly
- Destination MUST be correct ATA for (payTo, asset)
- Source ATA MUST exist and have sufficient balance
- Source ATA owner MUST NOT be the fee payer

**Simulation Verification** (when last instruction is NOT TransferChecked):
Facilitator MUST simulate the transaction and verify:
- Exactly ONE TransferChecked CPI occurred during execution
- CPI amount equals maxAmountRequired exactly
- CPI destination equals correct ATA for (payTo, asset)
- CPI mint equals specified asset
- CPI program is spl-token or token-2022

Simulation-based verification enables smart wallets (Squads, SWIG, SPL Governance) that execute
transfers via invoke_signed() with TransferChecked embedded in account data or instruction payloads.

NOTE: Simulation-based verification has inherent limitations (TOCTOU, RPC trust) that facilitators should understand. Implementation guidance will be provided separately.
```

### Implementation Guidance: Verification Method Selection

Facilitators determine verification method by inspecting the last instruction:

```rust
let last_instruction = tx.message.instructions.last();

if last_instruction.program_id == TOKEN_PROGRAM_ID
   || last_instruction.program_id == TOKEN_2022_PROGRAM_ID {
    // Last instruction is TransferChecked
    verify_transfer_statically(tx, payment_requirements)?;
} else {
    // Last instruction is something else (e.g., smart wallet execution)
    verify_transfer_via_simulation(tx, payment_requirements)?;
}
```

This approach is agnostic to wallet architecture—it works with ANY program executing a valid CPI transfer, enabling permissionless innovation.

**Why Smart Wallets Require Different Verification:**

Smart wallets and EOA wallets are architecturally different:
- **EOA wallets:** Client directly signs TransferChecked as a top-level instruction
- **Smart wallets:** Client signs execution instruction; smart wallet PDA signs TransferChecked via CPI

Smart wallet transactions cannot be verified with static analysis because:
1. TransferChecked is embedded in account data or instruction payload (not visible as top-level instruction)
2. Smart wallet PDA is the actual signer via `invoke_signed()`
3. Transfer occurs during CPI execution, not as a separate instruction

### Feature 1: Deadline Validation (Opt-in)

**Purpose:** Enforce `maxTimeoutSeconds` beyond Solana's ~80-90 second blockhash limit

**PaymentRequirements Extension:**
```json
{
  "scheme": "exact",
  "network": "solana",
  "extra": {
    "feePayer": "...",
    "validatorProgram": "DEADaT1auZ8JjUMWUhhPWjQqFk9HSgHBkt5KaGMVnp1H",
    "deadline": 1731432060
  }
}
```

**When enabled:**
- Instruction checks Clock sysvar (position varies based on nonce presence)
- Validator program verifies: `Clock.unix_timestamp <= deadline`
- Transaction fails atomically if deadline exceeded

**Timeout Enforcement:**
- For `maxTimeoutSeconds ≤ 90`: Deadline validator alone provides precise enforcement
  - Blockhash expires at ~90s, but deadline validator enforces earlier limit
  - Example: 30-second timeout enforced by validator, blockhash irrelevant
- For `maxTimeoutSeconds > 90`: MUST be combined with durable nonces
  - Without nonces: blockhash expires at ~90s regardless of deadline
  - With nonces: deadline validator enforces the actual timeout
  - Nonces bypass blockhash expiry, deadline validator provides enforcement

**Design:** Intentionally ultra-simple, deployed as immutable infrastructure (no upgrade authority)

**Reference Implementation:**

The canonical deadline validator implementation is provided as a community reference:

- **Program ID:** `DEADaT1auZ8JjUMWUhhPWjQqFk9HSgHBkt5KaGMVnp1H`
- **Repository:** https://github.com/cascade-protocol/deadline-validator
- **Author:** Misha Kolesnik (Cascade)
- **Status:** Deployed on mainnet, upgrade authority will be revoked after final verification
- **Verification:** Source code available for audit

Facilitators MAY use this reference implementation or deploy/verify their own deadline validator programs that meet the technical requirements specified above.

### Feature 2: Durable Nonce Support (Opt-in)

**Purpose:** Enable timeouts beyond Solana's ~80-90 second blockhash limit

**When Required:**
- REQUIRED for `maxTimeoutSeconds > 90`
- OPTIONAL for `maxTimeoutSeconds ≤ 90` (provides no benefit)

**PaymentRequirements Extension:**
```json
{
  "scheme": "exact",
  "network": "solana",
  "maxTimeoutSeconds": 300,
  "extra": {
    "feePayer": "...",
    "validatorProgram": "DEADaT1auZ8JjUMWUhhPWjQqFk9HSgHBkt5KaGMVnp1H",
    "deadline": 1731432360,
    "nonce": {
      "account": "NonceAcct1111111111111111111111111111111111",
      "authority": "ClientAuth111111111111111111111111111111111",
      "value": "4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi"
    }
  }
}
```

**Client Responsibilities:**
- Create and initialize nonce account (rent: ~0.00144 SOL)
- Client MUST be nonce authority
- Provide nonce details in PaymentRequirements
- Include NonceAdvance as first instruction
- Sign NonceAdvance with nonce authority key

**Facilitator Verification (MUST):**
1. **Nonce Authority Safety:**
   - `nonce_authority MUST NOT equal fee_payer`
   - Critical security check to prevent facilitator account drainage

2. **Nonce Account Validation:**
   - Nonce account owner MUST be System Program (11111111111111111111111111111111)
   - Nonce account MUST exist and be initialized
   - Nonce account MUST be rent-exempt
   - Current nonce value MUST match `extra.nonce.value`
   - Nonce authority MUST match `extra.nonce.authority`

3. **Instruction Position:**
   - If present, NonceAdvance MUST be first instruction (position 0)
   - Program ID MUST be System Program

**Transaction Structure:**
When nonce is provided, NonceAdvance replaces `recentBlockhash`:
- Transaction uses nonce value instead of recent blockhash
- No expiry from blockhash (transaction valid until deadline)
- Nonce advances after successful execution

**Security Model:**
- Client controls nonce (is authority)
- Facilitator only pays fees (never nonce authority)
- Nonce account can be client-created OR service-provisioned
  - If service-provisioned: client MUST still be authority
  - Service retains account ownership, client has usage rights

**Design Philosophy:**
Nonce support follows the same opt-in complexity model as other features:
- Simple use cases (≤90s) unaffected
- Complex use cases (>90s) bear additional complexity
- Client-managed (industry standard pattern)

### Feature 3: Smart Wallet CPI Transfer Support

**Purpose:** Enable smart wallets that execute transfers via Cross-Program Invocation

**How Smart Wallets Execute Transfers:**

All smart wallet protocols follow the same architecture:

1. **Storage**: Embedded instructions stored in account data or instruction payload
2. **Execution**: Smart wallet program calls `invoke_signed()` to execute TransferChecked via CPI

**Critical Code Example - How CPI Works:**

```rust
// Squads v4: vault_transaction_execute.rs
pub fn vault_transaction_execute(ctx: Context<VaultTransactionExecute>) -> Result<()> {
    // Load transaction from account data
    let transaction = ctx.accounts.transaction.take();
    let transaction_message = transaction.message; // Contains embedded TransferChecked

    let vault_seeds = &[SEED_PREFIX, multisig_key.as_ref(), SEED_VAULT,
                        &vault_index.to_le_bytes(), &[vault_bump]];

    // Execute embedded instructions via CPI
    for (ix, account_infos) in embedded_instructions.iter() {
        // THIS is where TransferChecked actually executes
        invoke_signed(&ix, &account_infos, &[vault_seeds])?;
    }
    Ok(())
}
```

**Transaction Structure:**
```
Top-level (visible):
1. ComputeBudget
2. ComputeBudget
3. SmartWallet::Execute

Embedded (NOT visible):
└─> TransferChecked (executed via invoke_signed)
```

**Verification Method Selection:**
- Last instruction is NOT TransferChecked → Use simulation verification
- Last instruction is TransferChecked → Use static verification

**Why Agnostic Verification:**
- No program ID whitelisting required
- Works with ANY program executing valid CPI transfer
- Enables permissionless innovation
- Future-proof design

---

## Security Analysis

### Security Properties Preserved

1. **Fee Payer Safety**: Never appears in instruction accounts (except as ATA funding account)
2. **Compute Budget Validity**: First two instructions validated
3. **Transfer Correctness**: Amount, destination, mint verified
4. **Amount Exactness**: Exact match required

### Additional Security Layers

5. **Deadline Validation** (when enabled): Clock sysvar consensus-managed
6. **Simulation Verification** (for CPI transfers): Catches malicious behavior before settlement
7. **Nonce Authority Safety** (when enabled): fee_payer never equals nonce_authority

### Defense in Depth

```
Layer 1: Static Analysis (existing)
  ↓ Verify instruction count, programs, accounts
Layer 2: Fee Payer Safety (existing)
  ↓ Fee payer not in any instruction (except ATA funding)
Layer 3: Verification Method Selection (NEW)
  ↓ Determine: Static or Simulation based on last instruction
Layer 4: Transfer Verification
  ↓ Static (direct TransferChecked) OR Simulation (CPI transfer)
Result: Safe to sponsor transaction
```

### Nonce-Specific Security Checks

**Critical: Nonce Authority Verification**
```typescript
// MUST verify nonce authority is NOT fee payer
if (nonceAuthority === feePayer) {
  throw new Error('fee_payer_cannot_be_nonce_authority');
}
```

**Why Critical:**
If facilitator is nonce authority, malicious client could submit transaction that:
1. Advances facilitator's nonce (consuming it)
2. Executes repeatedly with different nonces
3. Drains facilitator's nonce accounts


---

## Backward Compatibility

### 100% Backward Compatible

| Transaction Type | Current Spec | Proposed Spec |
|-----------------|--------------|---------------|
| EOA wallet payment (3 ix) | ✅ Supported | ✅ Supported |
| EOA + ATA creation (4 ix) | ✅ Supported | ✅ Supported |
| EOA + deadline (4 ix) | ❌ Rejected | ✅ Supported |
| EOA + nonce + deadline (5 ix) | ❌ Rejected | ✅ Supported |
| EOA + nonce + deadline + ATA (6 ix) | ❌ Rejected | ✅ Supported |
| Smart wallet (3 ix) | ❌ Rejected | ✅ Supported |
| Smart wallet + deadline (4 ix) | ❌ Rejected | ✅ Supported |
| Smart wallet + nonce + deadline (5 ix) | ❌ Rejected | ✅ Supported |

**No breaking changes** - existing implementations continue working unchanged.

### Facilitator Adoption Path

**Basic Facilitators (Status Quo)**
- Continue with 3-4 instruction support
- No changes required
- Zero additional complexity

**Power Facilitators (Opt-in)**
- Implement simulation verification for CPI transfers
- Support smart wallets (Squads, SWIG, SPL Governance)
- Optional deadline validation support
- Optional durable nonce support
- Bear additional costs for enhanced features

---

## Alternatives Considered

1. **Whitelist Smart Wallet Programs** - Rejected: Creates gatekeeping, blocks innovation
2. **Arbitrary Instruction Count** - Rejected: Opens attack surface, unbounded verification cost
3. **Require Nonces for All** - Rejected: Forces unnecessary complexity on ≤90s use cases, capital lockup
4. **Facilitator-Managed Nonces** - Rejected for initial spec: Industry pattern is client-managed; service-provisioned nonces can work within client-authority model

**Chosen approach**: Agnostic verification with opt-in complexity preserves simplicity while enabling advanced features.


---

## References

- x402 Exact Scheme Spec (SVM)
- x402 Repository
- Problem Reports: x402-solana Telegram (Nov 12), Corbits conversation (Nov 13)
- Alternative Implementation: @faremeter/x-solana-settlement (demonstrates business need)

---

## Acknowledgments

- **Misha Kolesnik** (CascadePay) - RFC author, deadline validator implementation
- **Carson** (Coinbase x402 team) - Identified deadline problem
- **Pontus & Corbits team** - Identified smart wallet incompatibility, built alternative implementation
- **x402-solana community** - Feedback and validation

---

## Smart Wallet Architecture Evidence

This section provides direct code evidence showing that smart wallets execute TransferChecked via CPI, not as a top-level instruction. This demonstrates why smart wallet transactions require simulation-based verification instead of static analysis.

### Key Code References

**Squads v4 - How CPI Execution Works:**
```rust
// vault_transaction_execute.rs
pub fn vault_transaction_execute(ctx: Context<VaultTransactionExecute>) -> Result<()> {
    let transaction = ctx.accounts.transaction.take();
    let transaction_message = transaction.message; // Contains embedded TransferChecked

    // Execute embedded instructions via CPI
    for (ix, account_infos) in embedded_instructions.iter() {
        invoke_signed(&ix, &account_infos, &[vault_seeds])?; // TransferChecked executes here
    }
}
```

**SWIG - Embedded in Payload:**
```rust
// sign_v1.rs
pub struct SignV1 {
    instruction_payload: Vec<u8>, // TransferChecked serialized here
}

// Execution
instruction.execute()?; // Calls invoke_signed() internally
```

**SPL Governance - Account Data Storage:**
```rust
// ProposalTransaction account stores instructions
// process_execute_transaction.rs line 811:
invoke_signed(&instruction, &account_infos, &[seeds])?;
```

### Summary

All smart wallet protocols:
1. Store TransferChecked as serialized bytes (not visible as top-level instruction)
2. Execute via `invoke_signed()` with PDA as signer
3. Cannot be verified through static analysis
4. Require simulation to detect the CPI transfer

**Important**: Smart wallet execution supports external fee payers, enabling x402 compatibility via simulation-based verification:
- **Multisig wallets** (Squads): Member signs execution; facilitator adds fee payer signature
- **Session wallets** (SWIG): Authority signs; facilitator adds fee payer signature
- **DAO governance** (SPL): Facilitator can execute directly (no client signature needed)

All smart wallet implementations maintain security: facilitator only pays fees, never gains custody. See "How Smart Wallets Work with Facilitators" section for detailed flows.

---

**End of RFC**
