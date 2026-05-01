# Private Prediction Market

Private Prediction Market is a Solana + Arcium starter project for encrypted
prediction markets. Users can submit predictions and stake amounts without
revealing their side or conviction while the market is live. Arcium computes
the aggregate result over encrypted inputs, and Solana records the public market
state, settlement lifecycle, and final revealed outcome.

## Why This Needs Arcium

Prediction markets work best when participants express honest beliefs. Public
votes and stake sizes can create herding, copy-trading, intimidation, and
manipulation before a market closes.

This project keeps the sensitive pieces private:

- The user's prediction side is encrypted before submission.
- The user's stake amount is encrypted before submission.
- Arcium MPC nodes compute over encrypted submissions.
- Only the settlement result is revealed on-chain.
- Solana remains the public coordination and settlement layer.

## Product Flow

1. A creator opens a market with a question and close timestamp.
2. Participants encrypt their prediction and stake client-side.
3. The Solana program queues an Arcium computation for each private prediction.
4. The market closes after the deadline.
5. Arcium reveals the winning side at settlement.
6. The Solana program emits the settlement result.

## Repository Layout

```text
.
├── Anchor.toml
├── Arcium.toml
├── encrypted-ixs/              # Arcis confidential computation code
├── programs/private_market/    # Solana/Anchor + Arcium CPI program
├── src/market.mjs              # Local product simulation
├── tests/market.test.mjs       # Runnable local tests
└── docs/privacy-model.md       # Plain-English privacy explanation
```

## Run The Local Simulation

The local simulation does not require Solana, Anchor, or Arcium. It is included
so reviewers can exercise the market lifecycle immediately.

```bash
npm test
```

## Open The Demo App

Open `app/index.html` in a browser, or deploy the repo to Vercel. The static
demo shows the user flow:

- submit encrypted predictions
- keep live market signal hidden
- close the market
- reveal settlement

## Run With Arcium Tooling

Install the Arcium toolchain on Linux or WSL2, then run:

```bash
arcium build
arcium test
```

The project follows the current Arcium project shape:

- `Arcium.toml` configures the local MPC cluster.
- `encrypted-ixs/src/lib.rs` contains the confidential instruction.
- `programs/private_market/src/lib.rs` queues the computation and handles the
  callback.

See `docs/development.md` for environment notes.

## Submission Notes

This project targets the prompt directly:

- Functional Solana project structure integrated with Arcium.
- Clear privacy story for prediction and stake confidentiality.
- Open-source friendly repo layout.
- English README and docs.
