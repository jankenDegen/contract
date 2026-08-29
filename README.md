# JankenDegen Solana Program

JankenDegen is a native Solana program written in Rust. It implements four
lamport-based games:

- Janken (rock-paper-scissors) with a commit-reveal flow
- A 100-number raffle
- A six-face multiplayer dice game
- Six-seat Russian roulette tables

The program uses Borsh for instruction payloads and account state. It is built
directly on `solana-program` and does not use Anchor.

## Toolchain

- Rust `1.92.0`, pinned in `rust-toolchain.toml`
- Solana/Agave CLI with `cargo build-sbf`
- `solana-program` `3.0.0`
- Borsh `1.5.1`

Install the Solana CLI before building the SBF program. Confirm the active
tools with:

```bash
rustc --version
cargo --version
solana --version
cargo build-sbf --version
```

## Build And Test

Run these commands from the contract directory:

```bash
cargo test
cargo build-sbf
```

Format the Rust sources separately when preparing code changes:

```bash
cargo fmt
```

The deployable program is written to:

```text
target/deploy/jankendegen.so
```

The crate supports a `no-entrypoint` feature for consumers that need the
program as a dependency without exporting its Solana entrypoint.

## Deploy

Print the address associated with the generated program keypair:

```bash
solana-keygen pubkey target/deploy/jankendegen-keypair.json
```

Deploy or upgrade the program with the same program keypair:

```bash
solana program deploy \
  target/deploy/jankendegen.so \
  --program-id target/deploy/jankendegen-keypair.json
```

The program ID is not hard-coded in the Rust crate. PDA derivations use the
program ID supplied by the Solana runtime.

## Source Layout

| File | Responsibility |
| --- | --- |
| `src/entrypoint.rs` | Solana entrypoint and security metadata |
| `src/processor.rs` | Instruction dispatch |
| `src/instruction.rs` | One-byte instruction tags and payload decoding |
| `src/state.rs` | Borsh payload and account-state types |
| `src/constants.rs` | PDA seeds, account sizes, statuses, and game limits |
| `src/initialize.rs` | Initial manager, config, and roulette table creation |
| `src/admin.rs` | Admin configuration, fee collection, and account closure |
| `src/janken.rs` | Commit-reveal rock-paper-scissors |
| `src/raffle.rs` | Ticket sales, VRF draw, and prize claims |
| `src/dice.rs` | Face selection, VRF draw, and winner settlement |
| `src/russian_roulette.rs` | Seat, round, VRF, payout, retry, and reset logic |
| `src/magicblock_vrf.rs` | MagicBlock VRF request and callback validation |
| `src/utils.rs` | Admin checks and PDA creation |
| `src/error.rs` | Custom program errors |

## Instruction Encoding

Instruction data starts with a one-byte tag. Any remaining bytes are the Borsh
encoding of the listed payload. VRF callbacks are the exception: their payload
is a raw 32-byte randomness value immediately after the tag.

| Tag | Variant | Payload |
| ---: | --- | --- |
| 0 | `InitGame` | `InitGame` |
| 1 | `JoinGame` | `JoinGame` |
| 2 | `Reveal` | `Reveal` |
| 4 | `TimeIsUp` | None |
| 7 | `SetConfig` | None |
| 8 | `InitAccounts` | None |
| 9 | `SetManager` | `Manager` |
| 10 | `CollectFee` | None |
| 11 | `SetRaffleManager` | `RaffleManager` |
| 12 | `CreateRaffle` | None |
| 13 | `BuyTicket` | `Buy` |
| 14 | `RequestRaffleDraw` | None |
| 15 | `FinalizeRaffleDraw` | None |
| 16 | `ClaimPrize` | None |
| 17 | `CloseRaffle` | None |
| 18 | `SetDiceManager` | `DiceManager` |
| 19 | `InitDiceManager` | None |
| 20 | `CreateDiceGame` | `InitDice` |
| 21 | `JoinDiceGame` | `JoinDice` |
| 22 | `RequestDiceDraw` | None |
| 23 | `FinalizeDiceDraw` | None |
| 24 | `CloseDiceGame` | None |
| 25 | `SetRussianRouletteTable` | `UpdateRussianRouletteTable` |
| 26 | `InitRussianRouletteTable` | `InitRussianRouletteTable` |
| 27 | `CreateRussianRouletteGame` | `InitRussianRoulette` |
| 28 | `JoinRussianRouletteGame` | `JoinRussianRoulette` |
| 29 | `RequestRussianRouletteDraw` | `u64` expected round ID |
| 30 | `FinalizeRussianRouletteDraw` | `u64` expected round ID |
| 31 | `CloseRussianRouletteGame` | None |
| 32 | `ResetRussianRouletteTable` | `u64` expected round ID |
| 33 | `RetryRussianRouletteDraw` | `u64` expected round ID |
| 90 | `RaffleVrfCallback` | Raw `[u8; 32]` randomness |
| 91 | `DiceVrfCallback` | Raw `[u8; 32]` randomness |
| 92 | `RussianRouletteVrfCallback` | Raw `[u8; 32]` randomness |

