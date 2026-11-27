package dev.qqlyric.app

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.media.AudioAttributes
import android.media.AudioManager
import android.media.MediaMetadata
import android.media.MediaPlayer
import android.media.session.MediaSession
import android.media.session.PlaybackState
import android.os.Build
import android.os.IBinder
import android.os.SystemClock
import android.os.Handler
import android.os.Looper
import org.json.JSONArray
import org.json.JSONObject
import java.util.concurrent.Executors

data class PlaybackSnapshot(
    val mid:String="", val title:String="", val artist:String="", val artwork:String="",
    val playing:Boolean=false, val position:Int=0, val duration:Int=0,
    val lyric:String="", val translation:String="", val error:String="", val token:String="",val quality:String="",val qualityPending:Boolean=false,val downgraded:Boolean=false,
    val sleepRemainingSeconds:Int=0,val sleepAfterTrack:Boolean=false,val sleepFinished:Boolean=false,val sleepMinutes:Int=0,
    val queueJson:String="[]",val index:Int=-1,val wordLyrics:String="[]"
)

class PlayerService : Service() {
    companion object {
        @Volatile var snapshot=PlaybackSnapshot()
            private set
        @Volatile var positionMs = 0
        @Volatile var durationMs = 0
        @Volatile var isPlaying = false
        @Volatile var currentTitle = ""
        @Volatile var currentArtist = ""
        @Volatile var currentLyric = ""
        @Volatile var currentTranslation = ""
        @Volatile var currentIndex = -1
        @Volatile var playbackError = ""
        @Volatile var currentArtwork = ""
    }
    private val worker = Executors.newSingleThreadExecutor()
    private val ui = Handler(Looper.getMainLooper())
    private val pulse = object : Runnable {
        override fun run() {
            expireSleepTimer()
            if (playing) { positionMs = runCatching { player?.currentPosition }.getOrNull() ?: positionMs;updateSnapshot() }
            else updateSnapshot()
            if(playing)persistPlayback()
            ui.postDelayed(this, if(playing)50 else 250)
        }
    }
    private var player: MediaPlayer? = null
    private lateinit var session: MediaSession
    private var song = JSONObject()
    private var lyric = ""
    private var translation = ""
    private var wordLyrics = "[]"
    private var playing = false
    private var prepared = false
    private var loading = false
    private var wantPlay = true
    private var artwork:android.graphics.Bitmap?=null
    private var queue = JSONArray()
    private var queueJson="[]"
    private var account = JSONObject()
    private val order=PlaybackOrder()
    private var generation=0
    private var playToken=""
    private var actualQuality=""
    private var qualityPending=false
    private var downgraded=false
    private var skipPending=false
    private val sleepTimer=SleepTimer()
    private var sleepFinished=false
    private var historyToken=""
    private var selectionPending=false
    private var selectionInterruptedBySleep=false
    private var lastSave=0L
    private var foreground=false

