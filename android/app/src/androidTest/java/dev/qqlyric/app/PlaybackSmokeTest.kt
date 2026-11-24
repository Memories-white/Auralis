package dev.qqlyric.app

import android.app.Activity
import android.app.Instrumentation
import android.content.Intent
import android.os.Bundle
import android.os.SystemClock
import org.json.JSONArray
import org.json.JSONObject
import java.io.File
import java.nio.ByteBuffer
import java.nio.ByteOrder

/** Device-only integration checks with generated silence; no network audio or account access. */
class PlaybackSmokeTest : Instrumentation() {
    private var holdSeconds=0
    override fun onCreate(arguments:Bundle?) {
        super.onCreate(arguments)
        holdSeconds=arguments?.getString("holdSeconds")?.toIntOrNull()?.coerceIn(0,90)?:0
        start()
    }

    private fun command(action:String,configure:Intent.()->Unit={})=runOnMainSync {
        targetContext.startService(Intent(targetContext,PlayerService::class.java).setAction(action).apply(configure))
    }
    private fun await(label:String,timeout:Long=8000,condition:()->Boolean) {
        val end=SystemClock.elapsedRealtime()+timeout
        while(!condition()) {
            check(SystemClock.elapsedRealtime()<end){"Timed out: $label"}
            Thread.sleep(50)
        }
    }
    private fun status(message:String) { sendStatus(0,Bundle().apply{putString("stream","$message\n")}) }

    override fun onStart() {
        val context=targetContext
        val prefs=context.getSharedPreferences("library",0)
        val originalHistory=prefs.getString(ListeningHistory.KEY,null)
        val originalMode=prefs.getString("playback_mode","ORDER")?:"ORDER"
        val audio=File(context.filesDir,"playback-smoke.wav")
        var result=Activity.RESULT_CANCELED
        var touched=false
        val report=Bundle()
        try {
            check(!PlayerService.snapshot.playing){"Pause existing playback before running this check"}
            check(PlayerService.snapshot.sleepRemainingSeconds==0&&!PlayerService.snapshot.sleepAfterTrack){"Cancel an existing sleep timer before running this check"}
            touched=true
            val samples=8000*120
            val header=ByteBuffer.allocate(44).order(ByteOrder.LITTLE_ENDIAN).apply {
                put("RIFF".toByteArray());putInt(36+samples*2);put("WAVEfmt ".toByteArray())
                putInt(16);putShort(1);putShort(1);putInt(8000);putInt(16000);putShort(2);putShort(16)
                put("data".toByteArray());putInt(samples*2)
            }.array()
            audio.outputStream().use{it.write(header);it.write(ByteArray(samples*2))}
            // Compose's continuously animated page may never satisfy startActivitySync's idle wait.
            context.startActivity(Intent(context,MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK).putExtra("open_player",true))
            fun play(index:Int) {
                val song=JSONObject().put("mid","auralis-smoke-$index").put("title",if(index==1)"本地验证音频"else"最近播放验证音频").put("artist","Auralis UI Test").put("duration",120)
                command("PLAY") {
                    putExtra("url",audio.toURI().toString());putExtra("song",song.toString())
                    putExtra("queue",JSONArray().put(song).toString());putExtra("index",0)
                    putExtra("token","smoke-$index-${SystemClock.elapsedRealtime()}")
                    putExtra("lyric","[00:00.00]本地静音验证音频\n[00:05.00]检查歌词与播放位置\n[00:10.00]检查最近播放与睡眠定时\n[00:15.00]此音频不包含第三方内容")
                }
                await("local playback"){PlayerService.snapshot.mid==song.getString("mid")&&PlayerService.snapshot.playing}
            }
            play(1)
            await("history recorded"){ListeningHistory.read(context).firstOrNull()?.optString("mid")=="auralis-smoke-1"}
            val first=ListeningHistory.read(context).first()
            check(!first.has("url")&&!first.has("session"))
            command("PAUSE");command("RESUME");Thread.sleep(300)
            check(ListeningHistory.read(context).first().optLong("lastPlayedAt")==first.optLong("lastPlayedAt")){"Resume duplicated history"}
            play(2)
            await("two history records"){ListeningHistory.read(context).take(2).map{it.optString("mid")}==listOf("auralis-smoke-2","auralis-smoke-1")}
            command("MODE"){putExtra("mode","LOOP")}
            command("SLEEP"){putExtra("minutes",-1)}
            command("SEEK"){putExtra("position",119200)}
            await("stop at track end",6000){PlayerService.snapshot.sleepFinished&&!PlayerService.snapshot.playing}
            check(PlayerService.snapshot.mid=="auralis-smoke-2"&&PlayerService.snapshot.position>118000){"Track looped after sleep"}
            status("PASS: successful-start history, resume dedupe, track-end sleep before loop")
            play(1)
            command("SLEEP"){putExtra("minutes",1)}
            await("deadline armed"){PlayerService.snapshot.sleepRemainingSeconds in 58..60}
            Thread.sleep(2200)
            check(PlayerService.snapshot.sleepRemainingSeconds in 55..58){"Countdown did not advance"}
            status("Checking the real service deadline for one minute")
            await("deadline pause",62000){PlayerService.snapshot.sleepFinished&&!PlayerService.snapshot.playing}
            status("PASS: real timer deadline paused playback")
            if(holdSeconds>0) {
                play(2);command("PAUSE");command("SLEEP"){putExtra("minutes",15)}
                status("UI READY: generated local audio, paused, timer armed")
                Thread.sleep(holdSeconds*1000L)
            }
            report.putString("stream","Playback integration checks passed.\n")
            result=Activity.RESULT_OK
        } catch(error:Throwable) {
            report.putString("stream","Playback integration check failed: ${error.message}\n")
        } finally {
            if(touched) {
                command("STOP")
                runOnMainSync {
                    prefs.edit().apply{if(originalHistory==null)remove(ListeningHistory.KEY)else putString(ListeningHistory.KEY,originalHistory)}.putString("playback_mode",originalMode).commit()
                }
            }
            audio.delete()
            finish(result,report)
        }
    }
}
