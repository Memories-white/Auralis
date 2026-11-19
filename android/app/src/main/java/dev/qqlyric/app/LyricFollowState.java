package dev.qqlyric.app;

/** Touch suspension is independent from the currently highlighted lyric line. */
public final class LyricFollowState {
    private boolean touching;
    private long resumeAt;
    private Integer lastTarget;
    public void touchDown() { touching = true; lastTarget = null; }
    public void touchEnd(long now) { touching = false; resumeAt = now + 1800; lastTarget = null; }
    public void layoutChanged() { lastTarget = null; }
    public void resume() { touching = false; resumeAt = 0; lastTarget = null; }
    public boolean requestScroll(long now, boolean visible, int target) {
        if (!visible || touching || now < resumeAt || (lastTarget != null && lastTarget == target)) return false;
        lastTarget = target;
        return true;
    }
}
