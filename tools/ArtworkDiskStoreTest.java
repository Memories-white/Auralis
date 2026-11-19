import dev.qqlyric.app.ArtworkDiskStore;
import java.io.*;
import java.nio.file.*;
import java.util.*;
public class ArtworkDiskStoreTest {
    static String key(int n){return String.format("%064x",n);}
    static void check(boolean ok,String message){if(!ok)throw new AssertionError(message);}
    public static void main(String[] args)throws Exception{
        File directory=Files.createTempDirectory("auralis-artwork-test").toFile();
        try{
            ArtworkDiskStore store=new ArtworkDiskStore(directory,50,2,32);
            byte[] image=new byte[12];Arrays.fill(image,(byte)7);
            check(store.write(key(1),image,100),"write");
            check(Arrays.equals(store.read(key(1),110,50),image),"round trip");
            check(store.read(key(1),151,50)==null,"access must not extend freshness");
            check(store.read(key(1),151,Long.MAX_VALUE)!=null,"offline stale fallback");
            store.write(key(2),image,160);store.read(key(1),170,1000);store.write(key(3),image,180);
            check(store.read(key(2),181,1000)==null,"evict least recently used");
            check(store.read(key(1),181,1000)!=null,"retain recently accessed image");
            check(!store.write(key(4),new byte[33],190),"single image cap");
            boolean rejected=false;try{store.read("../escape",200,1000);}catch(IllegalArgumentException expected){rejected=true;}
            check(rejected,"reject traversal");
            check(Arrays.stream(directory.listFiles()).mapToLong(File::length).sum()<=50,"total disk cap");
            System.out.println("Artwork cache round-trip, freshness, stale fallback, LRU, size and key checks passed.");
        }finally{for(File file:directory.listFiles())file.delete();directory.delete();}
    }
}
