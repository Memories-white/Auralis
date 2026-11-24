package dev.qqlyric.app

import android.content.Context
import org.json.JSONArray
import org.json.JSONObject

internal data class SavedPlayback(val song:JSONObject,val queue:List<JSONObject>,val index:Int,val position:Int,val duration:Int)

/** Restorable metadata only. Signed media URLs and account credentials are never persisted here. */
internal object PlaybackStore {
    const val PREFS="playback"
    const val KEY="last_playback"
    private const val MAX_QUEUE=500

    private fun snapshot(song:JSONObject,queue:List<JSONObject>,index:Int,position:Int,duration:Int):SavedPlayback? {
        val mid=song.optString("mid")
        if(mid.isBlank())return null
        val clean=queue.filter{it.optString("mid").isNotBlank()}.map{it.songMetadata()}
        val current=if(clean.getOrNull(index)?.optString("mid")==mid)index else clean.indexOfFirst{it.optString("mid")==mid}
        val selected=song.songMetadata()
        val items=if(current<0)listOf(selected)else clean
        val selectedIndex=if(current<0)0 else current
        val start=(selectedIndex-MAX_QUEUE/2).coerceAtLeast(0).coerceAtMost((items.size-MAX_QUEUE).coerceAtLeast(0))
        val bounded=items.drop(start).take(MAX_QUEUE)
        val length=duration.coerceAtLeast(0)
        return SavedPlayback(selected,bounded,selectedIndex-start,position.coerceIn(0,length),length)
    }

    fun save(context:Context,song:JSONObject,queue:List<JSONObject>,index:Int,position:Int,duration:Int,sync:Boolean=false) {
        val saved=snapshot(song,queue,index,position,duration)?:return
        val json=JSONObject().put("version",1).put("song",saved.song).put("queue",JSONArray(saved.queue))
            .put("index",saved.index).put("position",saved.position).put("duration",saved.duration)
        val edit=context.getSharedPreferences(PREFS,Context.MODE_PRIVATE).edit().putString(KEY,json.toString())
        if(sync)edit.commit()else edit.apply()
    }

    fun read(context:Context):SavedPlayback?=runCatching {
        val raw=context.getSharedPreferences(PREFS,Context.MODE_PRIVATE).getString(KEY,null)
        if(raw==null) {
            // Older builds retained recent metadata but no resume position.
            val last=ListeningHistory.read(context).firstOrNull()?:return@runCatching null
            val library=context.getSharedPreferences("library",Context.MODE_PRIVATE)
            val old=runCatching{JSONArray(library.getString("active_queue","[]")).objects()}.getOrDefault(emptyList())
            val length=(last.optLong("duration").coerceIn(0,Int.MAX_VALUE/1000L)*1000).toInt()
            snapshot(last,old,-1,0,length)?.also{save(context,it.song,it.queue,it.index,it.position,it.duration)}
        }else{
            val value=JSONObject(raw)
            if(value.optInt("version")!=1)return@runCatching null
            val song=value.optJSONObject("song")?:return@runCatching null
            val queue=value.optJSONArray("queue")?.objects().orEmpty()
            snapshot(song,queue,value.optInt("index",-1),value.optLong("position").coerceIn(0,Int.MAX_VALUE.toLong()).toInt(),
                value.optLong("duration").coerceIn(0,Int.MAX_VALUE.toLong()).toInt())
        }
    }.getOrNull()
}
