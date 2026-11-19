package dev.qqlyric.app

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.os.Build
import android.util.LruCache
import java.io.ByteArrayOutputStream
import java.net.URL
import java.security.MessageDigest

/** Public artwork only. Call load on a worker; peek never performs disk IO. */
object ArtworkCache {
    private data class Entry(val bitmap:Bitmap,val written:Long)
    private val memory=object:LruCache<String,Entry>(8*1024*1024){override fun sizeOf(key:String,value:Entry)=value.bitmap.byteCount}
    private val locks=Array(16){Any()}
    @Volatile private var disk:ArtworkDiskStore?=null
    private fun store(context:Context)=disk ?: synchronized(this){disk ?: ArtworkDiskStore(java.io.File(context.applicationContext.cacheDir,"artwork-v1"),24L*1024*1024,240,256*1024).also{disk=it}}
    private fun address(url:String,avatar:Boolean):String {
        val secure=url.replace("http://","https://")
        return if(avatar)secure.replace(Regex("([?&]s=)640")){it.groupValues[1]+"160"}
        else secure.replace(Regex("/T002R\\d+x\\d+M000"),"/T002R500x500M000")
    }
    private fun key(url:String)=MessageDigest.getInstance("SHA-256").digest(url.toByteArray()).joinToString(""){"%02x".format(it)}
    private fun age(avatar:Boolean)=if(avatar)24L*60*60*1000 else 7L*24*60*60*1000
    fun peek(url:String,avatar:Boolean=false):Bitmap? {
        val entry=memory.get(key(address(url,avatar))) ?: return null
        return entry.bitmap.takeIf{System.currentTimeMillis()-entry.written<age(avatar)}
    }
    fun load(context:Context,url:String,avatar:Boolean=false):Bitmap? {
        val address=address(url,avatar)
        if(!address.startsWith("https://"))return null
        val key=key(address)
        synchronized(locks[(key.hashCode() and Int.MAX_VALUE)%locks.size]){
            peek(address,avatar)?.let{return it}
            val now=System.currentTimeMillis();val store=store(context)
            store.read(key,now,age(avatar))?.let{bytes->
                BitmapFactory.decodeByteArray(bytes,0,bytes.size)?.let{bitmap->memory.put(key,Entry(bitmap,now));return bitmap}
            }
            val bitmap=runCatching{
                val connection=URL(address).openConnection().apply{connectTimeout=5000;readTimeout=5000;setRequestProperty("User-Agent","Auralis")}
                try{
                    require(connection.contentLengthLong<=8L*1024*1024)
                    val bytes=connection.getInputStream().use{input->
                        val buffer=ByteArray(8192);val output=ByteArrayOutputStream()
                        while(true){val size=input.read(buffer);if(size<0)break;require(output.size()+size<=8*1024*1024);output.write(buffer,0,size)}
                        output.toByteArray()
                    }
                    val bounds=BitmapFactory.Options().apply{inJustDecodeBounds=true}
                    BitmapFactory.decodeByteArray(bytes,0,bytes.size,bounds)
                    require(bounds.outWidth in 1..8192 && bounds.outHeight in 1..8192)
                    val edge=if(avatar)160 else 512
                    var sample=1
                    while(maxOf(bounds.outWidth,bounds.outHeight)/sample>edge*2)sample*=2
                    val decoded=BitmapFactory.decodeByteArray(bytes,0,bytes.size,BitmapFactory.Options().apply{inSampleSize=sample}) ?: return@runCatching null
                    val ratio=minOf(1f,edge.toFloat()/maxOf(decoded.width,decoded.height))
                    val scaled=if(ratio<1)Bitmap.createScaledBitmap(decoded,(decoded.width*ratio).toInt().coerceAtLeast(1),(decoded.height*ratio).toInt().coerceAtLeast(1),true)else decoded
                    if(scaled!==decoded)decoded.recycle()
                    scaled
                }finally{(connection as? java.net.HttpURLConnection)?.disconnect()}
            }.getOrNull()
            if(bitmap!=null){
                val output=ByteArrayOutputStream()
                @Suppress("DEPRECATION") val format=if(Build.VERSION.SDK_INT>=30)Bitmap.CompressFormat.WEBP_LOSSY else Bitmap.CompressFormat.WEBP
                if(bitmap.compress(format,76,output))store.write(key,output.toByteArray(),now)
                memory.put(key,Entry(bitmap,now));return bitmap
            }
            // Temporary network failures can still display a previously cached image.
            return store.read(key,now,Long.MAX_VALUE)?.let{bytes->BitmapFactory.decodeByteArray(bytes,0,bytes.size)}
        }
    }
}
