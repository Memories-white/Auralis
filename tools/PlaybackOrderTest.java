import dev.qqlyric.app.PlaybackOrder;
import java.util.*;
public class PlaybackOrderTest {
 static void check(boolean b){if(!b)throw new AssertionError();}
 public static void main(String[] a){
  PlaybackOrder p=new PlaybackOrder(new Random(7));p.reset(3,0);
  check(p.step(1,true)==1);check(p.step(1,true)==2);check(p.step(1,true)==-1);
  p.setMode(PlaybackOrder.Mode.LOOP);check(p.step(1,true)==0);check(p.step(-1,false)==2);
  p.setMode(PlaybackOrder.Mode.SINGLE);check(p.step(1,true)==2);check(p.step(1,false)==0);
  p.reset(8,0);p.setMode(PlaybackOrder.Mode.SHUFFLE);Set<Integer> heard=new HashSet<>();heard.add(0);
  int before=0,last=0;for(int i=0;i<7;i++){before=last;last=p.step(1,true);check(heard.add(last));}
  check(heard.size()==8);check(p.step(-1,false)==before);check(p.step(1,false)==last);check(p.step(1,true)!=last);
  p.reset(1,0);check(p.step(1,true)==0);p.reset(0,-1);check(p.step(1,true)==-1);
  System.out.println("Playback order: sequential end, loop, single, shuffle coverage/history and empty queue passed.");
 }
}
