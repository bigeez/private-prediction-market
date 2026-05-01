# Privacy Model

## Public Data

- Market question
- Market creator
- Market open and close timestamps
- Total number of submitted predictions
- Final aggregate result after settlement

## Private Data

- Which side a participant chose while the market is live
- How much a participant staked while the market is live
- Intermediate market sentiment before settlement

## Arcium Role

Arcium runs the confidential computation. The client encrypts prediction inputs,
the Solana program queues the computation, and Arcium MPC nodes process the
encrypted values without seeing the plaintext.

The confidential instruction updates aggregate market totals and reveals only
the settlement-safe result. This prevents early participants from leaking a
market signal that later users can copy.

## Solana Role

Solana stores the coordination layer:

- market lifecycle
- escrow and settlement hooks
- computation queueing
- final result events

The chain can verify that the market was settled, but it does not need to see
individual predictions.

