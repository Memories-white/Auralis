package dev.qqlyric.app;

import java.util.*;

/** Queue traversal is independent of decoding, network and UI lifetimes. */
public final class PlaybackOrder {
    public enum Mode { ORDER, LOOP, SINGLE, SHUFFLE }
    private final Random random;
    private Mode mode=Mode.ORDER;
    private int size,index=-1,cursor;
    private final ArrayList<Integer> bag=new ArrayList<>(),history=new ArrayList<>();
    public PlaybackOrder(){this(new Random());}
    public PlaybackOrder(Random random){this.random=random;}
    public void reset(int size,int index){this.size=Math.max(0,size);this.index=index;bag.clear();history.clear();if(index>=0&&index<size)history.add(index);cursor=0;}
    public void setMode(Mode mode){if(this.mode!=mode){this.mode=mode;reset(size,index);}}
    public int step(int direction,boolean automatic){
        if(size==0||index<0||index>=size)return -1;
        if(automatic&&mode==Mode.SINGLE)return index;
        if(mode==Mode.SHUFFLE){
            if(direction<0){if(cursor>0)index=history.get(--cursor);return index;}
            if(cursor+1<history.size()){index=history.get(++cursor);return index;}
            if(size==1)return index;
            if(bag.isEmpty()){for(int i=0;i<size;i++)if(i!=index)bag.add(i);Collections.shuffle(bag,random);}
            index=bag.remove(bag.size()-1);history.add(index);cursor=history.size()-1;
            if(history.size()>512){history.remove(0);cursor--;}
            return index;
        }
        int next=index+(direction<0?-1:1);
        if(next<0||next>=size){if(mode==Mode.ORDER)return -1;next=(next+size)%size;}
        index=next;return index;
    }
}
