#![forbid(unsafe_code)]

//! Temporal rhythm and timing coordination across the fleet.
//!
//! Provides `TideClock` for synchronizing room ticks, `Phase` for phase offsets
//! between rooms, `TidePool` for groups of phase-locked rooms, `TidePredictor`
//! for predicting future room states, `LightCycle` for day/night resource
//! management, and `SlackTide` for quiet maintenance periods.

use std::collections::HashMap;

// ── Ternary Value ──────────────────────────────────────────────────────────

/// A balanced ternary digit: Negative (-1), Zero (0), or Positive (+1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ternary {
    Negative,
    Zero,
    Positive,
}

impl Ternary {
    pub fn to_i8(self) -> i8 {
        match self {
            Ternary::Negative => -1,
            Ternary::Zero => 0,
            Ternary::Positive => 1,
        }
    }
}

// ── Tide Clock ─────────────────────────────────────────────────────────────

/// Synchronizes room ticks across the fleet.
///
/// Each room has a tick counter. The TideClock tracks global time and
/// computes which rooms should be active at any given tick.
#[derive(Debug, Clone)]
pub struct TideClock {
    /// Current global tick.
    pub tick: u64,
    /// Tick interval between synchronization pulses.
    pub sync_interval: u64,
}

impl TideClock {
    pub fn new(sync_interval: u64) -> Self {
        Self { tick: 0, sync_interval }
    }

    /// Advance by one tick.
    pub fn advance(&mut self) {
        self.tick += 1;
    }

    /// Advance by n ticks.
    pub fn advance_by(&mut self, n: u64) {
        self.tick += n;
    }

    /// Is this a synchronization tick?
    pub fn is_sync_tick(&self) -> bool {
        self.tick > 0 && self.tick % self.sync_interval == 0
    }

    /// Ticks until next sync.
    pub fn ticks_to_sync(&self) -> u64 {
        if self.sync_interval == 0 {
            return 0;
        }
        let remainder = self.tick % self.sync_interval;
        if remainder == 0 {
            self.sync_interval
        } else {
            self.sync_interval - remainder
        }
    }

    /// Compute phase offset for a room that ticks at a different rate.
    pub fn room_tick(&self, room_offset: u64) -> u64 {
        self.tick.wrapping_add(room_offset)
    }

    /// Reset the clock.
    pub fn reset(&mut self) {
        self.tick = 0;
    }
}

// ── Phase ──────────────────────────────────────────────────────────────────

/// Represents a phase offset between two rooms.
///
/// Phase is measured in ticks and determines how rooms' rhythms relate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Phase {
    /// Offset in ticks.
    pub offset: i64,
    /// Period in ticks (how often the phase repeats).
    pub period: u64,
}

impl Phase {
    pub fn new(offset: i64, period: u64) -> Self {
        Self { offset, period: period.max(1) }
    }

    /// Compute the effective offset at a given tick (wrapping around period).
    pub fn effective_offset(&self, tick: u64) -> i64 {
        let wrapped = (tick as i64 + self.offset) % self.period as i64;
        if wrapped < 0 {
            wrapped + self.period as i64
        } else {
            wrapped
        }
    }

    /// Are two rooms in phase at this tick?
    pub fn in_phase(&self, tick: u64) -> bool {
        self.effective_offset(tick) == 0
    }

    /// Ticks until next in-phase moment.
    pub fn ticks_to_in_phase(&self, tick: u64) -> u64 {
        let eff = self.effective_offset(tick);
        if eff == 0 {
            0
        } else {
            (self.period as i64 - eff) as u64
        }
    }

    /// Add two phases.
    pub fn add(&self, other: &Phase) -> Phase {
        Phase::new(self.offset + other.offset, lcm(self.period, other.period))
    }
}

/// Least common multiple.
fn lcm(a: u64, b: u64) -> u64 {
    a / gcd(a, b) * b
}