Tags `3`, `5`, and `6` are currently unused.

## Program-Derived Accounts

Integer seed components use little-endian byte order.

| Account | Seeds | Space |
| --- | --- | ---: |
| Config | `[b"config"]` | 161 bytes |
| Fee manager | `[b"manager"]` | 27 bytes |
| Raffle manager | `[b"rafflemanager"]` | 45 bytes |
| Dice manager | `[b"dicemanager"]` | 17 bytes |
| Janken game | `[b"game", commitment_hash]` | 124 bytes |
| Raffle | `[b"raffle", raffle_no_le]` | 179 bytes |
| Ticket | `[b"ticket", ticket_no, b"raffle", raffle_no_le]` | 69 bytes |
| Dice game | `[b"dice", game_id_le]` | 281 bytes |
| Roulette table | `[b"russianroulette", table_id]` | 284 bytes |

## Account State

All program-owned state is serialized with Borsh and has no Anchor account
discriminator.

| State | Purpose |
| --- | --- |
| `Config` | Five authorized administrator public keys |
| `Manager` | Janken timing, basis-point fee, minimum stake, collected fees |
| `Game` | Janken commitment, players, stake, deadline, and decisions |
| `RaffleManager` | Raffle number and payout configuration |
| `RaffleState` | Ticket bitmap, pricing, prizes, draw state, and VRF value |
| `Ticket` | Ticket number, raffle number, beneficiary, and rent payer |
| `DiceManager` | Minimum stake per face and fixed program fee |
| `DiceGame` | Face ownership, players, draw state, VRF value, and winner |
| `RussianRouletteGame` | Persistent table configuration, round, seats, and result |

Raffle, dice, and roulette draw statuses use:

| Value | Status |
| ---: | --- |
| 0 | Open |
| 1 | VRF pending |
| 2 | Drawn |

Janken game state uses `1` for waiting for a guest and `2` for waiting for the
initializer's reveal.

## Ordered Accounts

Account arrays are positional. The following lists show the order consumed by
the Rust processor. Accounts being created or changed, and accounts receiving
or sending lamports, must be writable. The Janken manager passed to `InitGame`
is intentionally read-only. Payers and player/admin accounts checked as
signers must be marked as signers in the instruction metas.

### Initialization And Administration

| Instruction | Ordered accounts |
| --- | --- |
| `InitAccounts` | payer, manager, admin 1, admin 2, admin 3, admin 4, admin 5, config, raffle manager, system program |
| `SetConfig` | admin, admin 1, admin 2, admin 3, admin 4, admin 5, config |
| `SetManager` | admin, manager, config |
| `CollectFee` | admin, manager, config |
| `SetRaffleManager` | admin, raffle manager, config |
| `SetDiceManager` | admin, dice manager, config |
| `InitDiceManager` | admin, dice manager, config, system program |
| `SetRussianRouletteTable` | admin, roulette table, config |
| `InitRussianRouletteTable` | admin, roulette table, config, system program |
| `CloseRaffle` | admin, raffle, config |
| `CloseDiceGame` | admin, dice game, config |
| `CloseRussianRouletteGame` | admin, roulette table, config |

### Janken

