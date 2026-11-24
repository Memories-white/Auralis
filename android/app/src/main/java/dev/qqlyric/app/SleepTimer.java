package dev.qqlyric.app;

/** Monotonic playback timer. The service supplies elapsedRealtime, including sleep. */
public final class SleepTimer {
    private long deadline;
    private boolean afterTrack;
    private int minutes;
    public void setMinutes(int minutes, long now) {
        if (minutes < 1 || minutes > 90) throw new IllegalArgumentException("Timer must be 1..90 minutes");
        long duration = minutes * 60_000L;
        deadline = Math.addExact(now, duration);
        this.minutes = minutes;
        afterTrack = false;
    }
    public void setAfterTrack() { deadline = 0; minutes = 0; afterTrack = true; }
    public void clear() { deadline = 0; minutes = 0; afterTrack = false; }
    public int getMinutes() { return minutes; }
    public boolean isAfterTrack() { return afterTrack; }
    public long remaining(long now) { return deadline == 0 ? 0 : Math.max(0, deadline - now); }
    public boolean consumeDeadline(long now) {
        if (deadline == 0 || now < deadline) return false;
        clear(); return true;
    }
    public boolean consumeTrackEnd() {
        if (!afterTrack) return false;
        clear(); return true;
    }
}
