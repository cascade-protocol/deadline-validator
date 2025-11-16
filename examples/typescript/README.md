# TypeScript Example

Simple TypeScript client for Deadline Validator. No SDK installation needed - just copy the function!

## Quick Start

```bash
npm install @solana/web3.js
```

## Usage

Copy the `createValidateDeadlineInstruction` function from `index.ts` into your project:

```typescript
import { createValidateDeadlineInstruction } from './path/to/index';

// Create instruction with deadline 5 minutes from now
const deadline = Math.floor(Date.now() / 1000) + 300;
const instruction = createValidateDeadlineInstruction(deadline);

// Add to your transaction
transaction.add(instruction);
```

## Examples

### Payment with 24-hour deadline

```typescript
const paymentDeadline = Math.floor(Date.now() / 1000) + 86400;  // 24 hours
const instruction = createValidateDeadlineInstruction(paymentDeadline);
transaction.add(instruction);
```

### Never expires

```typescript
const instruction = createValidateDeadlineInstruction(0);  // 0 = never expires
```

### Specific timestamp

```typescript
const deadline = 1735689600;  // Jan 1, 2025 00:00:00 UTC
const instruction = createValidateDeadlineInstruction(deadline);
```

## Error Handling

```typescript
try {
  await sendAndConfirmTransaction(connection, transaction, [payer]);
} catch (error) {
  if (error.message.includes('custom program error: 0x0')) {
    console.log('Deadline has expired!');
  }
}
```

## That's It!

No SDK to install, no code generation, no build steps. Just copy 20 lines and go.
