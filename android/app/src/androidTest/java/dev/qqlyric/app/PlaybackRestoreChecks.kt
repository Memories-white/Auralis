package dev.qqlyric.app

import android.app.Activity
import android.app.Instrumentation
import android.content.Intent
import android.os.Bundle
import android.os.SystemClock
import org.json.JSONObject

/** Exercises persisted state and the real service without fetching or starting media. */
internal fun Instrumentation.runPlaybackRestoreChecks(){
    val context=targetContext
    val prefs=context.getSharedPreferences(PlaybackStore.PREFS,0)
    val original=prefs.getString(PlaybackStore.KEY,null)
    var touched=false
    var model:MusicState?=null
    val report=Bundle()
    var result=Activity.RESULT_CANCELED
    fun command(action:String,configure:Intent.()->Unit={})=runOnMainSync{
        context.startService(Intent(context,PlayerService::class.java).setAction(action).apply(configure))
    }
    fun await(label:String,condition:()->Boolean){
        val deadline=SystemClock.elapsedRealtime()+5000
        while(!condition()){check(SystemClock.elapsedRealtime()<deadline){label};Thread.sleep(50)}
    }
    try{
        check(!PlayerService.snapshot.playing&&PlayerService.snapshot.sleepRemainingSeconds==0&&!PlayerService.snapshot.sleepAfterTrack){"Pause existing playback and cancel its timer first"}
        touched=true
        val first=JSONObject().put("mid","restore-first").put("title","First").put("duration",120)
        val second=JSONObject().put("mid","restore-second").put("title","Second").put("artist","Local check").put("duration",120)
            .put("url","https://example.invalid/signed-audio").put("session",JSONObject().put("musicKey","fixture-only"))
        PlaybackStore.save(context,second,listOf(first,second),1,45000,120000,true)
        check(!prefs.getString(PlaybackStore.KEY,"").orEmpty().contains("signed-audio"))
        check(!prefs.getString(PlaybackStore.KEY,"").orEmpty().contains("musicKey"))
        val saved=checkNotNull(PlaybackStore.read(context))
        check(saved.index==1&&saved.position==45000&&saved.song.optString("mid")=="restore-second")
        runOnMainSync{model=MusicState(context);model!!.sample()}
        check(model!!.title=="Second"&&!model!!.playing&&model!!.position==45000&&model!!.currentIndex==1)
        command("PAUSE")
        await("Service did not restore paused metadata"){PlayerService.snapshot.mid=="restore-second"&&!PlayerService.snapshot.playing&&PlayerService.snapshot.position==45000}
        command("SEEK"){putExtra("position",67000)}
        await("Restored seek was not saved"){PlaybackStore.read(context)?.position==67000}
        command("STOP")
        await("Service did not stop"){PlayerService.snapshot.mid.isBlank()}
        runOnMainSync{model!!.sample()}
        check(model!!.position==67000&&model!!.mid=="restore-second"&&!model!!.playing)
        val large=(0..699).map{JSONObject().put("mid","item-$it").put("duration",10)}
        PlaybackStore.save(context,large[650],large,650,99999,10000,true)
        val bounded=checkNotNull(PlaybackStore.read(context))
        check(bounded.queue.size==500&&bounded.queue[bounded.index].optString("mid")=="item-650"&&bounded.position==10000)
        prefs.edit().putString(PlaybackStore.KEY,"{invalid").commit()
        check(PlaybackStore.read(context)==null)
        prefs.edit().putString(PlaybackStore.KEY,"{\"version\":999}").commit()
        check(PlaybackStore.read(context)==null)
        report.putString("stream","PASS: metadata-only persistence, paused cold restore, queue identity, seek, stop/reopen, bounds and malformed state.\n")
        result=Activity.RESULT_OK
    }catch(error:Throwable){report.putString("stream","Restore checks failed: ${error.message}\n")}
    finally{
        runOnMainSync{model?.close()}
        if(touched){
            command("STOP")
            runOnMainSync{prefs.edit().apply{if(original==null)remove(PlaybackStore.KEY)else putString(PlaybackStore.KEY,original)}.commit()}
        }
        finish(result,report)
    }
}
