# Connect 4 3D

A 4×4×4 Connect 4 game built with Bevy. It can be played locally or directly
between two players over UDP using STUN-assisted hole punching.

Requires a stable Rust toolchain and Cargo. GNU Make is used for the canonical
verification commands.

## Local play

```text
cargo run --release
```

## Online play without a game server

The host and guest exchange small connection codes through any chat app. The
codes contain the UDP candidates needed for hole punching; gameplay packets go
directly between the two game processes.

On the host:

```text
cargo run --release -- --host
```

Copy the printed `C4O2|...` offer to the other player. They run:

```text
cargo run --release -- --join 'C4O2|...'
```

They send the printed `C4A2|...` answer back to the host, who pastes it into
the host terminal. Once both windows report a direct connection, the host
plays red and the guest plays blue.

The offer and answer strings can also be put into a QR-code generator for
phone-to-computer signaling. No QR or chat service is built into the game, so
no server account or additional CLI tool is required.

The default STUN server is `stun.l.google.com:19302`. It can be overridden for
testing with `CONNECT4_STUN_SERVER=host:port`. Direct connections are not
guaranteed on symmetric NATs, CGNAT, or networks that block UDP; those cases
would need a relay service.

## Development verification

```text
make verify
```

The individual checks are `make fmt-check`, `make test`, `make lint`, and
`make build`. CI runs the same `make verify` target.

Repository operating instructions for coding agents are in [AGENTS.md](AGENTS.md).
