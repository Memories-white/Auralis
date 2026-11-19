package dev.qqlyric.app

import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.os.Handler
import android.os.Looper
import android.util.Base64
import androidx.compose.runtime.*
import org.json.JSONArray
import org.json.JSONObject
import java.util.concurrent.Executors

fun JSONArray.objects()=List(length()){getJSONObject(it)}
fun JSONObject.artwork():String=optString("coverUrl").takeIf{it.startsWith("http")}?.replace("http://","https://")
    ?: optString("albumMid").takeIf{it.isNotBlank()}?.let{"https://y.gtimg.cn/music/photo_new/T002R500x500M000$it.jpg"} ?: ""

/** Account/protocol state stays independent of the Compose page tree. */
class MusicState(private val context:Context){
    private val worker=Executors.newSingleThreadExecutor()
    private val feedWorker=Executors.newSingleThreadExecutor()
    private val ui=Handler(Looper.getMainLooper())
    private val secure=SecureStore(context)
    private val prefs=context.getSharedPreferences("library",Context.MODE_PRIVATE)
    private var session=JSONObject(secure.load()?:"{}")
    private var epoch=0
    private var loginGeneration=0
    private var playGeneration=0
    private var searchGeneration=0
    private var playlistGeneration=0
    private var homeGeneration=0
    private var feedGeneration=0
    private var feedPage=1
    var discoverySongs by mutableStateOf(emptyList<JSONObject>())
    var discoveryLoading by mutableStateOf(false)
    var discoveryMore by mutableStateOf(true)
    var discoveryError by mutableStateOf("")
    var collectionBusy by mutableStateOf(false)
    var collectionMessage by mutableStateOf("")
    var detailRevision by mutableIntStateOf(0)
    private val backStack=mutableListOf<String>()
    private var pendingMid:String?=null
    private var pendingToken:String?=null
    private var lastServiceError=""
    var mid by mutableStateOf(PlayerService.snapshot.mid)
    var monetEnabled by mutableStateOf(prefs.getBoolean("monet_enabled",false))
    var monetSource by mutableStateOf(prefs.getString("monet_source","wallpaper")?:"wallpaper")
    var monetColor by mutableStateOf(prefs.getString("monet_color","3482FF")?:"3482FF")
    var monetStyle by mutableStateOf(prefs.getString("monet_style","TonalSpot")?:"TonalSpot")
    var monetSpec by mutableStateOf(prefs.getString("monet_spec","Spec2021")?:"Spec2021")
    var audioQuality by mutableStateOf(prefs.getString("audio_quality","auto")?:"auto")
    var qualityFallback by mutableStateOf(prefs.getBoolean("quality_fallback",true))
    var actualQuality by mutableStateOf("")
    var qualityPending by mutableStateOf(false)
    var qualityDowngraded by mutableStateOf(false)
    var playbackMode by mutableStateOf(prefs.getString("playback_mode","ORDER")?:"ORDER")
    var page by mutableStateOf("home")
    var parentPage="home"
    var detailTitle by mutableStateOf("")
    var searchText by mutableStateOf("")
    var searchResults by mutableStateOf(emptyList<JSONObject>())
    var searchedQuery by mutableStateOf("")
    var searchHistory by mutableStateOf(JSONArray(prefs.getString("search_history","[]")).let{a->List(a.length()){a.optString(it)}.filter{it.isNotBlank()}.take(8)})
    var status by mutableStateOf("")
    var busy by mutableStateOf(false)
    var home by mutableStateOf(JSONObject())
    var library by mutableStateOf(JSONObject())
    var tracks by mutableStateOf(emptyList<JSONObject>())
    var favorites by mutableStateOf(JSONArray(prefs.getString("favorites","[]")).objects())
    var queue by mutableStateOf(JSONArray(prefs.getString("active_queue","[]")).objects())
    var loggedIn by mutableStateOf(session.optString("musicKey").isNotBlank())
    var profile by mutableStateOf(session.optJSONObject("profile")?:JSONObject())
    var colorMode by mutableStateOf(prefs.getString("color_mode","system")?:"system")
    var qrOpen by mutableStateOf(false)
    var qrBitmap by mutableStateOf<Bitmap?>(null)
    var qrStatus by mutableStateOf("")
    var title by mutableStateOf(PlayerService.currentTitle)
    var artist by mutableStateOf(PlayerService.currentArtist)
    var cover by mutableStateOf(PlayerService.currentArtwork)
    var playing by mutableStateOf(PlayerService.isPlaying)
    var position by mutableIntStateOf(0)
    var duration by mutableIntStateOf(0)
    var lyrics by mutableStateOf("")
    var translation by mutableStateOf("")
    val uin get()=session.optString("uin")
    val avatar get()=profile.optString("avatar").takeIf{it.startsWith("http")}?.replace("http://","https://")
        ?: uin.takeIf{it.matches(Regex("[0-9]{5,11}"))}?.let{"https://q1.qlogo.cn/g?b=qq&nk=$it&s=160"} ?: ""
    private var alive=true
    init{prefs.edit().putString("ui_style","MIUIX").apply();refreshHome();if(loggedIn)refreshProfile()}
    private fun call(name:String,args:JSONObject=JSONObject(),done:(JSONObject)->Unit){
        val expected=epoch
        (if(name=="recommend")feedWorker else worker).execute{
            val response=runCatching{JSONObject(RustCore.call(name,args.toString()))}.getOrElse{JSONObject().put("ok",false).put("error",it.message?:context.getString(R.string.request_failed))}
            ui.post{if(alive&&expected==epoch)done(response)}
        }
    }
    private fun failed(result:JSONObject):Boolean{
        if(result.optBoolean("ok"))return false
        busy=false
        val text=result.optString("error",context.getString(R.string.request_retry))
        status=if(text.contains("resolve",true)||text.contains("lookup address",true))context.getString(R.string.network_retry)else text
        return true
    }
    fun refreshHome(){
        if(!loggedIn){status=context.getString(R.string.login_recommendations);return}
        val generation=++homeGeneration
        busy=true;status=context.getString(R.string.loading_recommendations)
        call("home",JSONObject().put("session",session)){result->
            if(generation!=homeGeneration)return@call
            busy=false;if(!failed(result)){home=result.getJSONObject("data");status="";resetDiscovery()}
        }
    }
    private fun resetDiscovery(){feedGeneration++;feedPage=1;discoverySongs=emptyList();discoveryLoading=false;discoveryMore=true;discoveryError="";loadMoreDiscovery()}
    fun loadMoreDiscovery(){
        if(!loggedIn||discoveryLoading||!discoveryMore)return
        val request=feedGeneration
        val daily=home.optJSONArray("daily")?.objects().orEmpty()
        discoveryLoading=true;discoveryError=""
        call("recommend",JSONObject().put("session",session).put("page",feedPage)
            .put("exclude",JSONArray((daily+discoverySongs).map{it.optString("mid")}))
            .put("excludeIds",JSONArray(daily.map{it.optLong("id")}))){result->
            if(request!=feedGeneration)return@call
            discoveryLoading=false
            if(!result.optBoolean("ok")){discoveryError=result.optString("error",context.getString(R.string.request_retry));return@call}
            val data=result.getJSONObject("data")
            val blocked=(home.optJSONArray("daily")?.objects().orEmpty()+discoverySongs).map{it.optString("mid")}.toHashSet()
            val added=data.optJSONArray("songs")?.objects().orEmpty().filter{blocked.add(it.optString("mid"))}
            discoverySongs=discoverySongs+added;feedPage=data.optInt("nextPage",feedPage+1)
            discoveryMore=data.optBoolean("hasMore")&&added.isNotEmpty()
            if(BuildConfig.DEBUG)android.util.Log.i("AuralisDiscovery","batch=${added.size} total=${discoverySongs.size} next=$feedPage more=$discoveryMore dailyOverlap=${discoverySongs.count{s->daily.any{it.optString("mid")==s.optString("mid")}}}")
        }
    }
    fun refreshLibrary(){
        if(!loggedIn)return
        busy=true;status=context.getString(R.string.loading_library)
        call("library",JSONObject().put("session",session)){result->busy=false;if(!failed(result)){library=result.getJSONObject("data");status=""}}
    }
    fun navigate(destination:String){
        if(destination==page)return
        playlistGeneration++;searchGeneration++
        if(destination=="home"||destination=="mine")backStack.clear() else backStack.add(page)
        page=destination;status="";busy=false
        if(destination=="mine"&&loggedIn&&library.length()==0)refreshLibrary()
    }
    fun back(){playlistGeneration++;searchGeneration++;page=if(backStack.isNotEmpty())backStack.removeAt(backStack.lastIndex)else "home";status="";busy=false}
    fun openSongs(songs:List<JSONObject>,label:String){if(page!="songs")parentPage=page;detailTitle=label;tracks=songs;detailRevision++;navigate("songs")}
    fun daily(){
        val songs=home.optJSONArray("daily")?.objects().orEmpty()
        if(songs.isNotEmpty())openSongs(songs,context.getString(R.string.daily_mix))
        else if(!loggedIn)login()else if(home.optLong("dailyId")>0)playlist(home.optLong("dailyId"),0,context.getString(R.string.daily_mix))else refreshHome()
    }
    fun playlist(id:Long,dirid:Long,label:String){
        val request=++playlistGeneration;busy=true;status=context.getString(R.string.loading_playlist)
        call("playlist",JSONObject().put("id",id).put("dirid",dirid).put("session",session)){result->
            if(request!=playlistGeneration)return@call
            busy=false;if(!failed(result)){openSongs(result.getJSONArray("data").objects(),label);status=""}
        }
    }
    fun search(){
        val query=searchText.trim();if(query.isEmpty())return
        searchHistory=(listOf(query)+searchHistory.filterNot{it==query}).take(8)
        prefs.edit().putString("search_history",JSONArray(searchHistory).toString()).apply()
        searchedQuery=query;searchResults=emptyList()
        val request=++searchGeneration;busy=true;status=context.getString(R.string.searching)
        call("search",JSONObject().put("query",query).put("session",session)){result->
            if(request!=searchGeneration)return@call
            busy=false;if(!failed(result)){searchResults=result.getJSONArray("data").objects();status=context.getString(R.string.search_count,searchResults.size)}
        }
    }
    fun clearSearchHistory(){searchHistory=emptyList();prefs.edit().remove("search_history").apply()}
    fun currentSong():JSONObject?=queue.firstOrNull{it.optString("mid")==mid}
    fun cloudLiked(song:JSONObject)=library.optJSONArray("likes")?.objects().orEmpty().any{it.optString("mid")==song.optString("mid")}
    fun writablePlaylists()=library.optJSONArray("playlists")?.objects().orEmpty().filter{!it.optBoolean("saved")&&it.optLong("dirid")>0}
    fun addCloudSong(song:JSONObject,dirid:Long){
        if(collectionBusy)return
        if(!loggedIn){collectionMessage=context.getString(R.string.login_library_summary);return}
        if(dirid==201L&&cloudLiked(song)){collectionMessage=context.getString(R.string.already_liked);return}
        collectionBusy=true;collectionMessage=context.getString(R.string.adding_song)
        call("add_song",JSONObject().put("session",session).put("dirid",dirid).put("id",song.optLong("id"))){result->
            collectionBusy=false
            if(!result.optBoolean("ok")){collectionMessage=result.optString("error",context.getString(R.string.request_retry));return@call}
            val data=result.getJSONObject("data")
            if(data.has("confirmed")&&!data.optBoolean("confirmed")){
                collectionMessage=context.getString(R.string.song_add_accepted)
                if(BuildConfig.DEBUG)android.util.Log.i("AuralisCollections","acknowledged; response keys=${data.optJSONArray("responseKeys")}")
                refreshLibrary();return@call
            }
            collectionMessage=context.getString(if(data.optBoolean("alreadyExists"))R.string.song_already_present else R.string.song_added)
            if(dirid==201L){val updated=library.optJSONArray("likes")?.objects().orEmpty().filterNot{it.optString("mid")==song.optString("mid")};library=JSONObject(library.toString()).put("likes",JSONArray(listOf(song)+updated))}
            refreshLibrary()
        }
    }
    fun play(songs:List<JSONObject>,index:Int){
        if(index !in songs.indices)return
        val preferred=audioQuality;val fallback=qualityFallback
        val song=songs[index];val request=++playGeneration;val token=java.util.UUID.randomUUID().toString();pendingToken=token;pendingMid=song.optString("mid");mid=pendingMid?:"";title=song.optString("title");artist=song.optString("artist");cover=song.artwork();lyrics="";translation="";actualQuality="";qualityPending=false;qualityDowngraded=false;position=0;duration=song.optInt("duration")*1000;playing=false;status=context.getString(R.string.loading_song)
        if(PlayerService.snapshot.mid.isNotBlank())command("SELECT")
        call("play_url",JSONObject().put("song",song).put("session",session).put("quality",preferred).put("fallback",fallback)){result->
            if(request!=playGeneration)return@call
            if(preferred!=audioQuality||fallback!=qualityFallback){play(songs,index);return@call}
            if(failed(result)){pendingMid=null;pendingToken=null;return@call}
            queue=songs.toList();prefs.edit().putString("active_queue",JSONArray(songs).toString()).apply()
            command("PLAY","quality",result.getJSONObject("data").optString("quality"),"downgraded",result.getJSONObject("data").optBoolean("downgraded").toString(),"token",token,"url",result.getJSONObject("data").getString("url"),"song",song.toString(),"lyric","","queue",JSONArray(songs).toString(),"session",session.toString(),"index",index)
            title=song.optString("title");artist=song.optString("artist");cover=song.artwork();status=""
            call("lyrics",JSONObject().put("mid",song.optString("mid")).put("id",song.optLong("id")).put("session",session)){lyricResult->
                if(request==playGeneration&&lyricResult.optBoolean("ok")){
                    val data=lyricResult.getJSONObject("data");command("LYRIC","token",token,"mid",song.optString("mid"),"lyric",data.optString("lyric"),"translation",data.optString("translation"))
                }
            }
        }
    }
    fun command(action:String,vararg extras:Any){
        val intent=Intent(context,PlayerService::class.java).setAction(action)
        for(i in extras.indices step 2){if(i+1>=extras.size)break;val key=extras[i] as String;val value=extras[i+1];if(value is Int)intent.putExtra(key,value)else intent.putExtra(key,value.toString())}
        if(action=="PLAY")context.startForegroundService(intent)else context.startService(intent)
    }
    fun sample(){
        val state=PlayerService.snapshot
        if(pendingToken!=null){if(state.token!=pendingToken)return else {pendingMid=null;pendingToken=null}}
        mid=state.mid;title=state.title;artist=state.artist;cover=state.artwork
        playing=state.playing;position=state.position;duration=state.duration
        lyrics=state.lyric;translation=state.translation
        actualQuality=state.quality;qualityPending=state.qualityPending;qualityDowngraded=state.downgraded
        if(state.error.isNotBlank())status=state.error else if(lastServiceError.isNotBlank()&&status==lastServiceError)status=""
        lastServiceError=state.error
    }
    fun chooseAudioQuality(value:String){
        if(value !in listOf("auto","sq","hq","standard"))return
        audioQuality=value;prefs.edit().putString("audio_quality",value).apply();applyQuality()
    }
    fun allowQualityFallback(enabled:Boolean){qualityFallback=enabled;prefs.edit().putBoolean("quality_fallback",enabled).apply();applyQuality()}
    private fun applyQuality(){if(pendingToken==null&&PlayerService.snapshot.mid.isNotBlank())command("QUALITY","quality",audioQuality,"fallback",qualityFallback.toString())}
    fun setMonet(enabled:Boolean){monetEnabled=enabled;prefs.edit().putBoolean("monet_enabled",enabled).apply()}
    fun setMonetOption(key:String,value:String){
        when(key){"source"->monetSource=value;"color"->monetColor=value;"style"->monetStyle=value;"spec"->monetSpec=value;else->return}
        prefs.edit().putString("monet_$key",value).apply()
    }
    fun choosePlaybackMode(mode:String){
        if(PlaybackOrder.Mode.values().toList().none{it.name==mode})return
        playbackMode=mode;prefs.edit().putString("playback_mode",mode).apply()
        if(PlayerService.snapshot.mid.isNotBlank())command("MODE","mode",mode)
    }
    fun cyclePlaybackMode(){val modes=PlaybackOrder.Mode.values().toList();choosePlaybackMode(modes[(modes.indexOfFirst{it.name==playbackMode}+1)%modes.size].name)}
    fun recommendations()=home.optJSONArray("cards")?.objects().orEmpty().filterNot{
        val name=it.optString("title").replace(Regex("\\s+"),"")
        (home.optLong("dailyId")>0&&it.optLong("id")==home.optLong("dailyId"))||name.contains("每日30首")||name.contains("每日三十首")
    }
    fun favorite():Boolean=if(pendingToken!=null)false else queue.getOrNull(PlayerService.currentIndex)?.let{song->favorites.any{it.optString("mid")==song.optString("mid")}}?:false
    fun toggleFavorite(){
        if(pendingToken!=null)return
        val song=queue.getOrNull(PlayerService.currentIndex)?:return
        favorites=if(favorite())favorites.filterNot{it.optString("mid")==song.optString("mid")}else favorites+song
        prefs.edit().putString("favorites",JSONArray(favorites).toString()).apply()
    }
    fun setAppearance(mode:String){colorMode=mode;prefs.edit().putString("color_mode",mode).apply()}
    private fun refreshProfile(){call("profile",JSONObject().put("session",session)){result->if(result.optBoolean("ok")){profile=result.getJSONObject("data");session.put("profile",profile);secure.save(session.toString())}}}
    fun login(){
        val generation=++loginGeneration;qrOpen=true;qrBitmap=null;qrStatus=context.getString(R.string.getting_qr)
        call("qr_start"){result->
            if(generation!=loginGeneration||!qrOpen)return@call
            if(failed(result)){qrStatus=status;return@call}
            val data=result.getJSONObject("data");val bytes=Base64.decode(data.getString("image"),Base64.DEFAULT)
            qrBitmap=BitmapFactory.decodeByteArray(bytes,0,bytes.size);qrStatus=context.getString(R.string.scan_qq);poll(generation,data.getString("qrsig"))
        }
    }
    private fun poll(generation:Int,sig:String){
        if(generation!=loginGeneration||!qrOpen)return
        call("qr_poll",JSONObject().put("qrsig",sig)){result->
            if(generation!=loginGeneration||!qrOpen)return@call
            if(failed(result)){qrStatus=status;return@call}
            val data=result.getJSONObject("data")
            when(data.optString("status")){
                "success"->{session=data.getJSONObject("session");secure.save(session.toString());loggedIn=true;dismissLogin();navigate("mine");refreshLibrary();refreshProfile();refreshHome()}
                "expired","canceled"->qrStatus=context.getString(R.string.qr_expired)
                else->{qrStatus=if(data.optString("status")=="confirming")context.getString(R.string.confirm_qq)else context.getString(R.string.waiting_scan);ui.postDelayed({poll(generation,sig)},1800)}
            }
        }
    }
    fun dismissLogin(){loginGeneration++;qrOpen=false}
    fun logout(){homeGeneration++;feedGeneration++;discoverySongs=emptyList();discoveryLoading=false;discoveryMore=true;discoveryError="";collectionBusy=false;collectionMessage="";epoch++;loginGeneration++;playGeneration++;pendingMid=null;pendingToken=null;backStack.clear();secure.clear();session=JSONObject();profile=JSONObject();loggedIn=false;home=JSONObject();library=JSONObject();page="home";status=context.getString(R.string.logged_out);command("STOP")}
    fun close(){alive=false;loginGeneration++;worker.shutdownNow();feedWorker.shutdownNow();ui.removeCallbacksAndMessages(null)}
}
