import dev.qqlyric.app.ArtworkSeed;
import java.util.*;
public class ArtworkSeedTest{
 static void check(boolean b){if(!b)throw new AssertionError();}
 public static void main(String[] a){int fallback=0xff3482ff;
  check(ArtworkSeed.extract(new int[0],fallback)==fallback);
  check(ArtworkSeed.extract(new int[]{0x00ff0000,0x01ffffff},fallback)==fallback);
  int[] cover=new int[100];Arrays.fill(cover,0xff202020);Arrays.fill(cover,0,45,0xffd83840);
  int seed=ArtworkSeed.extract(cover,fallback);check((seed>>>16&255)>160);check((seed>>>24)==255);
  check(ArtworkSeed.extract(new int[]{0xff808080,0xff808080},fallback)==0xff808080);
  System.out.println("Artwork seed: transparent/empty fallback, dominant chroma, opaque output and monochrome passed.");
 }
}