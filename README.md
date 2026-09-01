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

- Solana/Agave CLI `4.0.3`, pinned for `solana-verify` in
  `[workspace.metadata.cli]` in `Cargo.toml`
- The SBF Rust toolchain supplied by that Solana CLI
- `solana-program` `3.0.0`
- Borsh requirement `1.5.1` (resolved to `1.8.1` by `Cargo.lock`)

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

## Verifiable Build

Install Docker and the Solana Verify CLI, then run the deterministic build from
the repository root:

```bash
cargo install solana-verify --version 0.5.1 --locked
solana-verify build --library-name jankendegen --arch v0
solana-verify get-executable-hash target/deploy/jankendegen.so
```

`solana-verify` reads the Solana CLI `4.0.3` pin from `Cargo.toml` and selects
the corresponding digest-pinned build image. After the exact source revision
has been pushed and its ELF deployed, upload the verification record from the
official repository:

```bash
solana-verify verify-from-repo \
  -u mainnet-beta \
  --program-id Degens1tGHNhGYcMGNjTj4fMP32zrwcyeJ8ti28sfScJ \
  --commit-hash <FULL_PUBLIC_COMMIT_SHA> \
  --library-name jankendegen \
  --arch v0 \
  --keypair /path/to/upgrade-authority.json \
  https://github.com/jankenDegen/contract
```

After that command rebuilds an identical ELF and uploads the verification PDA,
submit the independent remote verification job:

```bash
solana-verify remote submit-job \
  --url mainnet-beta \
  --program-id Degens1tGHNhGYcMGNjTj4fMP32zrwcyeJ8ti28sfScJ \
  --uploader <UPGRADE_AUTHORITY_PUBKEY>
```

The repository build, deployed program, verification PDA, and remote job must
all refer to the same commit, build arguments, and ELF hash.

## Deploy

Deploy or upgrade the existing mainnet program with its upgrade-authority
keypair. Do not substitute the generated `target/deploy` keypair: it represents
a different program address.

