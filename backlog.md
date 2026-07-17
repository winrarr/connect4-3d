# Backlog

This document contains planned outcomes that are intentionally out of scope
for the current implementation. Move an item here only when it is a real,
user-relevant outcome; remove it when the outcome is delivered or no longer
wanted.

## Planned

### Type the internal game state

- **Status:** Planned
- **Outcome:** Evaluate the board and related game-state representations and
  introduce domain types where they make invalid state or coordinate mistakes
  harder to express.
- **Completion:** The resulting types improve the rule/placement code without
  changing Connect 4 behavior, and focused tests cover the affected rules.

### Restrict rod hit areas

- **Status:** Planned
- **Outcome:** Make the rods unclickable through the base of the board so a
  click only selects a playable column/depth location.
- **Completion:** Pointer interaction cannot place a piece through the board
  base, while valid rod selection and online/offline placement still work.
