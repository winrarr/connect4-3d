# Connect 4 3D

## Orientation

This is a Bevy 0.19 desktop game implementing Connect 4 on a 4×4×4 board.

- `src/main.rs` assembles the Bevy app and plugins.
- `src/board.rs` owns board state, piece placement, turn handling, and input.
- `src/winner.rs` contains the rule check for a winning line.
- `src/network.rs` handles optional direct online play using ICE and UDP.
- `src/camera.rs` and `src/constants.rs` contain camera behavior and shared scene dimensions.

Offline play is the default. In online play, the host is red and the guest is
blue. Chat or QR exchange is signaling only; gameplay is sent directly between
the two processes. The game does not require a game server, but difficult NATs
may still need a TURN relay in the future.

## Sources of truth and boundaries

- Keep the rules shared between offline and online moves. Changes to placement,
  turn order, board dimensions, or winner detection affect both modes.
- Keep connection-code parsing and ICE transport in `src/network.rs`; do not
  put networking logic in rendering systems.
- `Cargo.toml` and `Cargo.lock` are authoritative for dependencies and the
  supported Bevy/toolchain configuration.
- `target/` is build output and must not be committed. Do not commit secrets,
  connection codes, or local network captures.

## Commands and verification

Use the Make targets as the canonical local command interface:

```text
make run
make fmt-check
make test
make lint
make build
make verify
```

`make verify` runs formatting checks, all tests, Clippy with warnings denied,
and the release build. Online play is exercised manually with two devices on
separate networks; local and virtual ICE checks do not prove traversal through
two real routers.

## Maintenance

Keep human onboarding and play instructions in `README.md`. Keep executable
commands in the `Makefile`, and have CI call those same targets. Put durable
agent operating rules here, implementation details in code/tests, and record
only real future outcomes or consequential decisions in dedicated documents
when they arise.