```bash
solana program deploy \
  target/deploy/jankendegen.so \
  --program-id Degens1tGHNhGYcMGNjTj4fMP32zrwcyeJ8ti28sfScJ \
  --keypair /path/to/upgrade-authority.json \
  --fee-payer /path/to/upgrade-authority.json \
  --upgrade-authority /path/to/upgrade-authority.json \
  --url mainnet-beta \
  --use-rpc \
  --max-sign-attempts 100
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
encoding of the listed payload. VRF callback payloads are fixed-width values
whose randomness is appended by the VRF program.

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
| 24 | `CloseDiceGame` | None; confirms the preceding intrinsic close |
| 25 | `SetRussianRouletteTable` | `UpdateRussianRouletteTable` |
| 26 | `InitRussianRouletteTable` | `InitRussianRouletteTable` |
| 27 | `CreateRussianRouletteGame` | `InitRussianRoulette` |
| 28 | `JoinRussianRouletteGame` | `JoinRussianRoulette` |
| 29 | `RequestRussianRouletteDraw` | `u64` expected round ID |
| 30 | `FinalizeRussianRouletteDraw` | `u64` expected round ID |
| 31 | `CloseRussianRouletteGame` | None |
| 32 | `ResetRussianRouletteTable` | `u64` expected round ID |
| 33 | `RetryRussianRouletteDraw` | `u64` expected round ID |
| 34 | `SetRussianRouletteParticipationFee` | `UpdateRussianRouletteParticipationFee` |
| 35 | `RetryRaffleDraw` | None |
| 36 | `MarkRaffleVrfFailed` | None |
| 37 | `RefundFailedRaffleTicket` | None |
| 41 | `MarkRussianRouletteVrfFailed` | `u64` expected round ID |
| 42 | `RefundFailedRussianRouletteRound` | `u64` expected round ID |
| 43 | `CreateDiceGame` | `InitDice` with nonzero 32-byte generation entropy |
| 44 | `JoinDiceGame` | `JoinDice` with expected generation nonce |
| 45 | `RequestDiceDraw` | `[u8; 32]` expected generation nonce |
| 46 | `FinalizeDiceDraw` | `[u8; 32]` expected generation nonce |
| 47 | `RetryDiceDraw` | `[u8; 32]` expected generation nonce |
| 48 | `MarkDiceVrfFailed` | `[u8; 32]` expected generation nonce |
| 49 | `RefundFailedDiceGame` | `[u8; 32]` expected generation nonce |
| 90 | `RaffleVrfCallback` | Exactly `[u8; 32]` randomness, then `[u8; 32]` request seed |
| 91 | `DiceVrfCallback` | Exactly `[u8; 32]` randomness, then `[u8; 32]` request seed |
| 92 | `RussianRouletteVrfCallback` | `[u8; 32]` randomness, then `u64` round ID and `[u8; 32]` request seed |

Tags `3`, `5`, `6`, `20` through `23`, and `38` through `40` are unused and rejected.

## Program-Derived Accounts

Integer seed components use little-endian byte order.

| Account | Seeds | Space |
| --- | --- | ---: |
| Config | `[b"config"]` | 161 bytes |
| Fee manager | `[b"manager"]` | 27 bytes |
| Raffle manager | `[b"rafflemanager"]` | 45 bytes |
| Dice manager | `[b"dicemanager"]` | 17 bytes |
| Janken game | `[b"game", commitment_hash]` | 124 bytes |
| Raffle | `[b"raffle", raffle_no_le]` | 188 bytes |
| Ticket | `[b"ticket", ticket_no, b"raffle", raffle_no_le]` | 69 bytes |
| Dice game | `[b"dice", game_id_le]` | 298 bytes |
| Roulette table | `[b"russianroulette", table_id]` | 301 bytes |

## Account State

All program-owned state is serialized with Borsh and has no Anchor account
discriminator.

| State | Purpose |
| --- | --- |
| `Config` | Five authorized administrator public keys |
| `Manager` | Janken timing, basis-point fee, minimum stake, collected fees |
| `Game` | Janken commitment, players, stake, deadline, and decisions |
| `RaffleManager` | Raffle number and payout configuration |
| `RaffleState` | Ticket bitmap, pricing, prizes, draw state, VRF value, and retry metadata |
| `Ticket` | Ticket number, raffle number, beneficiary, and rent payer |
| `DiceManager` | Minimum stake per face and fixed program fee |
| `DiceGame` | Face ownership, players, snapshotted fee, draw state, VRF value, live generation nonce, and retry metadata |
| `RussianRouletteGame` | Persistent table configuration, round, seats, result, retry metadata, and settlement time |

Raffle, dice, and roulette draw statuses use:

| Value | Status |
| ---: | --- |
| 0 | Open |
| 1 | VRF pending |
| 2 | Drawn |
| 3 | VRF failed |

The VRF account layouts store retry and settlement metadata at these offsets:

| Account | Fields and offsets |
| --- | --- |
| Raffle | `vrf_last_request_at: i64` at 179; `vrf_retry_count: u8` at 187 |
| Dice | `generation_nonce: [u8; 32]` at 249; `program_fee: u64` at 281; `vrf_last_request_at: i64` at 289; `vrf_retry_count: u8` at 297 |
| Roulette | `vrf_last_request_at: i64` at 284; `vrf_retry_count: u8` at 292; `settled_at: i64` at 293 |

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
| `SetRussianRouletteParticipationFee` | admin, roulette table, config |
| `InitRussianRouletteTable` | admin, roulette table, config, system program |
| `CloseRaffle` | admin, raffle, config |
| `CloseDiceGame` | initializer, dice game |
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
| `RetryRaffleDraw` | payer, raffle, VRF request identity, oracle queue, system program, slot hashes sysvar, VRF program |
| `MarkRaffleVrfFailed` | raffle |
| `RaffleVrfCallback` | VRF callback identity, raffle |
| `FinalizeRaffleDraw` | raffle, fee manager, winning ticket |
| `ClaimPrize` | payer, player, ticket, raffle |
| `RefundFailedRaffleTicket` | payer, player, ticket, raffle |

### Dice

| Instruction | Ordered accounts |
| --- | --- |
| `CreateDiceGame` | player, dice game, dice manager, system program |
| `JoinDiceGame` | player, dice game, system program |
| `RequestDiceDraw` | payer, dice game, VRF request identity, oracle queue, system program, slot hashes sysvar, VRF program |
| `RetryDiceDraw` | payer, dice game, VRF request identity, oracle queue, system program, slot hashes sysvar, VRF program |
| `MarkDiceVrfFailed` | dice game |
| `DiceVrfCallback` | VRF callback identity, dice game |
| `FinalizeDiceDraw` | dice game, fee manager, face 1 player, face 2 player, face 3 player, face 4 player, face 5 player, face 6 player |
| `RefundFailedDiceGame` | dice game, face 1/initializer, face 2 player, face 3 player, face 4 player, face 5 player, face 6 player |

### Russian Roulette

| Instruction | Ordered accounts |
| --- | --- |
| `CreateRussianRouletteGame` | player, roulette table, system program |
| `JoinRussianRouletteGame` | player, roulette table, system program |
| `RequestRussianRouletteDraw` | payer, table, VRF request identity, oracle queue, system program, slot hashes sysvar, VRF program, fee manager, seats 1 through 6 |
| `RetryRussianRouletteDraw` | payer, table, VRF request identity, oracle queue, system program, slot hashes sysvar, VRF program |
| `MarkRussianRouletteVrfFailed` | table |
| `RussianRouletteVrfCallback` | VRF callback identity, table |
| `FinalizeRussianRouletteDraw` | table, fee manager, seats 1 through 6 |
| `RefundFailedRussianRouletteRound` | table, fee manager, seats 1 through 6 |
| `ResetRussianRouletteTable` | table |

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
claimed. If VRF exhausts its retry lifecycle, each ticket is instead processed
with `RefundFailedRaffleTicket`: the ticket price returns from raffle escrow to
the stored beneficiary, the ticket account's rent returns to its stored payer,
and the sold-ticket bitmap and count are cleared. No beneficiary or payer
signature is required because both destinations are fixed in the ticket state.
After all 100 refunds, an admin may close the failed raffle.

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

`CreateDiceGame` requires 32 bytes of nonzero caller entropy. The
program derives the live generation token from that entropy, the immutable game
terms, creator, PDA, and authoritative creation slot; the slot is also embedded
in the token. A draw may begin only after that creation slot. Consequently a
later generation cannot copy an earlier public token—even if a hostile creator
reuses the same entropy—because the earlier game cannot close in its creation
slot. While the game is live, the token is stored in
`DiceGame.generation_nonce`. Every join and lifecycle instruction carries the
expected token and rejects a
different live generation before transferring funds or requesting VRF. This
does not add a PDA or change the 298-byte Borsh account layout. The contract
exposes only tags 43 through 49 for Dice creation and mutation; every builder
requires the generation value.

Once all six faces are assigned, VRF produces a winning face in `1..=6`. The
owner of that face receives the six-stake pot minus the fixed dice program fee.
The fee is snapshotted into the game account when the game is created, so an
administrative fee update cannot change settlement for an existing game. The
initial minimum stake is `10,000,000` lamports per face and the initial fee is
`3,000,000` lamports. `FinalizeDiceDraw` pays the winner and fee manager,
returns the remaining rent and dust to the stored initializer, then resizes the
Dice PDA to zero and assigns it to the System Program in the same instruction.
The game ID can therefore be reused without leaving a per-game terminal account.
The backend appends `CloseDiceGame` in the same transaction as an account-state
check: it succeeds only when the supplied Dice account is an empty,
zero-lamport System account.

The initial request seed is
`sha256("dice-vrf" || "v2" || dice_pda || game_id_le || generation_nonce || request_slot_le)`.
Consequently, two closed-and-recreated generations cannot share a callback
identity even when they reuse the same PDA and game ID. Here
`generation_nonce` is the program-derived live token, not the raw creation
entropy. Retries retain that stable token-bound callback seed.

If VRF fails, `RefundFailedDiceGame` atomically returns one stake for each of
the six occupied face slots. A player who owns several faces receives one
refund per face. The instruction then returns the game's rent and any dust to
the stored initializer and closes the dice PDA. The six destination accounts
are fixed by the stored face assignments; no player signature is required.

### Russian Roulette

Roulette uses three persistent table accounts with IDs `0..2`. Each table has
six numbered seats and a monotonically increasing round ID. Seat and lifecycle
instructions include the expected round ID to reject stale transactions.

Each player pays the table stake plus that table's participation fee, which
defaults to `10,000,000` lamports. An admin can update the fee only while the
table is open and empty, using the expected round ID; a change advances the
round ID. When all seats are occupied, VRF selects one losing seat. The losing
stake is split across the five survivors. Participation fees and any division
remainder are moved to the fee manager.

After settlement, anyone may reset the table once its result has remained
available for 120 seconds. Resetting validates the exact table PDA and expected
round, increments the round ID, clears all seats, preserves the configured
participation fee, and reopens the table.

If VRF fails, `RefundFailedRussianRouletteRound` atomically returns exactly one
stake to every occupied seat. Participation fees are not refunded: all six are
moved to the canonical fee manager and added to `collected_fee`, matching the
normal fee policy. The same instruction clears the table, increments its round,
and reopens it. All destination accounts are fixed by the stored seats and
canonical manager PDA, so no player or admin signature is required.

The constants module provides these table stake presets:

| Preset | Value |
| --- | ---: |
| Starter | 100,000,000 lamports |
| Prime | 500,000,000 lamports |
| Apex | 1,000,000,000 lamports |

Table initialization enforces a nonzero stake and a participation fee below the
stake; the caller chooses which preset value to assign to each table.

## MagicBlock VRF

Raffle, dice, and roulette randomness is requested from the MagicBlock VRF
program:

```text
Vrf1RNUjXmQGjmQrQLvJHs9SNkvDJEsRVFPkfSQUwGz
```

Requests accept either the default queue or the default ephemeral queue.
Raffle, dice, and roulette bind authenticated callback arguments to the stable
request seed; roulette also binds the round ID. Tags 90 and 91 require exactly
64 payload bytes after the tag, and tag 92 requires exactly 72. Unseeded
callback payloads are rejected. Initial request seeds include the game PDA and
request slot, and every callback validates the canonical game PDA before
accepting randomness. Callback instructions can only be invoked by the signer
identity PDA derived under the VRF program for this program ID.

### Retry And Failure Lifecycle

Every VRF game follows the same permissionless recovery schedule. The payer of
a request or retry must sign and pays that VRF request; it need not be an admin
or player.

| Earliest time from initial request | Allowed transition | Stored retry count |
| ---: | --- | ---: |
| 0 seconds | Initial VRF request | 0 |
| 120 seconds | First retry | 1 |
| 240 seconds | Second retry | 2 |
| 360 seconds | Mark unresolved request `VRF failed` | 2 |

Each retry uses a unique caller seed but keeps the original stable callback
seed (and, for roulette, the original round ID). Therefore a valid response to
any of the three requests can still complete the same draw. Retry and failure
delays are measured from the preceding request timestamp, not from a client
timer.

Callbacks, failure markers, and retries all require the same unresolved pending
state: pending status plus the game's undrawn result sentinel. If a callback
lands first, failure marking is rejected because a result exists. If failure
marking lands first, a late callback is rejected because the status is no
longer pending. Solana's atomic transaction ordering makes the first confirmed
transition authoritative; neither path can partially execute.

A plain `solana-test-validator` does not provide the MagicBlock VRF program.
VRF-dependent integration tests need that program deployed or a compatible
test double. Unit tests do not require a validator.

## Administration And Fees

The config account stores five administrator keys. Any one configured admin can
change manager settings, initialize or update roulette tables, collect fees,
and close eligible raffle or roulette accounts. VRF retries, failure markers,
failed-game refunds, drawn-dice closure, and timed roulette reset are
permissionless, with their destinations and PDAs enforced by on-chain state.

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
| 39 | `VrfRetryTooEarly` |
| 40 | `VrfRetryLimitReached` |
| 41 | `VrfFailureTooEarly` |
| 42 | `VrfRetriesNotExhausted` |
| 43 | `DiceGenerationNonceRequired` |
| 44 | `DiceGenerationNonceMismatch` |
| 45 | `DiceGenerationNotMature` |

## Tests

The Rust unit tests currently cover:

- Exact instruction payloads, round/seed callback binding, and rejection of
  unseeded callbacks
- Exact 188-byte raffle, 298-byte dice, and 301-byte roulette Borsh layouts,
  including rejection of truncated account data
- Two 120-second retries followed by the final 120-second failure gate
- Callback-versus-failure mutual exclusion for all three VRF games
- Raffle winner selection and per-ticket failed-refund accounting
- Dice duplicate-player face refunds plus account-rent recovery
- Dice tag/payload decoding, nonce gates, 298-byte layout, and
  same-slot generation-specific VRF seeds
- Roulette normal settlement and failed-round stake/fee accounting and clear

Run them with:

```bash
cargo test
```
