package dev.qqlyric.app;
/** Small bounded histogram of decoded cover pixels, never of account/network data. */
public final class ArtworkSeed {
 public static int extract(int[] pixels,int fallback){
  int[] counts=new int[32768];long[] red=new long[32768],green=new long[32768],blue=new long[32768];
  for(int p:pixels){if((p>>>24)<128)continue;int r=p>>>16&255,g=p>>>8&255,b=p&255;int k=(r>>3)<<10|(g>>3)<<5|(b>>3);counts[k]++;red[k]+=r;green[k]+=g;blue[k]+=b;}
  double best=-1;int result=fallback;
  for(int k=0;k<counts.length;k++)if(counts[k]>0){int n=counts[k],r=(int)(red[k]/n),g=(int)(green[k]/n),b=(int)(blue[k]/n);int max=Math.max(r,Math.max(g,b)),min=Math.min(r,Math.min(g,b));double saturation=max==0?0:(double)(max-min)/max;double score=n*(0.15+saturation)*(max<32||min>238?0.08:1);
   if(score>best){best=score;result=0xff000000|r<<16|g<<8|b;}
  }
  return result;
 }
}
