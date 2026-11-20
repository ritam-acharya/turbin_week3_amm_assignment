# Turbin Week 3 AMM Assignment

An [Anchor](https://www.anchor-lang.com/) program implementing a constant-product automated market maker (AMM) on Solana.

The program manages a liquidity pool for two SPL tokens, mints LP tokens to liquidity providers, supports proportional withdrawals, and applies a configurable swap fee with slippage protection.

## Features

- Initialize a pool with two SPL token mints.
- Store pool configuration in a PDA derived from a numeric seed.
- Create an LP token mint controlled by the pool configuration PDA.
- Deposit token X and token Y and receive LP tokens.
- Withdraw liquidity by burning LP tokens.
- Swap token X for token Y or token Y for token X.
- Enforce maximum deposit and minimum withdrawal/swap amounts.
- Run integration tests in [LiteSVM](https://github.com/LiteSVM/litesvm) without a local validator.

## Program details

| Item | Value |
| --- | --- |
| Program ID | `6Rk7ZKqJbfMY97yb13you6LorRg1Q1RESDSuERNgYoW6` |
| Anchor entrypoint | `turbin_week3_assignment` |
| Package | `turbin_week3_amm_assignment` |
| LP token decimals | 6 |
| Default test cluster | `localnet` |

## Instructions

The on-chain program exposes four instructions:

### `initialize`

Creates the pool configuration, LP mint, and token vaults.

Arguments:

- `fee`: swap fee in basis points (`30` = `0.30%`).
- `seed`: numeric seed used to derive the config PDA.
- `authority`: optional pool authority stored in configuration.

### `deposit`

Adds token X and token Y to the pool and mints LP tokens.

Arguments:

- `amount`: LP token amount to mint.
- `max_x`: maximum token X accepted.
- `max_y`: maximum token Y accepted.

### `withdraw`

Burns LP tokens and returns the proportional share of both pool reserves.

Arguments:

- `amount`: LP tokens to burn.
- `min_x`: minimum token X returned.
- `min_y`: minimum token Y returned.

### `swap`

Swaps one pool token for the other using the constant-product curve.

Arguments:

- `is_x`: `true` to deposit token X and receive token Y; `false` for the reverse.
- `amount`: input token amount.
- `min`: minimum output token amount.

## Project layout

```text
.
├── Anchor.toml
├── Cargo.toml
├── programs/
│   └── turbin_week3_amm_assignment/
│       ├── src/
│       │   ├── instructions/   # initialize, deposit, withdraw, and swap handlers
│       │   ├── state.rs        # pool configuration account
│       │   ├── error.rs        # custom program errors
│       │   └── lib.rs          # Anchor program entrypoint
│       └── tests/              # LiteSVM integration tests and instruction builders
└── rust-toolchain.toml
```

## Prerequisites

- Rust `1.89.0` or a compatible toolchain
- Solana CLI
- Anchor CLI

The required Rust toolchain is declared in `rust-toolchain.toml`. The wallet configured by `Anchor.toml` is `~/.config/solana/id.json`.

## Build and test

Clone the repository and run:

```bash
anchor build
cargo test
```

The integration tests load the compiled program into LiteSVM and cover:

- Pool initialization
- Liquidity deposits
- Liquidity withdrawals
- Successful swaps
- Swap failure when the slippage limit is exceeded

To format and lint the Rust code:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
```

## PDA structure

The program derives its main accounts as follows:

```text
config PDA = ["config", seed.to_le_bytes()]
LP mint PDA = ["lp", config.key()]
vaults      = associated token accounts owned by the config PDA
```

The pool configuration stores the token mint addresses, fee, seed, PDA bumps, optional authority, and lock state.

## License

No license has been specified for this repository yet.
