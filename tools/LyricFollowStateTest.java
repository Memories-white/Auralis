import dev.qqlyric.app.LyricFollowState;
public final class LyricFollowStateTest {
    private static void check(boolean value) { if (!value) throw new AssertionError(); }
    public static void main(String[] args) {
        LyricFollowState state = new LyricFollowState();
        check(state.requestScroll(0, true, 400));
        check(!state.requestScroll(10, true, 400));
        state.touchDown();
        check(!state.requestScroll(100, true, 500));
        state.touchEnd(100);
        check(!state.requestScroll(1899, true, 400));
        // Resume the SAME line after dragging; no line-change event is necessary.
        check(state.requestScroll(1900, true, 400));
        check(!state.requestScroll(2000, false, 600));
        state.resume();
        check(state.requestScroll(2100, true, 400));
        state.layoutChanged();
        check(state.requestScroll(2200, true, 400));
        System.out.println("Lyric follow regression checks passed.");
    }
}
