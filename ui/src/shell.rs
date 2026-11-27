//! Rust shell metrics mapped from Miuix beta01; no second navigation owner.
pub fn wide(width: f64, height: f64) -> bool {
    width >= 840.0 || width >= 600.0 && height / width.max(1.0) < 1.2
}
pub fn floating_bottom(safe: f64) -> f64 {
    if safe <= 16.0 {
        32.0
    } else if safe >= 32.0 {
        safe
    } else {
        safe + 12.0
    }
}
pub struct DockMetrics {
    pub clearance: f64,
    pub extra: f64,
}
pub fn dock_visible(page: &str) -> bool {
    !page.starts_with("monet")
}
/// Actual mini + navigation + system inset, with 16px of readable list breathing
/// room. Floating padding and the system navigation inset are different owners.
pub fn dock_metrics(wide: bool, root: bool, floating: bool, safe: f64) -> DockMetrics {
    let safe = safe.max(0.0);
    let (mini, gap, extra) = if floating {
        (64.0, 8.0, 138.0)
    } else {
        (68.0, 6.0, 134.0)
    };
    let clearance = if wide {
        mini + safe + 12.0 + 16.0
    } else {
        mini + gap
            + if root { 64.0 } else { 0.0 }
            + if floating {
                floating_bottom(safe)
            } else {
                safe
            }
            + 16.0
    };
    DockMetrics { clearance, extra }
}
pub fn settings(page: &str) -> bool {
    matches!(
        page,
        "settings"
            | "appearance"
            | "quality"
            | "monet"
            | "monet-source"
            | "monet-style"
            | "monet-spec"
            | "storage"
            | "about"
            | "licenses"
            | "ui-customize"
            | "ui-dock"
            | "ui-player"
            | "ui-list"
            | "ui-motion"
            | "lyrics-style"
            | "lyric-type"
            | "lyric-effect"
    )
}
pub fn collapse(scroll: f64) -> f64 {
    (scroll / 56.0).clamp(0.0, 1.0)
}
pub fn project_link(key: &str) -> Option<&'static str> {
    match key {
        "project" => Some("https://github.com/Memories-white/Auralis"),
        "releases" => Some("https://github.com/Memories-white/Auralis/releases"),
        "issues" => Some("https://github.com/Memories-white/Auralis/issues"),
        _ => None,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shell_geometry_and_profile_boundaries() {
        assert_eq!(dock_metrics(false, false, true, 0.0).clearance, 120.0);
        assert_eq!(dock_metrics(false, true, false, 20.0).clearance, 174.0);
        assert_eq!(dock_metrics(true, true, true, 20.0).clearance, 112.0);
        assert_eq!(dock_metrics(false, true, true, 20.0).extra, 138.0);
        assert!(!dock_visible("monet-style"));
        assert!(dock_visible("songs"));
        assert!(project_link("project")
            .unwrap()
            .starts_with("https://github.com/"));
        assert!(project_link("javascript:alert(1)").is_none());
        assert!(!wide(411.0, 914.0));
        assert!(wide(900.0, 650.0));
        assert!(wide(700.0, 500.0));
        assert!(!wide(700.0, 1000.0));
        assert!(!wide(0.0, 0.0));
        assert_eq!(floating_bottom(16.0), 32.0);
        assert_eq!(floating_bottom(24.0), 36.0);
        assert_eq!(floating_bottom(40.0), 40.0);
        assert_eq!(collapse(28.0), 0.5);
        assert_eq!(collapse(1000.0), 1.0);
        assert!(!settings("home"));
        assert!(settings("licenses"));
    }
}