/// Greatest common divisor.
fn gcd(a: u64, b: u64) -> u64 {
    let mut a = a;
    let mut b = b;
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

// ── Tide Pool ──────────────────────────────────────────────────────────────

/// A group of phase-locked rooms.
///
/// All rooms in a TidePool share a synchronization rhythm but may have
/// individual phase offsets.
#[derive(Debug, Clone)]
pub struct TidePool {
    pub id: String,
    /// Room ID → phase offset in ticks.
    pub rooms: HashMap<String, u64>,
    /// Sync period for the pool.
    pub period: u64,
}

impl TidePool {
    pub fn new(id: &str, period: u64) -> Self {
        Self {
            id: id.to_string(),
            rooms: HashMap::new(),
            period: period.max(1),
        }
    }

    /// Add a room with a phase offset.
    pub fn add_room(&mut self, room_id: &str, offset: u64) {
        self.rooms.insert(room_id.to_string(), offset % self.period);
    }

    /// Remove a room.
    pub fn remove_room(&mut self, room_id: &str) -> bool {
        self.rooms.remove(room_id).is_some()
    }

    /// Which rooms are in phase at the given tick?
    pub fn in_phase_rooms(&self, tick: u64) -> Vec<&str> {
        self.rooms
            .iter()
            .filter(|(_, &offset)| (tick + offset) % self.period == 0)
            .map(|(id, _)| id.as_str())
            .collect()
    }

    /// Number of rooms in the pool.
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// Compute the harmonic alignment: fraction of rooms in phase at tick.
    pub fn alignment(&self, tick: u64) -> f64 {
        if self.rooms.is_empty() {
            return 1.0;
        }
        let in_phase = self.in_phase_rooms(tick).len();
        in_phase as f64 / self.rooms.len() as f64
    }
}

// ── Tide Predictor ─────────────────────────────────────────────────────────

/// Predicts future room states from tidal patterns.
///
/// Uses simple periodic extrapolation to predict when rooms will be in
/// specific ternary states.
#[derive(Debug, Clone)]
pub struct TidePredictor {
    /// Room state histories: room ID → list of (tick, Ternary) observations.
    histories: HashMap<String, Vec<(u64, Ternary)>>,
}

impl TidePredictor {
    pub fn new() -> Self {
        Self {
            histories: HashMap::new(),
        }
    }

    /// Record an observation.
    pub fn observe(&mut self, room_id: &str, tick: u64, value: Ternary) {
        self.histories
            .entry(room_id.to_string())
            .or_default()
            .push((tick, value));
    }

    /// Predict the state at a future tick using the most recent observation's pattern.
    /// Simple: returns the most recent value if we've seen this room before.
    /// More sophisticated: checks for periodic repetition.
    pub fn predict(&self, room_id: &str, tick: u64) -> Option<Ternary> {
        let history = self.histories.get(room_id)?;
        if history.is_empty() {
            return None;
        }

        // Find periodicity: look for the shortest repeating pattern
        let period = self.detect_period(room_id);
        if period == 0 {
            // No pattern; use last known value
            return Some(history.last().unwrap().1);
        }

        // Map target tick into the pattern cycle
        let base_tick = history[0].0;
        if tick < base_tick {
            return Some(history[0].1);
        }
        let offset = (tick - base_tick) % period;
        // Find the observation at this offset
        for &(t, v) in history {
            if (t - base_tick) == offset {
                return Some(v);
            }
        }
        Some(history.last().unwrap().1)
    }

    /// Detect the period of a room's history. Returns 0 if no pattern found.
    fn detect_period(&self, room_id: &str) -> u64 {
        let history = match self.histories.get(room_id) {
            Some(h) if h.len() >= 2 => h,
            _ => return 0,
        };
        // Simple heuristic: try periods from 1 to half the history length
        let max_period = history.len() / 2;
        for period in 1..=max_period.max(1) {
            let period = period as u64;
            let base_tick = history[0].0;
            let mut matches = true;
            for &(t, v) in history {
                let idx = (t - base_tick) as usize;
                if idx >= period as usize {
                    let compare_idx = idx - period as usize;
                    if compare_idx < history.len() {
                        let (_, compare_v) = history[compare_idx];
                        if v != compare_v {
                            matches = false;
                            break;
                        }
                    }
                }
            }
            if matches && period > 0 {
                return period;
            }
        }
        // Fall back to full history span
        let span = history.last().unwrap().0 - history[0].0;
        if span > 0 { span } else { 0 }
    }

    /// Number of observations for a room.
    pub fn observation_count(&self, room_id: &str) -> usize {
        self.histories.get(room_id).map(|h| h.len()).unwrap_or(0)
    }
}

// ── Light Cycle ────────────────────────────────────────────────────────────

/// Day/night resource management for the fleet.
///
/// Resources follow a cycle: active (day) and reduced (night) phases.
/// Ternary states map to resource levels: Positive = full, Zero = reduced,
/// Negative = minimal.
#[derive(Debug, Clone)]
pub struct LightCycle {
    /// Length of a full day/night cycle in ticks.
    pub cycle_length: u64,
    /// Ticks at which day starts (within the cycle).
    pub day_start: u64,
    /// Ticks at which day ends (within the cycle).
    pub day_end: u64,
}

impl LightCycle {
    pub fn new(cycle_length: u64, day_start: u64, day_end: u64) -> Self {
        Self {
            cycle_length: cycle_length.max(1),
            day_start: day_start.min(cycle_length - 1),
            day_end: day_end.min(cycle_length),
        }
    }

    /// Is it day at the given tick?
    pub fn is_day(&self, tick: u64) -> bool {
        let phase = tick % self.cycle_length;
        if self.day_start <= self.day_end {
            phase >= self.day_start && phase < self.day_end
        } else {
            // Wraps around (e.g., day_start=18, day_end=6 → night spans midnight)
            phase >= self.day_start || phase < self.day_end
        }
    }

    /// Resource level at the given tick as a ternary value.
    pub fn resource_level(&self, tick: u64) -> Ternary {
        if self.is_day(tick) {
            Ternary::Positive
        } else {
            let phase = tick % self.cycle_length;
            // Twilight zones near transitions
            let near_day_start = phase.abs_diff(self.day_start) <= 1;
            let near_day_end = phase.abs_diff(self.day_end) <= 1;
            if near_day_start || near_day_end {
                Ternary::Zero
            } else {
                Ternary::Negative
            }
        }
    }

    /// Ticks until next day start.
    pub fn ticks_to_day(&self, tick: u64) -> u64 {
        let phase = tick % self.cycle_length;
        if phase < self.day_start {
            self.day_start - phase
        } else {
            self.cycle_length - phase + self.day_start
        }
    }

    /// Fraction of the cycle that is day.
    pub fn day_fraction(&self) -> f64 {
        if self.day_start <= self.day_end {
            (self.day_end - self.day_start) as f64 / self.cycle_length as f64
        } else {
            (self.cycle_length - self.day_start + self.day_end) as f64 / self.cycle_length as f64
        }
    }
}

// ── Slack Tide ─────────────────────────────────────────────────────────────

/// Represents quiet periods for maintenance.
///
/// Slack tides occur when room activity is minimal, allowing for safe
/// maintenance operations.
#[derive(Debug, Clone)]
pub struct SlackTide {
    /// Start tick of the slack period (within a cycle).
    pub start: u64,
    /// End tick of the slack period (within a cycle).
    pub end: u64,
    /// Cycle length for periodic slack tides.
    pub cycle: u64,
}

impl SlackTide {
    pub fn new(start: u64, end: u64, cycle: u64) -> Self {
        Self {
            start,
            end,
            cycle: cycle.max(1),
        }
    }

    /// Is this a slack period at the given tick?
    pub fn is_slack(&self, tick: u64) -> bool {
        let phase = tick % self.cycle;
        if self.start <= self.end {
            phase >= self.start && phase <= self.end
        } else {
            phase >= self.start || phase <= self.end
        }
    }

    /// Ticks until next slack period.
    pub fn ticks_to_slack(&self, tick: u64) -> u64 {
        let phase = tick % self.cycle;
        if self.is_slack(tick) {
            return 0;
        }
        if phase < self.start {
            self.start - phase
        } else {
            self.cycle - phase + self.start
        }
    }

    /// Duration of the slack period in ticks.
    pub fn duration(&self) -> u64 {
        if self.start <= self.end {
            self.end - self.start + 1
        } else {
            self.cycle - self.start + self.end + 1
        }
    }

    /// Compute a ternary activity level: Positive = active, Zero = transitioning, Negative = slack.
    pub fn activity_level(&self, tick: u64) -> Ternary {
        if self.is_slack(tick) {
            Ternary::Negative
        } else {
            let phase = tick % self.cycle;
            let near_start = phase.abs_diff(self.start) <= 1;
            let near_end = phase.abs_diff(self.end) <= 1;
            if near_start || near_end {
                Ternary::Zero
            } else {
                Ternary::Positive
            }
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tide_clock_advance() {
        let mut clock = TideClock::new(10);
        clock.advance();
        assert_eq!(clock.tick, 1);
        clock.advance_by(5);
        assert_eq!(clock.tick, 6);
    }

    #[test]
    fn test_tide_clock_sync() {
        let mut clock = TideClock::new(5);
        clock.advance_by(5);
        assert!(clock.is_sync_tick());
    }

    #[test]
    fn test_tide_clock_ticks_to_sync() {
        let clock = TideClock::new(10);
        assert_eq!(clock.ticks_to_sync(), 10);
    }

    #[test]
    fn test_tide_clock_reset() {
        let mut clock = TideClock::new(10);
        clock.advance_by(50);
        clock.reset();
        assert_eq!(clock.tick, 0);
    }

    #[test]
    fn test_tide_clock_room_tick() {
        let clock = TideClock::new(10);
        assert_eq!(clock.room_tick(5), 5);
    }

    #[test]
    fn test_phase_in_phase() {
        let phase = Phase::new(0, 10);
        assert!(phase.in_phase(0));
        assert!(phase.in_phase(10));
        assert!(!phase.in_phase(5));
    }

    #[test]
    fn test_phase_effective_offset() {
        let phase = Phase::new(3, 10);
        assert_eq!(phase.effective_offset(0), 3);
        assert_eq!(phase.effective_offset(7), 0); // wraps
    }

    #[test]
    fn test_phase_ticks_to_in_phase() {
        let phase = Phase::new(3, 10);
        assert_eq!(phase.ticks_to_in_phase(0), 7);
    }

    #[test]
    fn test_phase_add() {
        let a = Phase::new(2, 6);
        let b = Phase::new(3, 4);
        let sum = a.add(&b);
        assert_eq!(sum.offset, 5);
        assert_eq!(sum.period, 12); // lcm(6,4)
    }

    #[test]
    fn test_tide_pool_add_remove() {
        let mut pool = TidePool::new("pool1", 10);
        pool.add_room("r1", 0);
        pool.add_room("r2", 5);
        assert_eq!(pool.room_count(), 2);
        assert!(pool.remove_room("r1"));
        assert_eq!(pool.room_count(), 1);
    }

    #[test]
    fn test_tide_pool_in_phase_rooms() {
        let mut pool = TidePool::new("pool1", 10);
        pool.add_room("r1", 0);
        pool.add_room("r2", 0);
        pool.add_room("r3", 5);
        let in_phase = pool.in_phase_rooms(10);
        assert_eq!(in_phase.len(), 2);
    }

    #[test]
    fn test_tide_pool_alignment() {
        let mut pool = TidePool::new("pool1", 10);
        pool.add_room("r1", 0);
        pool.add_room("r2", 0);
        let alignment = pool.alignment(10);
        assert!((alignment - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_tide_predictor_observe_and_predict() {
        let mut pred = TidePredictor::new();
        pred.observe("r1", 0, Ternary::Positive);
        pred.observe("r1", 1, Ternary::Zero);
        pred.observe("r1", 2, Ternary::Positive);
        pred.observe("r1", 3, Ternary::Zero);
        let result = pred.predict("r1", 100);
        assert!(result.is_some());
    }

    #[test]
    fn test_tide_predictor_unknown_room() {
        let pred = TidePredictor::new();
        assert_eq!(pred.predict("unknown", 0), None);
    }

    #[test]
    fn test_tide_predictor_observation_count() {
        let mut pred = TidePredictor::new();
        pred.observe("r1", 0, Ternary::Positive);
        pred.observe("r1", 1, Ternary::Zero);
        assert_eq!(pred.observation_count("r1"), 2);
    }

    #[test]
    fn test_light_cycle_day() {
        let cycle = LightCycle::new(24, 6, 18);
        assert!(cycle.is_day(12));
        assert!(!cycle.is_day(0));
    }

    #[test]
    fn test_light_cycle_resource_level() {
        let cycle = LightCycle::new(24, 6, 18);
        assert_eq!(cycle.resource_level(12), Ternary::Positive);
        assert_eq!(cycle.resource_level(0), Ternary::Negative);
    }

    #[test]
    fn test_light_cycle_ticks_to_day() {
        let cycle = LightCycle::new(24, 6, 18);
        assert_eq!(cycle.ticks_to_day(0), 6);
        assert_eq!(cycle.ticks_to_day(3), 3);
    }

    #[test]
    fn test_light_cycle_day_fraction() {
        let cycle = LightCycle::new(24, 6, 18);
        assert!((cycle.day_fraction() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_slack_tide_is_slack() {
        let slack = SlackTide::new(2, 4, 10);
        assert!(slack.is_slack(2));
        assert!(slack.is_slack(3));
        assert!(!slack.is_slack(6));
    }

    #[test]
    fn test_slack_tide_duration() {
        let slack = SlackTide::new(2, 4, 10);
        assert_eq!(slack.duration(), 3);
    }

    #[test]
    fn test_slack_tide_ticks_to_slack() {
        let slack = SlackTide::new(2, 4, 10);
        assert_eq!(slack.ticks_to_slack(0), 2);
    }

    #[test]
    fn test_slack_tide_activity_level() {
        let slack = SlackTide::new(2, 4, 10);
        assert_eq!(slack.activity_level(3), Ternary::Negative); // slack
        assert_eq!(slack.activity_level(7), Ternary::Positive); // active
    }

    #[test]
    fn test_lcm_and_gcd() {
        assert_eq!(gcd(12, 8), 4);
        assert_eq!(lcm(6, 4), 12);
    }
}