| Instruction | Ordered accounts |
| --- | --- |
| `InitGame` | initializer, game, manager, system program |
| `JoinGame` | guest, game, system program |
| `Reveal` | initializer, guest, game, manager |
| `TimeIsUp` | game, manager, guest, initializer |

### Raffle

| Instruction | Ordered accounts |
| --- | --- |
| `CreateRaffle` | admin, raffle, raffle manager, config, system program |
| `BuyTicket` | payer, player, ticket, raffle, system program |
| `RequestRaffleDraw` | payer, raffle, VRF request identity, oracle queue, system program, slot hashes sysvar, VRF program |
| `RaffleVrfCallback` | VRF callback identity, raffle |
| `FinalizeRaffleDraw` | raffle, fee manager, winning ticket |
| `ClaimPrize` | payer, player, ticket, raffle |

### Dice

| Instruction | Ordered accounts |
| --- | --- |
| `CreateDiceGame` | player, dice game, dice manager, system program |
| `JoinDiceGame` | player, dice game, system program |
| `RequestDiceDraw` | payer, dice game, VRF request identity, oracle queue, system program, slot hashes sysvar, VRF program |
| `DiceVrfCallback` | VRF callback identity, dice game |
| `FinalizeDiceDraw` | dice game, dice manager, fee manager, face 1 player, face 2 player, face 3 player, face 4 player, face 5 player, face 6 player |

### Russian Roulette

| Instruction | Ordered accounts |
| --- | --- |
| `CreateRussianRouletteGame` | player, roulette table, system program |
| `JoinRussianRouletteGame` | player, roulette table, system program |
| `RequestRussianRouletteDraw` | payer, table, VRF request identity, oracle queue, system program, slot hashes sysvar, VRF program, fee manager, seats 1 through 6 |
| `RetryRussianRouletteDraw` | payer, table, VRF request identity, oracle queue, system program, slot hashes sysvar, VRF program, fee manager, seats 1 through 6 |
| `RussianRouletteVrfCallback` | VRF callback identity, table |
| `FinalizeRussianRouletteDraw` | table, fee manager, seats 1 through 6 |
| `ResetRussianRouletteTable` | admin, table, config |

## Game Rules

### Janken

The initializer commits to a throw with:

```text
keccak256(decision_u8 || seed_32_bytes)
```

Decision values are `1 = rock`, `2 = paper`, and `3 = scissors`. The
initializer funds one stake when creating the game. The guest chooses a throw
and funds an equal stake. The initializer then reveals the decision and seed.

The manager fee is expressed in basis points with a denominator of `10_000`.
The remaining pot goes to the winner or is split on a draw. If the initializer
does not reveal within `allowed_time`, `TimeIsUp` awards the remaining pot to
the guest after deducting the fee.

Initial manager settings are:

| Setting | Value |
| --- | ---: |
| Reveal time | 60 seconds |
| Fee | 100 basis points (1%) |
| Minimum stake | 10,000,000 lamports |

### Raffle

Each raffle has ticket numbers `0..99`. A draw can be requested only after all
100 tickets are sold. MagicBlock VRF selects one sold ticket. After finalizing
the draw, every ticket is claimed individually:

- Exact ticket match receives `winner_prize`.
- Same unit digit receives `unit_digit_match_prize`.
- Same tens digit receives `decimal_digit_match_prize`.
- Other tickets receive no prize.

Claiming closes the ticket account and returns its rent to the original payer.
The raffle can be closed by an admin after all ticket accounts have been
claimed.

Initial raffle settings are:

| Setting | Value |
| --- | ---: |
| Ticket price | 20,000,000 lamports |
| Exact winner | 1,000,000,000 lamports |
| Same tens digit | 40,000,000 lamports |
| Same unit digit | 60,000,000 lamports |
| Program fee | 100,000,000 lamports |

### Dice

The six dice faces are independent positions. A player may select one or more
unclaimed faces in a single instruction and pays the configured stake for each
face. Duplicate and out-of-range selections are rejected.

Once all six faces are assigned, VRF produces a winning face in `1..=6`. The
owner of that face receives the six-stake pot minus the fixed dice program fee.
The initial minimum stake is `10,000,000` lamports per face and the initial fee
is `3,000,000` lamports.

### Russian Roulette

