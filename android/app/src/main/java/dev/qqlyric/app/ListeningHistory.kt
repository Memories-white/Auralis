package dev.qqlyric.app

import android.content.Context
import org.json.JSONArray
import org.json.JSONObject

/** Only reusable song metadata is persisted, never credentials or signed media URLs. */
object ListeningHistory {
    const val KEY="recent_plays"
    const val LIMIT=100
    private val keys=listOf("mid","id","title","artist","album","albumMid","mediaMid","duration","vip","coverUrl")
    fun read(context:Context):List<JSONObject> = runCatching {
        JSONArray(context.getSharedPreferences("library",Context.MODE_PRIVATE).getString(KEY,"[]"))
            .let{a->List(a.length()){a.optJSONObject(it)}.filterNotNull().filter{it.optString("mid").isNotBlank()}.take(LIMIT)}
    }.getOrDefault(emptyList())
    fun record(context:Context,song:JSONObject){
        if(song.optString("mid").isBlank())return
        val item=JSONObject()
        keys.forEach{key->if(song.has(key))item.put(key,song.get(key))}
        item.put("lastPlayedAt",System.currentTimeMillis())
        val songs=(listOf(item)+read(context).filterNot{it.optString("mid")==song.optString("mid")}).take(LIMIT)
        context.getSharedPreferences("library",Context.MODE_PRIVATE).edit().putString(KEY,JSONArray(songs).toString()).apply()
    }
    fun clear(context:Context){context.getSharedPreferences("library",Context.MODE_PRIVATE).edit().remove(KEY).apply()}
}
