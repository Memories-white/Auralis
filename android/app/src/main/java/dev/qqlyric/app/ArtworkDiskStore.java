package dev.qqlyric.app;

import java.io.*;
import java.nio.file.*;
import java.util.*;

/** Bounded private cache. Written time is separate from LRU access time. */
public final class ArtworkDiskStore {
    private final File directory;
    private final long maxBytes;
    private final int maxEntries;
    private final int maxEntryBytes;
    public ArtworkDiskStore(File directory, long maxBytes, int maxEntries, int maxEntryBytes) {
        this.directory=directory;this.maxBytes=maxBytes;this.maxEntries=maxEntries;this.maxEntryBytes=maxEntryBytes;
    }
    private File file(String key) {
        if(!key.matches("[a-f0-9]{64}"))throw new IllegalArgumentException("Invalid artwork key");
        return new File(directory,key+".bin");
    }
    public synchronized byte[] read(String key,long now,long maxAge) {
        File entry=file(key);
        if(entry.length()<=8 || entry.length()>maxEntryBytes+8L)return null;
        try(DataInputStream input=new DataInputStream(new FileInputStream(entry))){
            long written=input.readLong();
            if(now-written>maxAge || written>now+60000)return null;
            byte[] bytes=new byte[(int)entry.length()-8];input.readFully(bytes);
            entry.setLastModified(now);
            return bytes;
        }catch(IOException e){return null;}
    }
    public synchronized boolean write(String key,byte[] bytes,long now) {
        if(bytes.length==0 || bytes.length>maxEntryBytes || bytes.length+8L>maxBytes)return false;
        File destination=file(key);
        File temporary=new File(directory,key+".tmp");
        try{
            if(!directory.isDirectory() && !directory.mkdirs())return false;
            try(DataOutputStream output=new DataOutputStream(new FileOutputStream(temporary))){output.writeLong(now);output.write(bytes);}
            try{Files.move(temporary.toPath(),destination.toPath(),StandardCopyOption.REPLACE_EXISTING,StandardCopyOption.ATOMIC_MOVE);}
            catch(AtomicMoveNotSupportedException e){Files.move(temporary.toPath(),destination.toPath(),StandardCopyOption.REPLACE_EXISTING);}
            destination.setLastModified(now);trim();return true;
        }catch(IOException e){temporary.delete();return false;}
    }
    private void trim(){
        File[] entries=directory.listFiles((dir,name)->name.matches("[a-f0-9]{64}\\.bin"));
        if(entries==null)return;
        Arrays.sort(entries,Comparator.comparingLong(File::lastModified));
        long total=0;for(File entry:entries)total+=entry.length();
        int count=entries.length;
        for(File entry:entries){
            if(total<=maxBytes && count<=maxEntries)break;
            long size=entry.length();if(entry.delete()){total-=size;count--;}
        }
    }
}
