//! Frame-rate independent spring sampling and shared-container geometry for the
//! player sheet.
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
    pub const PAGER: Feel = Feel {
        stiffness: 22.0,
        damping: 1.0,
    };
}

// ---------------------------------------------------------------------------
// Shared container geometry
// ---------------------------------------------------------------------------
//
// One progress value drives a single rounded rectangle that interpolates between
// the measured song-bar rect and the window rect. Everything the user sees is
// either that rectangle (background + clip), the page laid out once at its final
// size and uniformly scaled into it, or the shared cover.
//
// The page scale is derived from the container *width* only, so the page never
// squashes: while the card is still short the page bottom is simply clipped by
// the card, and the footer controls slide into view with the bottom edge instead
// of being faded or re-positioned. This is the geometry that the previous
// per-section `offsetTop * (sy/sx - 1)` correction was approximating, and it is
// why that correction overlapped the sections.

/// Card opacity finishes early so the growing card is opaque while small.
pub const CARD_REVEAL_END: f64 = 0.22;
/// The song bar hands over to the page content across the slow part of the spring
/// arc. A narrower window made the bar flash out in ~40 ms, which reads as the
/// controls being switched off rather than handed over.
pub const BAR_FADE_START: f64 = 0.02;
pub const BAR_FADE_END: f64 = 0.45;
/// The page content crossfades in while the bar is still visible, so the title
/// band is never empty and two titles are never both fully opaque.
pub const PAGE_FADE_START: f64 = 0.06;
pub const PAGE_FADE_END: f64 = 0.60;

/// One handoff clock for the card, the song bar and the page content.
#[derive(Clone, Copy, Debug)]
pub struct SheetBlend {
    /// Card background/wash opacity.
    pub card: f64,
    /// Song bar (mini player) opacity; also the bottom chrome opacity.
    pub bar: f64,
    /// Full player page content opacity.
    pub page: f64,
    /// The bar must paint above the growing card while it is still visible.
    pub bar_above: bool,
}

pub fn sheet_blend(progress: f64) -> SheetBlend {
    let p = progress.clamp(0.0, 1.0);
    let bar = 1.0 - smooth(((p - BAR_FADE_START) / (BAR_FADE_END - BAR_FADE_START)).clamp(0.0, 1.0));
    SheetBlend {
        card: smooth((p / CARD_REVEAL_END).clamp(0.0, 1.0)),
        bar,
        page: smooth(
            ((p - PAGE_FADE_START) / (PAGE_FADE_END - PAGE_FADE_START)).clamp(0.0, 1.0),
        ),
        bar_above: bar > 0.001,
    }
}

/// The interpolated card rect `[x, y, width, height]` for a progress value.
/// Both endpoints are measured before the transition starts, so this never reads
/// layout mid-frame.
pub fn sheet_rect(from: [f64; 4], to: [f64; 4], progress: f64) -> [f64; 4] {
    let p = progress.clamp(0.0, 1.0);
    [
        lerp(from[0], to[0], p),
        lerp(from[1], to[1], p),
        lerp(from[2], to[2], p),
        lerp(from[3], to[3], p),
    ]
}

/// Uniform page scale for a card width. The page keeps its final layout box and is
/// scaled by this factor around the card's top-left corner.
pub fn sheet_scale(card_width: f64, page_width: f64) -> f64 {
    (card_width / page_width.max(1.0)).clamp(0.02, 4.0)
}

