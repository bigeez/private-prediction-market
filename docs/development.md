# Development

## Local Product Tests

Run the dependency-free simulation:

```bash
npm test
```

## Arcium Build

Arcium's public docs currently recommend Linux or WSL2 for Windows users. This
repository includes the Arcium/Anchor project structure, but the native Windows
shell here does not have `arcium`, `anchor`, or `solana` installed.

On WSL2 or Linux:

```bash
curl --proto '=https' --tlsv1.2 -sSfL https://install.arcium.com/ | bash
arcium build
arcium test
```

## Implementation Notes

The Solana program mirrors the Arcium voting example pattern:

- `create_market` initializes encrypted market totals through Arcium.
- `submit_prediction` queues an encrypted side and encrypted stake.
- `tally_prediction_callback` stores updated encrypted totals back on the
  market account.
- `reveal_result` reveals only the final winning side.

