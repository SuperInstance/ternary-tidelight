# ternary-tidelight: Temporal rhythm and timing coordination across the fleet

## Why This Exists

A fleet of rooms can't all act at once — they need shared timing. Without synchronization, rooms make conflicting decisions, waste resources, and can't coordinate maintenance. This crate provides the clock, phase alignment, and day/night cycle primitives that let rooms tick together (or at known offsets) so the fleet moves as a coherent system.

## Core Concepts

**TideClock**: A global tick counter with configurable sync intervals. Every `sync_interval` ticks, all rooms synchronize. Think of it as a metronome for the fleet.

**Phase**: The temporal offset between two rooms. A room with phase offset 3 ticks behind another will always be 3 ticks late. Phases wrap around a period.

**TidePool**: A group of rooms that share a synchronization period. Rooms within a pool can have individual offsets but tick to the same rhythm. Alignment measures how many rooms are in-phase at any given tick.

**TidePredictor**: Records observations and detects periodicity to predict future room states. Simple extrapolation, not machine learning.

**LightCycle**: Day/night resource management. Maps ticks to resource levels: Positive (full resources during day), Zero (twilight transition), Negative (minimal resources at night).

**SlackTide**: Designated quiet periods for maintenance. During slack tide, activity is Negative, meaning rooms should avoid non-essential operations.

## Quick Start

```toml
[dependencies]
ternary-tidelight = "0.1"
```

```rust
use ternary_tidelight::{TideClock, TidePool, LightCycle, Ternary};

// Set up a clock that syncs every 100 ticks
let mut clock = TideClock::new(100);
clock.advance_by(50);
assert!(!clock.is_sync_tick()); // not at 100 yet
clock.advance_by(50);
assert!(clock.is_sync_tick()); // 100th tick

// Create a tide pool with 3 rooms
let mut pool = TidePool::new("main-pool", 10);
pool.add_room("room-a", 0);
pool.add_room("room-b", 5);
pool.add_room("room-c", 0);
let in_phase = pool.in_phase_rooms(20); // room-a and room-c are in phase

// Day/night cycle (24-tick day, day is ticks 6-18)
let cycle = LightCycle::new(24, 6, 18);
assert!(cycle.is_day(12));
assert_eq!(cycle.resource_level(3), Ternary::Negative); // night
```

## API Overview

| Type | What it is |
|------|-----------|
| `TideClock` | Global tick counter with sync interval |
| `Phase` | Phase offset between rooms with period wrapping |
| `TidePool` | Group of phase-locked rooms with alignment metrics |
| `TidePredictor` | Records history and predicts future ternary states |
| `LightCycle` | Day/night cycle mapping ticks to resource levels |
| `SlackTide` | Maintenance windows with activity levels |
| `Ternary` | The three values: Negative, Zero, Positive |

## How It Works

The TideClock is a simple monotonic counter. Sync ticks occur at multiples of `sync_interval`. This is intentionally simple — no vector clocks, no distributed consensus. For a single-process fleet, a global counter is sufficient.

Phase alignment uses modular arithmetic. A room with offset 5 and period 10 is in-phase when `(tick + 5) % 10 == 0`. The TidePool computes this for all rooms and reports alignment as a fraction.

TidePredictor records (tick, Ternary) observations and attempts to detect periodicity by checking if values repeat at candidate periods. This is brute-force but works for short histories. For long histories or complex patterns, it falls back to the last known value.

LightCycle uses interval arithmetic on the cycle phase. Day/night boundaries produce Zero (twilight) resource levels for one tick on each side of the transition.

## Known Limitations

- **Single-process only**: TideClock is not distributed. Multiple processes need external synchronization.
- **Naive periodicity detection**: TidePredictor tries all candidate periods up to half the history length. This is O(n²) in history size and won't detect complex patterns (e.g., patterns that only repeat every 3rd cycle).
- **No drift correction**: If a room's internal clock drifts from the TideClock, there's no mechanism to detect or correct it.
- **Fixed ternary resource levels**: LightCycle produces only three resource levels. Fine-grained resource management needs a different approach.

## Use Cases

- **Room synchronization**: Keep all rooms in a building ticking at the same rhythm.
- **Phase-offset coordination**: Rooms that shouldn't act simultaneously get different phase offsets.
- **Maintenance scheduling**: SlackTide defines when rooms enter low-activity mode for updates.
- **Resource planning**: LightCycle tells agents whether resources are abundant (day) or scarce (night).
- **State prediction**: TidePredictor lets a captain anticipate what a room will be doing at a future tick.

## Ecosystem Context

Part of the SuperInstance ternary ecosystem, inspired by Oracle1's Tide Pool interconnection layer:

- `ternary-captain` uses TideClock timing to coordinate fleet decisions
- `ternary-agent` agents use LightCycle resource levels to modulate behavior
- `ternary-flux` can use phase information to schedule flow evaluation
- `ternary-muse` can generate patterns synchronized to tide rhythms

No external dependencies — pure Rust.

## License

MIT
