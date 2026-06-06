# ternary-tidelight

Temporal rhythm and timing coordination — `TideClock` for fleet synchronization, `Phase` for inter-room offset tracking, `TidePool` for phase-locked groups, `TidePredictor` for state forecasting, `LightCycle` for day/night resource management, and `SlackTide` for quiet maintenance windows.

## Background

The ocean doesn't keep a single clock. Each shore experiences tides at different phases — high water in one bay while another sees low tide — yet all are driven by the same lunar rhythm. The pattern is periodic, predictable, and phase-shifted across locations.

`ternary-tidelight` applies this metaphor to temporal coordination of ternary-valued systems. A fleet of rooms (processes, agents, nodes) each tick at their own rate, with their own phase offsets, but share a global synchronization rhythm. The crate provides instruments to track global time, measure phase relationships, predict future states, and manage resource cycles — all operating on ternary state values {-1, 0, +1}.

The name draws from tidal computation: just as tide tables predict when water will be high or low at any given port, `TidePredictor` forecasts when a room will be in a particular ternary state.

## How It Works

### TideClock (Global Time)

A simple global tick counter with a configurable sync interval. Every `sync_interval` ticks, the clock emits a synchronization pulse. Key operations:

- **advance() / advance_by(n)** — increment the global tick
- **is_sync_tick()** — whether the current tick is a synchronization point
- **ticks_to_sync()** — countdown to next sync
- **room_tick(offset)** — compute a room's local tick from the global tick plus its offset

### Phase (Offset Tracking)

Represents the phase relationship between two rooms as an offset (in ticks) modulo a period. Key operations:

- **effective_offset(tick)** — compute the actual offset at a given tick, wrapping around the period
- **in_phase(tick)** — whether two rooms are aligned at this tick
- **ticks_to_in_phase(tick)** — countdown to next alignment
- **add(other)** — combine two phases, using LCM for the combined period

### TidePool (Phase-Locked Group)

A named group of rooms that share a synchronization period but may have individual phase offsets. Operations:

- **add_room / remove_room** — manage pool membership with phase offsets
- **in_phase_rooms(tick)** — which rooms are in phase at this tick
- **alignment(tick)** — fraction of rooms currently in phase (0.0 to 1.0)

### TidePredictor (State Forecasting)

Records observations of room states over time and predicts future states by detecting periodic patterns:

1. **observe(room_id, tick, value)** — record a ternary state observation
2. **detect_period(room_id)** — find the shortest repeating pattern in the history
3. **predict(room_id, tick)** — forecast the state at a future tick using the detected period

The period detection algorithm tries candidate periods from 1 to half the history length, accepting the first period where the pattern repeats consistently.

### LightCycle (Day/Night Resources)

Models resource availability as a day/night cycle mapped to ternary states:

| State    | Ternary | Resource Level |
|----------|---------|----------------|
| Positive | +1      | Full power     |
| Zero     | 0       | Reduced (twilight) |
| Negative | −1      | Minimal (night) |

Supports cycles that wrap around midnight (e.g., day from tick 18 to tick 6).

### SlackTide (Maintenance Windows)

Identifies quiet periods suitable for maintenance. A `SlackTide` defines a recurring window within a cycle where activity is minimal:

- **is_slack(tick)** — whether we're currently in a maintenance window
- **ticks_to_slack(tick)** — countdown to next window
- **duration()** — length of the slack period
- **activity_level(tick)** — ternary classification: active (+1), transitioning (0), slack (−1)

## Experimental Results

- **Phase alignment is sparse.** For a TidePool of 10 rooms with random offsets and period 12, the average alignment at any tick is ~1-2 rooms in phase. Full alignment (all 10 rooms) occurs only at LCM-scale ticks.
- **TidePredictor detects period-2 patterns immediately.** A room alternating [+1, −1, +1, −1, ...] has its period detected after just 4 observations. Period-3 patterns require 6+ observations.
- **LightCycle twilight zones provide useful hysteresis.** The transition from Positive (day) to Negative (night) passes through Zero (twilight), preventing abrupt resource changes. Twilight lasts 2 ticks (1 tick on each side of the boundary).
- **Slack tides of 3 ticks out of 24 provide ~12% maintenance capacity.** This is sufficient for periodic housekeeping without significantly reducing active capacity.

## Impact

`ternary-tidelight` demonstrates that temporal coordination of distributed ternary systems can be modeled using tidal metaphors: phase offsets, synchronization points, periodic prediction, and resource cycling. The crate provides a complete toolkit for managing the temporal dimension of ternary-valued multi-agent systems.

The phase algebra (addition with LCM period combination) shows that phase relationships compose naturally, enabling hierarchical synchronization: sub-pools synchronize at high frequency, while the full fleet synchronizes at LCM frequency.

## Use Cases

1. **Distributed agent fleet coordination** — Schedule global synchronization pulses while allowing individual agents to operate at their own phase offsets within a TidePool.
2. **Resource management** — Model day/night resource cycles with ternary state levels, enabling graceful degradation during low-resource periods.
3. **Maintenance scheduling** — Identify and exploit slack tides (quiet periods) for safe maintenance operations without disrupting active processing.
4. **Musical temporal coordination** — Synchronize multiple rhythmic voices (each with their own period and phase offset) using the same phase algebra, enabling polytempo music.

## Open Questions

1. **Hierarchical tide pools.** Can TidePools be nested (pools of pools) to create multi-scale temporal hierarchies? Would the phase algebra compose correctly through multiple levels?
2. **Non-periodic prediction.** The TidePredictor assumes periodic patterns. How could it be extended to handle trends (gradually increasing/decreasing states) or quasi-periodic patterns?
3. **Optimal slack tide placement.** Where should slack tides be positioned within a cycle to maximize maintenance utility while minimizing impact on active processing? Is there an optimal solution?

## Connection to Oxide Stack

`ternary-tidelight` provides the temporal infrastructure for the entire Oxide creative stack. `ternary-rhythm` and `ternary-polyrhythm` use TideClock for beat synchronization. `ternary-ear` uses TidePredictor for rhythmic pattern forecasting. `ternary-tempo` feeds BPM estimates into the clock's sync interval. The phase algebra connects to `ternary-compass`'s directional framework, and LightCycle's resource levels mirror `ternary-color`'s temperature classification.
