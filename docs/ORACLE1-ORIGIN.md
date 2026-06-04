# Oracle1 Origin: Tidelight → ternary-tidelight

## Oracle1 Concept
**Layer 2: Tide Pool** — Async bulletin board system via the Bottle Protocol. The Tide Pool is the fleet's primary async communication mechanism: agents drop messages (bottles) in shared directories, and beachcomb sweeps pick them up.

From Oracle1's 6-layer interconnection model:
> Tide Pool — Async BBS (Bottle Protocol) — Status: Active

Additionally, Oracle1 uses **temporal patterns** throughout:
- Beachcomb sweeps at configurable intervals (15min–2hr)
- PLATO pipeline runs hourly
- CFP monitor runs every 15 min
- Ambient briefing runs every 30 min
- The conservation law includes a logarithmic temporal component

The fleet's GIT-AGENT-STANDARD lifecycle is explicitly cyclic:
```
PULL → BOOT → WORK → LEARN → PUSH → SLEEP → (repeat)
```

### Oracle1's Temporal Concepts
- **Seasonal effects** in FLUX emergence: 9.2x fitness amplification from cyclical constraints
- **Circadian rhythms** in edge ISA: ATP budgets follow day/night cycles
- **Experiment wheel**: cyclical hypothesize → build → measure → debrief → question → redesign
- **Tile forge philosophy**: overnight crystallization of experience (1,920 tiles/night on Jetson)

## What We Borrowed
The **tidal rhythm metaphor** for temporal coordination:
- Tide clocks for synchronizing room ticks across the fleet
- Phase offsets for rooms that tick at different rates
- Tide pools for groups of phase-locked rooms
- Light cycles for day/night resource management
- Slack tides for quiet maintenance periods
- Tide predictors for forecasting future states

Specific concepts adapted:
- **TideClock** → Oracle1's beachcomb sweep intervals (synchronization pulses)
- **Phase** → Oracle1's different agents operating on different schedules (JC1 vs Oracle1 vs Forgemaster)
- **TidePool** → Oracle1's Tide Pool layer (groups of rooms sharing a communication rhythm)
- **LightCycle** → Oracle1's seasonal/circadian resource patterns (9.2x fitness amplification)
- **SlackTide** → Oracle1's "quiet hours" (agents sleep, maintenance runs)
- **TidePredictor** → Oracle1's experiment extrapolation (predicting fleet behavior)

## How Our Implementation Differs

| Aspect | Oracle1's Tide Pool/Temporal | Our ternary-tidelight |
|---|---|---|
| **Core abstraction** | Beachcomb polling intervals | `TideClock` with sync intervals and phase math |
| **Phase coordination** | Ad-hoc (different sweep schedules) | `Phase` with formal offset/period arithmetic |
| **Room grouping** | Implicit (agents that poll each other) | `TidePool` with explicit phase-locked membership |
| **Day/night** | Hard-coded service schedules | `LightCycle` with configurable cycle/day-start/day-end |
| **Maintenance** | ad-hoc ("quiet hours") | `SlackTide` with ternary activity levels |
| **Prediction** | None | `TidePredictor` with period detection and state forecasting |
| **Ternary** | Not ternary-aware | Resource levels and activity are ternary (Positive/Zero/Negative) |

### Key Innovation: Formal Phase Arithmetic
Our `Phase` type supports arithmetic (addition with LCM of periods). Oracle1 has no formal phase model — each agent just runs on its own schedule. We can compute when two rooms will be in phase, how long until alignment, and compose phases mathematically.

### Key Innovation: Ternary Resource Levels
Our `LightCycle` produces ternary resource levels:
- **Positive** = Day (full resources)
- **Zero** = Twilight (transitioning)
- **Negative** = Night (minimal resources)

Oracle1's circadian model is binary (active/sleeping). We add the twilight transition state.

### Key Innovation: Tide Prediction
Our `TidePredictor` records room state observations and detects periodicity to forecast future states. Oracle1 has no prediction mechanism — agents react to what they find on beachcomb, they don't forecast what they'll find.

### Key Innovation: Slack Tide Activity Levels
Our `SlackTide` produces ternary activity levels:
- **Positive** = Active (normal operation)
- **Zero** = Transitioning (entering/leaving slack)
- **Negative** = Slack (maintenance period)

This enables agents to plan: "I have 3 ticks until slack tide, I should finish this work before maintenance."

### Connection to Oracle1's Science
The most direct connection: Oracle1's FLUX emergence experiments found that **seasonal effects produce 9.2x fitness amplification**. Our `LightCycle` is the formalization of this finding — cyclical resource constraints make agents more fit. Day/night isn't just a schedule; it's a fitness amplifier.

The **constraint stacking** result (5.71x improvement from multiple constraints) maps to stacking our temporal constraints: TideClock sync + LightCycle day/night + SlackTide maintenance = three simultaneous temporal constraints that compound.

## See Also
- Oracle1 Architecture Review: `construct-coordination/notes/main/ORACLE1-ARCHITECTURE-REVIEW.md`
- Oracle1-Ternary Bridge: `construct-coordination/notes/main/ORACLE1-TERNARY-BRIDGE.md`
- Oracle1 Science Review: `construct-coordination/notes/main/ORACLE1-SCIENCE-REVIEW.md`
- FLUX emergence seasonal effects in oracle1-vessel/research/
