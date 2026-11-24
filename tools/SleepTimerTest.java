import dev.qqlyric.app.SleepTimer;
public class SleepTimerTest {
    static void check(boolean value){if(!value)throw new AssertionError();}
    public static void main(String[] args){
        SleepTimer timer=new SleepTimer();
        timer.setMinutes(15,1000);
        check(timer.remaining(2000)==899000);
        check(!timer.consumeDeadline(900999));
        check(timer.consumeDeadline(901000));
        check(!timer.consumeDeadline(901001));
        timer.setMinutes(30,10);timer.setAfterTrack();
        check(timer.isAfterTrack()&&timer.remaining(10)==0);
        check(!timer.consumeDeadline(Long.MAX_VALUE));
        check(timer.consumeTrackEnd()&&!timer.consumeTrackEnd());
        timer.setAfterTrack();timer.setMinutes(1,20);
        check(!timer.consumeTrackEnd());
        timer.clear();check(!timer.consumeDeadline(Long.MAX_VALUE));
        for(int invalid:new int[]{0,-1,91,Integer.MAX_VALUE}){
            try{timer.setMinutes(invalid,0);throw new AssertionError();}catch(IllegalArgumentException expected){}
        }
        System.out.println("Sleep timer: exact deadline, elapsed time, track end, replacement, cancellation and bounds passed.");
    }
}
