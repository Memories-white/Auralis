// SPDX-License-Identifier: Apache-2.0
// Rust adaptation of Melox LyricRowMotion.kt / LyricsView.kt motion semantics.
// Upstream: Inefy-03/Melox, commit 97cae45b6acf8a0d1a0a89282d1f7ca02d198ad7.
// Modified for DOM row placement and a shared requestAnimationFrame clock.
#[derive(Default)]
pub struct RowMotion {
    pub offset: f64,
    velocity: f64,
    wait: f64,
}
impl RowMotion {
    pub fn place(&mut self, travel: f64, delay_ms: f64) {
        self.offset += travel;
        self.wait = delay_ms / 1000.0;
    }
    pub fn sample(&mut self, dt: f64, stiffness: f64) -> f64 {
        let elapsed = dt.min(self.wait);
        self.wait -= elapsed;
        let remaining = dt - elapsed;
        // Exact overdamped spring: identical travel at 60/90/120/144 Hz.
        let omega = stiffness.sqrt();
        let root = (1.1_f64 * 1.1 - 1.0).sqrt();
        let a = -omega * (1.1 - root);
        let b = -omega * (1.1 + root);
        let c = (self.velocity - b * self.offset) / (a - b);
        let d = self.offset - c;
        let ea = (a * remaining).exp();
        let eb = (b * remaining).exp();
        self.offset = c * ea + d * eb;
        self.velocity = a * c * ea + b * d * eb;
        if self.offset.abs() < 0.01 && self.velocity.abs() < 0.01 {
            self.offset = 0.0;
            self.velocity = 0.0;
        }
        self.offset
    }
}
pub fn stiffness(gap_ms: i32) -> f64 {
    (80.0 * (1000.0 / gap_ms.max(1) as f64).max(1.0).powi(2)).min(1200.0)
}
pub fn delay(rank: usize, count: usize) -> f64 {
    rank as f64 * (200.0 / count.saturating_sub(1).max(1) as f64).min(50.0)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cascade_is_bounded() {
        assert_eq!(delay(5, 6), 200.0);
        assert_eq!(delay(1, 2), 50.0);
    }
    #[test]
    fn refresh_rates_share_the_same_trajectory() {
        let mut results = Vec::new();
        for hz in [60, 90, 120, 144] {
            let mut r = RowMotion::default();
            r.place(200.0, 0.0);
            for _ in 0..hz {
                r.sample(1.0 / hz as f64, 80.0);
            }
            results.push(r.offset);
        }
        assert!(results.iter().all(|v| (v - results[0]).abs() < 0.00001));
    }
    #[test]
    fn quick_lines_accelerate() {
        assert_eq!(stiffness(4000), 80.0);
        assert!(stiffness(300) > stiffness(1000));
        assert_eq!(stiffness(10), 1200.0);
    }
    #[test]
    fn interrupted_motion_retains_position() {
        let mut row = RowMotion::default();
        row.place(100.0, 0.0);
        row.sample(0.016, 80.0);
        let before = row.offset;
        row.place(40.0, 0.0);
        assert_eq!(row.offset, before + 40.0);
        for _ in 0..500 {
            row.sample(0.016, 80.0);
        }
        assert!(row.offset.abs() < 0.01);
    }
}