    override fun onCreate() {
        super.onCreate()
        order.setMode(runCatching{PlaybackOrder.Mode.valueOf(getSharedPreferences("library",MODE_PRIVATE).getString("playback_mode","ORDER")?:"ORDER")}.getOrDefault(PlaybackOrder.Mode.ORDER))
        getSystemService(NotificationManager::class.java).createNotificationChannel(
            NotificationChannel("playback", "音乐播放", NotificationManager.IMPORTANCE_LOW))
        session = MediaSession(this, "QQLyricRust")
        session.setFlags(MediaSession.FLAG_HANDLES_MEDIA_BUTTONS or MediaSession.FLAG_HANDLES_TRANSPORT_CONTROLS)
        session.setCallback(object : MediaSession.Callback() {
            override fun onPlay() = resume()
            override fun onPause() = pause()
            override fun onStop() = stopPlayback()
            override fun onSkipToNext() = skip(1)
            override fun onSkipToPrevious() = skip(-1)
            override fun onSeekTo(pos: Long) { seek(pos.toInt()) }
        })
        session.isActive = true
        PlaybackStore.read(this)?.let{saved->
            song=saved.song;queue=JSONArray(saved.queue);queueJson=queue.toString();currentIndex=saved.index
            positionMs=saved.position;durationMs=saved.duration;currentArtwork=song.artwork();wantPlay=false
            account=JSONObject(SecureStore(this).load()?:"{}")
            playToken=java.util.UUID.randomUUID().toString()
            order.reset(queue.length(),currentIndex)
            publish()
        }
        ui.post(pulse)
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            "PLAY" -> {
                val url = intent.getStringExtra("url") ?: return START_NOT_STICKY
                generation++;skipPending=false;qualityPending=false;actualQuality=intent.getStringExtra("quality")?:"";downgraded=intent.getStringExtra("downgraded")=="true";playToken=intent.getStringExtra("token")?:java.util.UUID.randomUUID().toString()
                song = JSONObject(intent.getStringExtra("song") ?: "{}")
                lyric = intent.getStringExtra("lyric") ?: ""
                translation = intent.getStringExtra("translation") ?: ""
                wordLyrics = intent.getStringExtra("wordLyrics")?.takeIf{it.length<=384*1024} ?: "[]"
                queue = JSONArray(intent.getStringExtra("queue") ?: "[]")
                queueJson=queue.toString()
                account = JSONObject(intent.getStringExtra("session") ?: "{}")
                currentIndex = intent.getIntExtra("index", -1)
                order.reset(queue.length(),currentIndex)
                val autoPlay=!selectionInterruptedBySleep
                selectionPending=false;selectionInterruptedBySleep=false
                if(autoPlay)sleepFinished=false
                play(url,startPosition=intent.getIntExtra("position",0).coerceAtLeast(0),autoPlay=autoPlay)
            }
            "QUALITY" -> changeQuality(intent.getStringExtra("quality")?:"auto",intent.getStringExtra("fallback")!="false")
            "SELECT" -> {selectionPending=true;selectionInterruptedBySleep=false;generation++;if(skipPending)order.reset(queue.length(),currentIndex);qualityPending=false;skipPending=false;if(player==null)loading=false;pause()}
            "SLEEP" -> {
                when(val value=intent.getIntExtra("minutes",0)){
                    -1->sleepTimer.setAfterTrack()
                    in 1..90->sleepTimer.setMinutes(value,SystemClock.elapsedRealtime())
                    else->sleepTimer.clear()
                }
                sleepFinished=false;publish()
            }
            "PAUSE" -> pause()
            "RESUME" -> resume()
            "STOP" -> stopPlayback()
            "SEEK" -> seek(intent.getIntExtra("position", 0))
            "MODE" -> {
                val mode=runCatching{PlaybackOrder.Mode.valueOf(intent.getStringExtra("mode")?:"ORDER")}.getOrDefault(PlaybackOrder.Mode.ORDER)
                if(skipPending){generation++;skipPending=false};order.reset(queue.length(),currentIndex);order.setMode(mode)
                getSharedPreferences("library",MODE_PRIVATE).edit().putString("playback_mode",mode.name).apply();updateSnapshot()
            }
            "LYRIC" -> {
                if(intent.getStringExtra("mid")!=song.optString("mid")||intent.getStringExtra("token")!=playToken)return START_NOT_STICKY
                lyric = intent.getStringExtra("lyric") ?: ""
                translation = intent.getStringExtra("translation") ?: ""
                wordLyrics = intent.getStringExtra("wordLyrics")?.takeIf{it.length<=384*1024} ?: "[]"
                publish()
            }
            "NEXT" -> skip(1)
            "PREVIOUS" -> skip(-1)
        }
        return START_NOT_STICKY
    }

    private fun play(url:String,startPosition:Int=0,autoPlay:Boolean=true) {
        if(autoPlay)sleepFinished=false
        playbackError=""
        player?.release()
        playing = false;prepared=false;loading=true;wantPlay=autoPlay;positionMs=startPosition;artwork=null
        durationMs=(song.optLong("duration").coerceIn(0,Int.MAX_VALUE/1000L)*1000).toInt()
        player = MediaPlayer().apply {
            setAudioAttributes(AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_MEDIA)
                .setContentType(AudioAttributes.CONTENT_TYPE_MUSIC).build())
            setOnPreparedListener { if(player===it){
                prepared=true
                expireSleepTimer()
                if(startPosition>0){loading=true;it.seekTo(startPosition.coerceAtMost((it.duration-1).coerceAtLeast(0)))}
                else{loading=false;if(wantPlay){it.start();playing=true;recordListening()}}
                publish()
            } }
            setOnSeekCompleteListener{if(player===it){positionMs=it.currentPosition;expireSleepTimer();if(loading){loading=false;if(wantPlay){it.start();playing=true;recordListening()}};publish()}}
            setOnCompletionListener { if(player===it){playing=false;if(sleepTimer.consumeTrackEnd()){finishSleep()}else{publish();skip(1,true)}} }
            setOnErrorListener { failedPlayer, what, extra -> if(player!==failedPlayer)return@setOnErrorListener true;playbackError="音频加载失败（$what / $extra），请重试或切换歌曲"; prepared=false;loading=false;playing = false; publish(); true }
            setDataSource(this@PlayerService, android.net.Uri.parse(url), mapOf("Referer" to "https://y.qq.com/", "User-Agent" to "Mozilla/5.0"))
            prepareAsync()
        }
        val identity=song.optString("mid")
        currentArtwork=song.artwork()
        startForeground(42,notification());foreground=true;publish()
        val artworkUrl=currentArtwork
        if(artworkUrl.isNotEmpty())worker.execute{
            val bitmap=ArtworkCache.load(this,artworkUrl)
            ui.post{if(song.optString("mid")==identity){artwork=bitmap;publish()}}
        }
    }

    private fun pause() { wantPlay=false;if(prepared && playing)player?.pause();playing=false;publish() }
    private fun resume() {
        if(expireSleepTimer())return
        sleepFinished=false;wantPlay=true
        if(prepared&&!loading){player?.start();playing=true;recordListening();publish()}
        else if(!loading&&song.optString("mid").isNotBlank())resolveSavedPlayback()
        else publish()
    }
    private fun seek(position:Int){
        val target=position.coerceIn(0,durationMs)
        if(prepared)player?.seekTo(target)
        positionMs=target
        if(player==null&&song.optString("mid").isNotBlank())PlaybackStore.save(this,song,queue.objects(),currentIndex,target,durationMs)
        publish()
    }
    private fun stopPlayback() { persistPlayback(true,true);sleepTimer.clear();sleepFinished=false;generation++;actualQuality="";qualityPending=false;skipPending=false;downgraded=false;player?.release(); player = null;song=JSONObject();queue=JSONArray();queueJson="[]";lyric="";translation="";wordLyrics="[]";currentArtwork="";currentIndex=-1;positionMs=0;durationMs=0;playbackError="";prepared=false;loading=false;playing=false;publish();foreground=false;stopForeground(STOP_FOREGROUND_REMOVE);stopSelf() }

    private fun persistPlayback(force:Boolean=false,sync:Boolean=false){
        if(!prepared||loading||song.optString("mid").isBlank())return
        val now=SystemClock.elapsedRealtime()
        if(!force&&now-lastSave<5000)return
        positionMs=runCatching{player?.currentPosition}.getOrNull()?:positionMs
        PlaybackStore.save(this,song,queue.objects(),currentIndex,positionMs,durationMs,sync)
        lastSave=now
    }

    private fun resolveSavedPlayback(){
        if(skipPending){order.reset(queue.length(),currentIndex);skipPending=false}
        qualityPending=false
        val request=++generation
        account=JSONObject(SecureStore(this).load()?:"{}")
        val candidate=JSONObject(song.toString());val credentials=account.toString()
        val preferences=getSharedPreferences("library",MODE_PRIVATE)
        val quality=preferences.getString("audio_quality","auto")?:"auto"
        val fallback=preferences.getBoolean("quality_fallback",true)
        loading=true;playbackError="";playToken=java.util.UUID.randomUUID().toString()
        startForeground(42,notification());foreground=true;publish()
        worker.execute{
            try{
                val result=JSONObject(RustCore.call("play_url",JSONObject().put("song",candidate).put("session",JSONObject(credentials)).put("quality",quality).put("fallback",fallback).toString()))
                ui.post{if(request==generation){
                    loading=false
                    if(!result.optBoolean("ok")){wantPlay=false;playbackError=result.optString("error",getString(R.string.request_retry));publish()}
                    else{
                        val data=result.getJSONObject("data");actualQuality=data.optString("quality");downgraded=data.optBoolean("downgraded")
                        val position=if(durationMs>0&&positionMs>=durationMs)0 else positionMs
                        play(data.getString("url"),position,wantPlay)
                    }
                }}
                if(result.optBoolean("ok")){
                    val resultLyrics=JSONObject(RustCore.call("lyrics",JSONObject().put("mid",candidate.optString("mid")).put("id",candidate.optLong("id")).put("session",JSONObject(credentials)).toString()))
                    if(resultLyrics.optBoolean("ok"))ui.post{if(request==generation){val data=resultLyrics.getJSONObject("data");lyric=data.optString("lyric");translation=data.optString("translation");wordLyrics=data.optJSONArray("wordLyrics")?.toString()?.takeIf{it.length<=384*1024}?:"[]";publish()}}
                }
            }catch(_:Exception){ui.post{if(request==generation){loading=false;wantPlay=false;playbackError=getString(R.string.request_retry);publish()}}}
        }
    }

    private fun recordListening(){
        if(historyToken!=playToken&&playing){ListeningHistory.record(this,song);historyToken=playToken}
    }
    private fun expireSleepTimer():Boolean{
        if(!sleepTimer.consumeDeadline(SystemClock.elapsedRealtime()))return false
        finishSleep();return true
    }
    private fun finishSleep(){
        generation++;if(skipPending)order.reset(queue.length(),currentIndex)
        skipPending=false;qualityPending=false;sleepFinished=true
        if(player==null)loading=false
        if(selectionPending)selectionInterruptedBySleep=true
        pause()
    }

    private fun skip(offset:Int,automatic:Boolean=false){
        if(expireSleepTimer())return
        val next=order.step(offset,automatic)
        if(BuildConfig.DEBUG)android.util.Log.d("AuralisPlayback","step direction=$offset automatic=$automatic from=$currentIndex target=$next generation=$generation")
        if(next<0){if(automatic){playing=false;publish()};return}
        if(automatic&&next==currentIndex&&prepared){seek(0);resume();return}
        val candidate=queue.optJSONObject(next)?:return
        val request=++generation;skipPending=true;qualityPending=false
        val preferences=getSharedPreferences("library",MODE_PRIVATE)
        val quality=preferences.getString("audio_quality","auto")?:"auto"
        val fallback=preferences.getBoolean("quality_fallback",true)
        if(player==null)account=JSONObject(SecureStore(this).load()?:"{}")
        val credentials=account.toString()
        worker.execute{
            try{
                val result=JSONObject(RustCore.call("play_url",JSONObject().put("song",candidate).put("session",JSONObject(credentials)).put("quality",quality).put("fallback",fallback).toString()))
                if(!result.optBoolean("ok")){
                    ui.post{if(request==generation){skipPending=false;order.reset(queue.length(),currentIndex);playbackError=result.optString("error",getString(R.string.request_retry));publish()}}
                    return@execute
                }
                val url=result.getJSONObject("data").getString("url")
                ui.post{if(request==generation){skipPending=false;actualQuality=result.getJSONObject("data").optString("quality");downgraded=result.getJSONObject("data").optBoolean("downgraded");playToken=java.util.UUID.randomUUID().toString();song=candidate;lyric="";translation="";wordLyrics="[]";currentIndex=next;play(url)}}
                val lyrics=JSONObject(RustCore.call("lyrics",JSONObject().put("mid",candidate.optString("mid")).put("id",candidate.optLong("id")).put("session",JSONObject(credentials)).toString()))
                if(lyrics.optBoolean("ok"))ui.post{if(request==generation){val data=lyrics.getJSONObject("data");lyric=data.optString("lyric");translation=data.optString("translation");wordLyrics=data.optJSONArray("wordLyrics")?.toString()?.takeIf{it.length<=384*1024}?:"[]";publish()}}
            }catch(_:Exception){ui.post{if(request==generation){skipPending=false;order.reset(queue.length(),currentIndex);playbackError=getString(R.string.request_retry);publish()}}}
        }
    }

    private fun changeQuality(preference:String,fallback:Boolean){
        if(song.optString("mid").isBlank())return
        val settings=getSharedPreferences("library",MODE_PRIVATE)
        settings.edit().putString("audio_quality",preference).putBoolean("quality_fallback",fallback).apply()
        if(skipPending){order.reset(queue.length(),currentIndex);skipPending=false}
        if(player==null){
            generation++;loading=false;qualityPending=false
            if(wantPlay)resolveSavedPlayback()else publish()
            return
        }
        val request=++generation
        val candidate=song.toString();val credentials=account.toString()
        qualityPending=true;updateSnapshot()
        worker.execute{
            try{
                val result=JSONObject(RustCore.call("play_url",JSONObject().put("song",JSONObject(candidate)).put("session",JSONObject(credentials)).put("quality",preference).put("fallback",fallback).toString()))
                ui.post{if(request==generation){
                    qualityPending=false
                    if(!result.optBoolean("ok")){playbackError=getString(R.string.quality_kept)+" · "+result.optString("error");updateSnapshot()}
                    else{
                        val position=if(prepared)player?.currentPosition?:positionMs else positionMs
                        val resume=playing||(loading&&wantPlay)
                        val data=result.getJSONObject("data");actualQuality=data.optString("quality");downgraded=data.optBoolean("downgraded")
                        play(data.getString("url"),position,resume)
                    }
                }}
            }catch(_:Exception){ui.post{if(request==generation){qualityPending=false;playbackError=getString(R.string.quality_kept);updateSnapshot()}}}
        }
    }

    private fun updateSnapshot(){snapshot=PlaybackSnapshot(song.optString("mid"),currentTitle,currentArtist,currentArtwork,playing,positionMs,durationMs,lyric,translation,playbackError,playToken,actualQuality,qualityPending,downgraded,((sleepTimer.remaining(SystemClock.elapsedRealtime())+999)/1000).toInt(),sleepTimer.isAfterTrack,sleepFinished,sleepTimer.minutes,queueJson,currentIndex,wordLyrics)}

    private fun publish() {
        val duration = (if(prepared)player?.duration?.toLong() else null)?.takeIf { it > 0 } ?: durationMs.toLong().takeIf{it>0} ?: song.optLong("duration") * 1000
        durationMs = duration.toInt()
        if(prepared&&!loading)positionMs=player?.currentPosition ?: 0
        isPlaying = playing
        currentTitle = song.optString("title")
        currentArtist = song.optString("artist")
        currentLyric = lyric
        currentTranslation = translation
        persistPlayback(true)
        updateSnapshot()
        val extra = android.os.Bundle().apply {
            // LyriconProvider can read this ColorOS compatible field from MediaSession.
            putString("lyricInfo", JSONObject().put("lyric", lyric).put("transLyric", translation).toString())
        }
        session.setMetadata(MediaMetadata.Builder()
            .putString(MediaMetadata.METADATA_KEY_MEDIA_ID, song.optString("mid"))
            .putString(MediaMetadata.METADATA_KEY_TITLE, song.optString("title"))
            .putString(MediaMetadata.METADATA_KEY_ARTIST, song.optString("artist"))
            .putString(MediaMetadata.METADATA_KEY_ALBUM, song.optString("album"))
            .putLong(MediaMetadata.METADATA_KEY_DURATION, duration)
            .putBitmap(MediaMetadata.METADATA_KEY_ALBUM_ART,artwork)
            .putString(MediaMetadata.METADATA_KEY_ALBUM_ART_URI,currentArtwork)
            .putString("lyricInfo", extra.getString("lyricInfo"))
            .build())
        session.setPlaybackState(PlaybackState.Builder()
            .setActions(PlaybackState.ACTION_PLAY or PlaybackState.ACTION_PAUSE or PlaybackState.ACTION_STOP or PlaybackState.ACTION_SEEK_TO or PlaybackState.ACTION_SKIP_TO_NEXT or PlaybackState.ACTION_SKIP_TO_PREVIOUS)
            .setState(if(loading)PlaybackState.STATE_BUFFERING else if(playbackError.isNotEmpty()&&!playing)PlaybackState.STATE_ERROR else if (playing) PlaybackState.STATE_PLAYING else PlaybackState.STATE_PAUSED,
                positionMs.toLong(), if (playing) 1f else 0f, SystemClock.elapsedRealtime()).build())
        if (foreground) getSystemService(NotificationManager::class.java).notify(42, notification())
        sendBroadcast(Intent("dev.qqlyric.app.PLAYBACK").setPackage(packageName)
            .putExtra("playing", playing).putExtra("position", positionMs))
    }

    private fun notification(): Notification {
        val open = PendingIntent.getActivity(this, 0, Intent(this, MainActivity::class.java).setFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP).putExtra("open_player",true), PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        val action = if (playing) "PAUSE" else "RESUME"
        val control = PendingIntent.getService(this, 1, Intent(this, PlayerService::class.java).setAction(action),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        return Notification.Builder(this, "playback").setSmallIcon(R.drawable.ic_stat_auralis)
            .setStyle(Notification.MediaStyle().setMediaSession(session.sessionToken).setShowActionsInCompactView(0,1,2))
            .setLargeIcon(artwork)
            .setVisibility(Notification.VISIBILITY_PUBLIC)
            .setOnlyAlertOnce(true)
            .setContentTitle(song.optString("title", "Auralis"))
            .setContentText(song.optString("artist")).setContentIntent(open)
            .addAction(android.R.drawable.ic_media_previous,"上一首",PendingIntent.getService(this,2,Intent(this,PlayerService::class.java).setAction("PREVIOUS"),PendingIntent.FLAG_IMMUTABLE))
            .addAction(if (playing) android.R.drawable.ic_media_pause else android.R.drawable.ic_media_play,
                if (playing) "暂停" else "播放", control)
            .addAction(android.R.drawable.ic_media_next,"下一首",PendingIntent.getService(this,3,Intent(this,PlayerService::class.java).setAction("NEXT"),PendingIntent.FLAG_IMMUTABLE))
            .setOngoing(playing).build()
    }

    override fun onDestroy(){
        persistPlayback(true,true)
        generation++;song=JSONObject();ui.removeCallbacksAndMessages(null);worker.shutdownNow();player?.release();session.release()
        snapshot=PlaybackSnapshot();positionMs=0;durationMs=0;isPlaying=false;currentTitle="";currentArtist="";currentLyric="";currentTranslation="";currentIndex=-1;playbackError="";currentArtwork=""
        super.onDestroy()
    }
    override fun onBind(intent: Intent?): IBinder? = null
}