/// A paint threshold is not a lifecycle endpoint. Keep the host throughout the
/// spring tail so a reversal can never animate a hidden full-player surface.
pub fn sheet_mounted(open: bool, progress: f64, settled: bool) -> bool {
    open || !settled || progress > 0.0005
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
                decay * ((b * damped - zeta * omega * a) * cos
                    - (a * damped + zeta * omega * b) * sin),
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
    value.trim_end_matches("px").parse::<f64>().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHEET_FROM: [f64; 4] = [12.0, 758.0, 363.0, 68.0];
    const SHEET_TO: [f64; 4] = [0.0, 0.0, 411.0, 914.0];
    const PAGE: [f64; 2] = [411.0, 914.0];
    /// Settled layout offsets of the page sections inside the 411x914 page box.
    const SECTION_TOPS: [f64; 5] = [0.0, 56.0, 112.0, 806.0, 826.0];
    const SECTION_HEIGHTS: [f64; 5] = [56.0, 56.0, 694.0, 20.0, 88.0];

    #[test]
    fn closing_tail_keeps_host_and_reopen_always_mounts() {
        assert!(sheet_mounted(false, 0.00044, false));
        assert!(sheet_mounted(true, 0.0, true));
        assert!(sheet_mounted(false, 0.0, false));
        assert!(!sheet_mounted(false, 0.0, true));
        for hz in [60, 90, 120, 144] {
            let mut spring = Spring::default();
            spring.place(1.0);
            spring.to(0.0);
            for _ in 0..hz {
                let p = spring.sample(1.0 / hz as f64, Feel::SHEET);
                assert_eq!(sheet_mounted(false, p, spring.settled()), !spring.settled());
            }
        }
    }

    #[test]
    fn shared_container_hits_both_measured_endpoints() {
        assert_eq!(sheet_rect(SHEET_FROM, SHEET_TO, 0.0), SHEET_FROM);
        assert_eq!(sheet_rect(SHEET_FROM, SHEET_TO, 1.0), SHEET_TO);
        assert_eq!(sheet_rect(SHEET_FROM, SHEET_TO, -3.0), SHEET_FROM);
        assert_eq!(sheet_rect(SHEET_FROM, SHEET_TO, 4.0), SHEET_TO);
        let mid = sheet_rect(SHEET_FROM, SHEET_TO, 0.5);
        assert_eq!(mid[2], 387.0);
        assert_eq!(mid[3], 491.0);
        assert!((mid[0] - 6.0).abs() < 1e-9 && (mid[1] - 379.0).abs() < 1e-9);
        // The card starts exactly where the song bar is, so p=0 adds no jump.
        assert_eq!(sheet_scale(SHEET_FROM[2], PAGE[0]), SHEET_FROM[2] / PAGE[0]);
        assert_eq!(sheet_scale(SHEET_TO[2], PAGE[0]), 1.0);
    }

    /// The card grows monotonically and the page scale only ever grows, at every
    /// refresh rate, from the same spring clock.
    #[test]
    fn card_and_scale_are_monotone_at_every_refresh_rate() {
        for hz in [60, 90, 120, 144] {
            let mut spring = Spring::default();
            spring.place(0.0);
            spring.to(1.0);
            let mut previous = sheet_rect(SHEET_FROM, SHEET_TO, 0.0);
            for _ in 0..hz * 2 {
                let p = spring.sample(1.0 / hz as f64, Feel::SHEET);
                let rect = sheet_rect(SHEET_FROM, SHEET_TO, p);
                assert!(rect[2] >= previous[2] - 1e-9, "card width reversed");
                assert!(rect[3] >= previous[3] - 1e-9, "card height reversed");
                assert!(rect[0] <= previous[0] + 1e-9, "card left moved right");
                assert!(rect[1] <= previous[1] + 1e-9, "card top moved down");
                previous = rect;
            }
            assert!((spring.position - 1.0).abs() < 1e-6);
        }
    }

    /// The page keeps its layout: section spacing and height are the settled values
    /// scaled by the page scale, so sections can never overlap or compress.
    #[test]
    fn rigid_page_sections_never_overlap_or_change_spacing() {
        for p in [0.0, 0.05, 0.2, 0.4, 0.5, 0.75, 0.9, 0.99, 1.0] {
            let card = sheet_rect(SHEET_FROM, SHEET_TO, p);
            let scale = sheet_scale(card[2], PAGE[0]);
            let mut previous_bottom = f64::MIN;
            for index in 0..SECTION_TOPS.len() {
                let top = card[1] + SECTION_TOPS[index] * scale;
                let height = SECTION_HEIGHTS[index] * scale;
                if index > 0 {
                    let expected = (SECTION_TOPS[index] - SECTION_TOPS[index - 1]) * scale;
                    let rendered = top - (card[1] + SECTION_TOPS[index - 1] * scale);
                    assert!((rendered - expected).abs() < 1e-9);
                    assert!(
                        top >= previous_bottom - 1e-9,
                        "sections overlap at p={p}: {top} < {previous_bottom}"
                    );
                }
                previous_bottom = top + height;
            }
        }
    }

    /// While closing the page bottom is clipped by the card and only reaches the
    /// window bottom at progress 1, so the footer controls are revealed by the
    /// moving clip instead of by a late fade.
    #[test]
    fn page_is_clipped_by_the_card_until_it_lands() {
        for p in [0.0, 0.25, 0.5, 0.75, 0.9, 0.999] {
            let card = sheet_rect(SHEET_FROM, SHEET_TO, p);
            let scale = sheet_scale(card[2], PAGE[0]);
            let page_bottom = card[1] + PAGE[1] * scale;
            assert!(
                page_bottom >= card[1] + card[3] - 1e-9,
                "the page does not fill the card at p={p}"
            );
        }
        let landed = sheet_rect(SHEET_FROM, SHEET_TO, 1.0);
        assert!((landed[1] + PAGE[1] - (landed[1] + landed[3])).abs() < 1e-9);
    }

    #[test]
    fn handoff_clock_leaves_no_blank_title_band() {
        let mut previous_card = -1.0;
        let mut previous_bar = 2.0;
        let mut previous_page = -1.0;
        for step in 0..=1000 {
            let p = step as f64 / 1000.0;
            let blend = sheet_blend(p);
            assert!(blend.card >= previous_card && blend.card <= 1.0);
            assert!(blend.bar <= previous_bar && blend.bar >= 0.0);
            assert!(blend.page >= previous_page && blend.page <= 1.0);
            let title = blend.bar.max(blend.page);
            assert!(title > 0.25, "no song title visible at p={p} ({title})");
            assert!(
                !(blend.bar > 0.5 && blend.page > 0.5),
                "two opaque song titles at p={p}"
            );
            previous_card = blend.card;
            previous_bar = blend.bar;
            previous_page = blend.page;
        }
        let start = sheet_blend(0.0);
        assert_eq!((start.card, start.bar, start.page), (0.0, 1.0, 0.0));
        let end = sheet_blend(1.0);
        assert_eq!((end.card, end.bar, end.page), (1.0, 0.0, 1.0));
    }

    #[test]
    fn handoff_clock_is_hit_identically_at_every_refresh_rate() {
        for hz in [60, 90, 120, 144] {
            let mut spring = Spring::default();
            spring.to(1.0);
            let mut previous = 0.0;
            for _ in 0..hz {
                let p = spring.sample(1.0 / hz as f64, Feel::SHEET);
                let blend = sheet_blend(p);
                assert!(blend.bar.max(blend.page) > 0.25);
                assert!(p >= previous - 1e-12);
                previous = p;
            }
            assert!((previous - 1.0).abs() < 1e-3);
        }
    }

    #[test]
    fn return_hands_off_foreground_before_the_surface_lands() {
        let tail = sheet_blend(0.1);
        assert!(tail.bar > 0.5 && tail.card <= 0.5 && tail.bar_above);
        let crossing = sheet_blend(0.15);
        assert!(
            crossing.bar > 0.5 && crossing.page > 0.0,
            "the page fades in while the bar is still visible at p=0.15"
        );
        // A handoff narrower than the spring's early arc flashes the bar off.
        let mut widest = 0.0_f64;
        let mut previous = 1.0;
        for step in 0..=2000 {
            let blend = sheet_blend(step as f64 / 2000.0);
            widest = widest.max((previous - blend.bar).abs());
            previous = blend.bar;
        }
        assert!(widest < 0.01, "the bar fade is not smooth in progress");
        for hz in [60, 90, 120, 144] {
            let mut s = Spring::default();
            s.place(1.0);
            s.to(0.0);
            let mut visible = 0;
            for _ in 0..hz {
                let p = s.sample(1.0 / hz as f64, Feel::SHEET);
                let b = sheet_blend(p);
                if p < 0.12 && p > 0.02 {
                    assert!(b.bar > 0.4 && b.card < 0.7);
                    visible += 1;
                }
            }
            assert!(visible > 0);
        }
    }

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
        assert_eq!(length("28px", 400.0), 28.0);
        assert_eq!(length("50%", 400.0), 200.0);
        assert_eq!(length("bogus", 400.0), 0.0);
        assert_eq!(lerp(10.0, 20.0, 0.25), 12.5);
    }
}