Roulette uses three persistent table accounts with IDs `0..2`. Each table has
six numbered seats and a monotonically increasing round ID. Seat and lifecycle
instructions include the expected round ID to reject stale transactions.

Each player pays the table stake plus a fixed `10,000,000` lamport participation
fee. When all seats are occupied, VRF selects one losing seat. The losing stake
is split across the five survivors. Participation fees and any division
remainder are moved to the fee manager.

After settlement, an admin resets the table. Resetting increments the round ID,
clears all seats, and reopens the table. A pending request that has not received
randomness can be submitted again with `RetryRussianRouletteDraw`.

The constants module provides these table stake presets:

| Preset | Value |
| --- | ---: |
| Starter | 100,000,000 lamports |
| Prime | 500,000,000 lamports |
| Apex | 1,000,000,000 lamports |

Table initialization enforces a nonzero stake and the fixed participation fee;
the caller chooses which preset value to assign to each table.

## MagicBlock VRF

Raffle, dice, and roulette randomness is requested from the MagicBlock VRF
program:

```text
Vrf1RNUjXmQGjmQrQLvJHs9SNkvDJEsRVFPkfSQUwGz
```

Requests accept either the default queue or the default ephemeral queue. Each
game records a deterministic request seed and enters the pending state before
settlement. Callback instructions can only be invoked by the signer identity
PDA derived under the VRF program for this program ID.

A plain `solana-test-validator` does not provide the MagicBlock VRF program.
VRF-dependent integration tests need that program deployed or a compatible
test double. Unit tests do not require a validator.

## Administration And Fees

The config account stores five administrator keys. Any one configured admin can
change manager settings, initialize or update roulette tables, collect fees,
and close eligible program accounts.

Game fees accumulate as lamports in the fee manager PDA and as the
`collected_fee` field in `Manager`. `CollectFee` transfers the recorded amount
to the calling admin and resets the counter.

All monetary values are lamports. One SOL is `1,000,000,000` lamports.

## Custom Errors

The program returns `ProgramError::Custom(code)` using this zero-based mapping:

| Code | Error |
| ---: | --- |
| 0 | `InvalidInstruction` |
| 1 | `ManagerWritable` |
| 2 | `ArithmeticError` |
| 3 | `InvalidCounter` |
| 4 | `MinStake` |
| 5 | `InvalidGameAccount` |
| 6 | `PlayerNotSigner` |
| 7 | `InvalidHash` |
| 8 | `InvalidTime` |
| 9 | `InvalidGameState` |
| 10 | `InvalidManager` |
| 11 | `InvalidAuth` |
| 12 | `NotSignerAuth` |
| 13 | `InvalidInitializer` |
| 14 | `InvalidGuest` |
| 15 | `InvalidConfig` |
| 16 | `InvalidRaffleAccount` |
| 17 | `InvalidTicketNo` |
| 18 | `TikcetsNotSold` |
| 19 | `InvalidDerivedAccount` |
| 20 | `AccountAlreadyInitialized` |
| 21 | `InvalidRaffleState` |
| 22 | `RaffleDrawAlreadyRequested` |
| 23 | `RaffleDrawPending` |
| 24 | `InvalidVrfAccount` |
| 25 | `InvalidTicketAccount` |
| 26 | `InvalidPlayer` |
| 27 | `InvalidFeePercentage` |
| 28 | `InvalidRaffleConfiguration` |
| 29 | `InvalidChosenDice` |
| 30 | `DiceGameNotReady` |
| 31 | `DiceDrawAlreadyRequested` |
| 32 | `DiceDrawPending` |
| 33 | `InvalidDiceConfiguration` |
| 34 | `RussianRouletteGameNotReady` |
| 35 | `RussianRouletteDrawAlreadyRequested` |
| 36 | `RussianRouletteDrawPending` |
| 37 | `InvalidRussianRouletteConfiguration` |
| 38 | `InvalidPayer` |

## Tests

The Rust unit tests currently cover:

- Round-bound roulette instruction decoding
- Rejection of missing roulette round IDs
- Ticket account size consistency with its Borsh layout
- Raffle winner selection from the sold-ticket bitmap
- Rejection of inconsistent raffle ticket bitmaps

Run them with:

```bash
cargo test
```
