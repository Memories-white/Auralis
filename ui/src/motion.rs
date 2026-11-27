//! Frame-rate independent spring sampling for interface motion.
//!
//! CSS transitions cannot be interrupted with retained velocity, cannot report a
//! settled state to Rust, and react badly to variable frame intervals on Android
//! WebView. Every custom trajectory in Auralis is therefore sampled here with an
//! exact closed-form solution and written to CSS custom properties once per frame.
//!
//! The solution is exact for any `dt`, so 60/90/120/144 Hz devices follow the same
//! curve. `Dense` motion (the default) is slightly under-damped to keep the soft
//! Melox-like overshoot; `snap` removes it for reduced-motion users or when the
//! system animation scale is zero.

/// A one-dimensional spring: `position` chases `target` with retained velocity.
#[derive(Clone, Copy, Debug, Default)]
pub struct Spring {
    pub position: f64,
    velocity: f64,
    pub target: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Feel {
    /// Angular frequency in rad/s. Larger is faster.
    pub stiffness: f64,
    /// 1.0 is critically damped; below 1.0 overshoots.
    pub damping: f64,
}

impl Feel {
    /// Soft, slightly bouncy motion for the dock indicator and mini player.
    pub const DOCK: Feel = Feel {
        stiffness: 26.0,
        damping: 0.82,
    };
    /// Heavier motion for the player sheet, whose artwork morph must not wobble.
    pub const SHEET: Feel = Feel {
        stiffness: 19.0,
        damping: 1.0,
    };
    /// Short reveal used when a control group expands.
    pub const REVEAL: Feel = Feel {
        stiffness: 30.0,
        damping: 0.88,
    };
}

impl Spring {
    /// Move the target without touching the current position or velocity, so an
    /// interrupted animation continues from where the user actually sees it.
    pub fn to(&mut self, target: f64) {
        self.target = target;
    }
    pub fn place(&mut self, position: f64) {
        self.position = position;
        self.velocity = 0.0;
    }
    pub fn settled(&self) -> bool {
        (self.target - self.position).abs() < 0.0005 && self.velocity.abs() < 0.002
    }
    /// Advance the spring and return the new position.
    pub fn sample(&mut self, dt: f64, feel: Feel) -> f64 {
        let dt = dt.clamp(0.0, 0.1);
        if dt <= 0.0 {
            return self.position;
        }
        let x0 = self.position - self.target;
        let v0 = self.velocity;
        if x0.abs() < 1e-9 && v0.abs() < 1e-9 {
            self.position = self.target;
            self.velocity = 0.0;
            return self.position;
        }
        let omega = feel.stiffness.max(0.001);
        let zeta = feel.damping.clamp(0.05, 1.0);
        let (x, v) = if (zeta - 1.0).abs() < 1e-6 {
            // Critically damped: x(t) = (x0 + (v0 + ω·x0)·t)·e^(-ω·t)
            let e = (-omega * dt).exp();
            let b = v0 + omega * x0;
            (e * (x0 + b * dt), e * (v0 - omega * b * dt))
        } else {
            // Under-damped: x(t) = e^(-ζωt)·(x0·cos(ωd t) + (v0+ζωx0)/ωd·sin(ωd t))
            let decay = (-zeta * omega * dt).exp();
            let damped = omega * (1.0 - zeta * zeta).sqrt();
            let (sin, cos) = (damped * dt).sin_cos();
            let a = x0;
            let b = (v0 + zeta * omega * x0) / damped;
            (
                decay * (a * cos + b * sin),
                decay * ((b * damped - zeta * omega * a) * cos - (a * damped + zeta * omega * b) * sin),
            )
        };
        self.position = self.target + x;
        self.velocity = v;
        if self.settled() {
            self.position = self.target;
            self.velocity = 0.0;
        }
        self.position
    }
}

/// Hermite ease with zero slope at both ends; used for values derived from a
/// spring so that an overshoot never overshoots geometry that must stay exact.
pub fn smooth(p: f64) -> f64 {
    let p = p.clamp(0.0, 1.0);
    p * p * (3.0 - 2.0 * p)
}
/// Fast-out, calm-in ramp used for content that appears while a sheet opens.
pub fn ease_out(p: f64) -> f64 {
    let p = p.clamp(0.0, 1.0);
    1.0 - (1.0 - p).powi(3)
}
/// Interpolate with a clamped fraction.
pub fn lerp(from: f64, to: f64, p: f64) -> f64 {
    from + (to - from) * p.clamp(0.0, 1.0)
}
/// Convert a CSS `<length>` to user units. Percentages resolve against `relative`.
pub fn length(value: &str, relative: f64) -> f64 {
    let value = value.trim();
    if let Some(percent) = value.strip_suffix('%') {
        return percent.parse::<f64>().unwrap_or(0.0) / 100.0 * relative;
    }
    value
        .trim_end_matches("px")
        .parse::<f64>()
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(hz: u32, feel: Feel, seconds: f64) -> Vec<f64> {
        let mut spring = Spring::default();
        spring.to(1.0);
        let mut samples = Vec::new();
        let steps = (hz as f64 * seconds) as usize;
        for _ in 0..steps {
            spring.sample(1.0 / hz as f64, feel);
            samples.push(spring.position);
        }
        samples
    }

    #[test]
    fn refresh_rates_share_the_same_trajectory() {
        for feel in [Feel::DOCK, Feel::SHEET, Feel::REVEAL] {
            let a = run(60, feel, 0.5);
            let b = run(120, feel, 0.5);
            let last = *a.last().unwrap();
            let last_high = *b.last().unwrap();
            assert!((last - last_high).abs() < 0.0015, "{last} vs {last_high}");
        }
    }

    #[test]
    fn dock_spring_overshoots_gently_and_settles() {
        let samples = run(120, Feel::DOCK, 2.0);
        let peak = samples.iter().cloned().fold(f64::MIN, f64::max);
        assert!(peak > 1.0, "expected a visible overshoot, peak {peak}");
        assert!(peak < 1.045, "overshoot must stay subtle, peak {peak}");
        assert!((samples.last().unwrap() - 1.0).abs() < 0.002);
    }

    #[test]
    fn sheet_spring_stays_below_geometry_tolerance() {
        let samples = run(120, Feel::SHEET, 2.0);
        let peak = samples.iter().cloned().fold(f64::MIN, f64::max);
        assert!(peak < 1.004, "sheet progress must not overshoot geometry");
    }

    #[test]
    fn interruption_keeps_position_and_velocity() {
        let mut spring = Spring::default();
        spring.to(1.0);
        for _ in 0..8 {
            spring.sample(1.0 / 120.0, Feel::DOCK);
        }
        let touched = spring.position;
        assert!(touched > 0.0 && touched < 1.0);
        spring.to(0.0);
        let next = spring.sample(1.0 / 120.0, Feel::DOCK);
        // The reversal is felt immediately but starts from the rendered position.
        assert!(next > touched - 0.05);
        for _ in 0..400 {
            spring.sample(1.0 / 120.0, Feel::DOCK);
        }
        assert!(spring.settled());
        assert!(spring.position.abs() < 1e-6);
    }

    #[test]
    fn smooth_and_length_are_bounded() {
        assert_eq!(smooth(-1.0), 0.0);
        assert_eq!(smooth(2.0), 1.0);
        assert_eq!(smooth(0.5), 0.5);
        assert_eq!(ease_out(1.0), 1.0);
        assert_eq!(length("28px", 400.0), 28.0);
        assert_eq!(length("50%", 400.0), 200.0);
        assert_eq!(length("bogus", 400.0), 0.0);
        assert_eq!(lerp(10.0, 20.0, 0.25), 12.5);
    }
}
