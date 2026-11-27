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
class MusicState(private val context:Context,private val protocol:(String,String)->String={name,input->RustCore.call(name,input)}){
    private val worker=Executors.newSingleThreadExecutor()
    private val feedWorker=Executors.newSingleThreadExecutor()
    private val detailWorker=Executors.newSingleThreadExecutor()
    /** Songs requested per playlist page; the page keeps loading while the user scrolls. */
    private val trackPageSize=100
    private val ui=Handler(Looper.getMainLooper())
    private val secure=SecureStore(context)
    private val prefs=context.getSharedPreferences("library",Context.MODE_PRIVATE)
    private val playbackPrefs=context.getSharedPreferences(PlaybackStore.PREFS,Context.MODE_PRIVATE)
    private val catalogPrefs=context.getSharedPreferences("artwork-catalog",Context.MODE_PRIVATE)
    private var savedPlayback=PlaybackStore.read(context)
    private var savedPlaybackDirty=false
    private val playbackListener=android.content.SharedPreferences.OnSharedPreferenceChangeListener{_,key->if(key==PlaybackStore.KEY)savedPlaybackDirty=true}
    private var lastLiveMid=PlayerService.snapshot.mid
    private var appliedQueueJson=""
    private var session=JSONObject(secure.load()?:"{}")
    private var epoch=0
    private var loginGeneration=0
    private var playGeneration=0
    private var searchGeneration=0
    private var playlistGeneration=0
    private var homeGeneration=0
    private var feedGeneration=0
    private var feedPage=1
    private var replacingDiscovery=false
    private var libraryFromCache=false
    var discoverySongs by mutableStateOf(emptyList<JSONObject>())
    var discoveryLoading by mutableStateOf(false)
    var discoveryMore by mutableStateOf(true)
    var discoveryError by mutableStateOf("")
    var collectionBusy by mutableStateOf(false)
    var collectionMessage by mutableStateOf("")
    var detailRevision by mutableIntStateOf(0)
    var detailLoading by mutableStateOf(false)
    var detailError by mutableStateOf("")
    /** Songs the service reports for the open playlist, not just the loaded page. */
    var detailTotal by mutableIntStateOf(0)
    var detailHasMore by mutableStateOf(false)
    var detailLoadingMore by mutableStateOf(false)
    private data class DetailRequest(val kind:String,val id:Long,val dirid:Long,val title:String,val cover:String,val excluded:List<String> = emptyList())
    private var detailRequest:DetailRequest?=null
    private val backStack=mutableListOf<String>()
    private var pendingMid:String?=null
    private var pendingToken:String?=null
    private var lastServiceError=""
    var mid by mutableStateOf(PlayerService.snapshot.mid.ifBlank{savedPlayback?.song?.optString("mid").orEmpty()})
    var currentIndex by mutableIntStateOf(if(PlayerService.snapshot.mid.isNotBlank())PlayerService.snapshot.index else savedPlayback?.index?:-1)
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
    var detailCover by mutableStateOf("")
    var recentPlays by mutableStateOf(ListeningHistory.read(context))
    var sleepRemainingSeconds by mutableIntStateOf(0)
    var sleepAfterTrack by mutableStateOf(false)
    var sleepFinished by mutableStateOf(false)
    var sleepMinutes by mutableIntStateOf(0)
    private val historyListener=android.content.SharedPreferences.OnSharedPreferenceChangeListener { _,key->if(key==ListeningHistory.KEY)recentPlays=ListeningHistory.read(context) }
    var searchText by mutableStateOf("")
    var searchResults by mutableStateOf(emptyList<JSONObject>())
    var searchedQuery by mutableStateOf("")
    var searchHistory by mutableStateOf(JSONArray(prefs.getString("search_history","[]")).let{a->List(a.length()){a.optString(it)}.filter{it.isNotBlank()}.take(8)})
    var status by mutableStateOf("")
    var playbackStatus by mutableStateOf("")
    var busy by mutableStateOf(false)
    var home by mutableStateOf(JSONObject())
    var library by mutableStateOf(JSONObject())
    var tracks by mutableStateOf(emptyList<JSONObject>())
    var favorites by mutableStateOf(JSONArray(prefs.getString("favorites","[]")).objects())
    var queue by mutableStateOf(if(PlayerService.snapshot.mid.isNotBlank())runCatching{JSONArray(PlayerService.snapshot.queueJson).objects()}.getOrDefault(emptyList())else savedPlayback?.queue?:runCatching{JSONArray(prefs.getString("active_queue","[]")).objects()}.getOrDefault(emptyList()))
    var loggedIn by mutableStateOf(session.optString("musicKey").isNotBlank())
    var profile by mutableStateOf(session.optJSONObject("profile")?:JSONObject())
    var colorMode by mutableStateOf(prefs.getString("color_mode","system")?:"system")
    var qrOpen by mutableStateOf(false)
    var qrBitmap by mutableStateOf<Bitmap?>(null)
    var qrStatus by mutableStateOf("")
    var title by mutableStateOf(PlayerService.currentTitle.ifBlank{savedPlayback?.song?.optString("title").orEmpty()})
    var artist by mutableStateOf(PlayerService.currentArtist.ifBlank{savedPlayback?.song?.optString("artist").orEmpty()})
    var cover by mutableStateOf(PlayerService.currentArtwork.ifBlank{savedPlayback?.song?.artwork().orEmpty()})
    var playing by mutableStateOf(PlayerService.isPlaying)
    var position by mutableIntStateOf(if(PlayerService.snapshot.mid.isNotBlank())PlayerService.snapshot.position else savedPlayback?.position?:0)
    var duration by mutableIntStateOf(if(PlayerService.snapshot.mid.isNotBlank())PlayerService.snapshot.duration else savedPlayback?.duration?:0)
    var lyrics by mutableStateOf("")
    var translation by mutableStateOf("")
    val uin get()=session.optString("uin")
    val avatar get()=profile.optString("avatar").takeIf{it.startsWith("http")}?.replace("http://","https://")
        ?: uin.takeIf{it.matches(Regex("[0-9]{5,11}"))}?.let{"https://q1.qlogo.cn/g?b=qq&nk=$it&s=160"} ?: ""
    private var alive=true
    init{prefs.registerOnSharedPreferenceChangeListener(historyListener);playbackPrefs.registerOnSharedPreferenceChangeListener(playbackListener);prefs.edit().putString("ui_style","MIUIX").apply();restoreCatalog();refreshHome();if(loggedIn)refreshProfile()}
    private fun restoreCatalog(){
        if(!loggedIn||catalogPrefs.getString("account","")!=uin)return
        runCatching{JSONObject(catalogPrefs.getString("value","{}").orEmpty())}.getOrNull()?.let{cached->
            home=cached.optJSONObject("home")?:JSONObject();library=cached.optJSONObject("library")?:JSONObject()
            libraryFromCache=library.length()>0
            discoverySongs=cached.optJSONArray("discovery")?.objects().orEmpty()
        }
    }
    private fun saveCatalog(){
        if(!loggedIn)return
        val input=JSONObject().put("home",home).put("library",library).put("discovery",JSONArray(discoverySongs)).toString()
        val account=uin
        // Rust strips unsupported fields and bounds every shelf. Apply is asynchronous.
        runCatching{JSONObject(protocol("artwork_catalog",input)).optJSONObject("data")}.getOrNull()?.toString()?.takeIf{it.length<=256*1024}?.let{value->catalogPrefs.edit().putString("account",account).putString("value",value).apply()}
    }
    private fun call(name:String,args:JSONObject=JSONObject(),done:(JSONObject)->Unit){
        val expected=epoch
        val input=args.toString()
        (when(name){"recommend"->feedWorker;"guess","playlist","playlist_page","playlist_all"->detailWorker;else->worker}).execute{
            val response=runCatching{JSONObject(protocol(name,input))}.getOrElse{JSONObject().put("ok",false).put("error",it.message?:context.getString(R.string.request_failed))}
            ui.post{if(alive&&expected==epoch)done(response)}
        }
    }
    private fun failed(result:JSONObject):Boolean{
        if(result.optBoolean("ok"))return false
        busy=false
        status=displayError(result.optString("error",context.getString(R.string.request_retry)))
        return true
    }
    private fun displayError(text:String)=when{
        text.contains("所选音质未返回可用音源")->context.getString(R.string.playback_unavailable)
        text.contains("resolve",true)||text.contains("lookup address",true)||text.contains("无法解析")->context.getString(R.string.network_retry)
        else->text
    }
    fun refreshHome(){
        if(!loggedIn){status=context.getString(R.string.login_recommendations);return}
        val generation=++homeGeneration
        busy=true;status=context.getString(R.string.loading_recommendations)
        call("home",JSONObject().put("session",session)){result->
            if(generation!=homeGeneration)return@call
            busy=false;if(!failed(result)){home=result.getJSONObject("data");status="";saveCatalog();resetDiscovery()}
        }
    }
    private fun resetDiscovery(){feedGeneration++;feedPage=1;replacingDiscovery=true;discoveryLoading=false;discoveryMore=true;discoveryError="";loadMoreDiscovery()}
    fun loadMoreDiscovery(){
        if(!loggedIn||discoveryLoading||!discoveryMore)return
        val request=feedGeneration
        val daily=home.optJSONArray("daily")?.objects().orEmpty()
        discoveryLoading=true;discoveryError=""
        call("recommend",JSONObject().put("session",session).put("page",feedPage)
            .put("exclude",JSONArray((daily+if(replacingDiscovery)emptyList()else discoverySongs).map{it.optString("mid")}))
            .put("excludeIds",JSONArray(daily.map{it.optLong("id")}))){result->
            if(request!=feedGeneration)return@call
            discoveryLoading=false
            if(!result.optBoolean("ok")){discoveryError=result.optString("error",context.getString(R.string.request_retry));return@call}
            val data=result.getJSONObject("data")
            val blocked=(home.optJSONArray("daily")?.objects().orEmpty()+if(replacingDiscovery)emptyList()else discoverySongs).map{it.optString("mid")}.toHashSet()
            val added=data.optJSONArray("songs")?.objects().orEmpty().filter{blocked.add(it.optString("mid"))}
            discoverySongs=(if(replacingDiscovery)emptyList()else discoverySongs)+added;replacingDiscovery=false;feedPage=data.optInt("nextPage",feedPage+1);saveCatalog()
            discoveryMore=data.optBoolean("hasMore")&&added.isNotEmpty()
            if(BuildConfig.DEBUG)android.util.Log.i("AuralisDiscovery","batch=${added.size} total=${discoverySongs.size} next=$feedPage more=$discoveryMore dailyOverlap=${discoverySongs.count{s->daily.any{it.optString("mid")==s.optString("mid")}}}")
        }
    }
    fun refreshLibrary(){
        if(!loggedIn)return
        busy=true;status=context.getString(R.string.loading_library)
        call("library",JSONObject().put("session",session)){result->busy=false;if(!failed(result)){library=result.getJSONObject("data");libraryFromCache=false;status="";saveCatalog()}}
    }
    fun navigate(destination:String){
        if(destination==page)return
        playlistGeneration++;searchGeneration++
        if(destination!="songs")cancelDetail()
        if(destination=="home"||destination=="mine"||destination=="settings")backStack.clear() else backStack.add(page)
        page=destination;status="";busy=false
        if(destination=="mine"&&loggedIn&&(library.length()==0||libraryFromCache))refreshLibrary()
    }
    private fun cancelDetail(){detailLoading=false;detailError="";detailRequest=null;detailHasMore=false;detailLoadingMore=false;detailTotal=0}
    fun back(){playlistGeneration++;searchGeneration++;cancelDetail();page=if(backStack.isNotEmpty())backStack.removeAt(backStack.lastIndex)else "home";status="";busy=false}
    fun openSongs(songs:List<JSONObject>,label:String,cover:String=""){playlistGeneration++;cancelDetail();if(page!="songs")parentPage=page;detailTitle=label;detailCover=cover.ifBlank{songs.firstOrNull()?.artwork().orEmpty()};tracks=songs;detailTotal=songs.size;detailRevision++;navigate("songs")}
    fun daily(){
        val songs=home.optJSONArray("daily")?.objects().orEmpty()
        if(songs.isNotEmpty())openSongs(songs,context.getString(R.string.daily_mix))
        else if(!loggedIn)login()else if(home.optLong("dailyId")>0)playlist(home.optLong("dailyId"),0,context.getString(R.string.daily_mix))else refreshHome()
    }
    fun playlist(id:Long,dirid:Long,label:String,cover:String=""){
        loadDetail(DetailRequest("playlist",id,dirid,label,cover.takeUnless{it=="null"}.orEmpty()))
    }
    fun recommendedPlaylist(card:JSONObject){
        val title=card.optString("title")
        if(card.optString("kind")!="guess"){
            playlist(card.optLong("id"),card.optLong("dirid"),title,card.optString("cover"));return
        }
        if(!loggedIn){login();return}
        val excluded=home.optJSONArray("daily")?.objects().orEmpty().map{it.optString("mid")}
        loadDetail(DetailRequest("guess",0,0,title,card.optString("cover").takeUnless{it=="null"}.orEmpty(),excluded))
    }
    private fun loadDetail(source:DetailRequest){
        if(detailLoading&&page=="songs"&&detailRequest==source)return
        if(!loggedIn){login();return}
        openSongs(emptyList(),source.title,source.cover)
        val request=++playlistGeneration
        detailRequest=source;status="";detailLoading=true
        if(source.kind=="playlist"&&source.id<=0&&source.dirid<=0){
            detailLoading=false;detailError=context.getString(R.string.playlist_invalid);return
        }
        val args=JSONObject().put("session",session).put("id",source.id).put("dirid",source.dirid).put("exclude",JSONArray(source.excluded))
        if(source.kind=="playlist")args.put("offset",0).put("size",trackPageSize)
        val command=if(source.kind=="playlist")"playlist_page" else source.kind
        if(BuildConfig.DEBUG)android.util.Log.d("AuralisPlaylist","open kind=${source.kind} generation=$request")
        call(command,args){result->
            if(request!=playlistGeneration||page!="songs"||detailRequest!=source)return@call
            detailLoading=false
            if(!result.optBoolean("ok")){
                detailError=displayError(result.optString("error",context.getString(R.string.request_retry)))
            }else if(source.kind=="playlist"){
                val payload=result.optJSONObject("data")
                val songs=payload?.optJSONArray("songs")
                if(payload==null||songs==null){detailError=context.getString(R.string.playlist_response_invalid);return@call}
                tracks=songs.objects()
                detailTotal=payload.optInt("total",tracks.size)
                detailHasMore=payload.optBoolean("hasMore")
                if(detailCover.isBlank())detailCover=tracks.firstOrNull()?.artwork().orEmpty()
                detailRevision++
                if(BuildConfig.DEBUG)android.util.Log.d("AuralisPlaylist","loaded kind=${source.kind} count=${tracks.size} total=$detailTotal more=$detailHasMore generation=$request")
            }else{
                val songs=result.optJSONArray("data")
                if(songs==null){detailError=context.getString(R.string.playlist_response_invalid);return@call}
                tracks=songs.objects()
                detailTotal=tracks.size
                detailHasMore=false
                if(detailCover.isBlank())detailCover=tracks.firstOrNull()?.artwork().orEmpty()
                if(BuildConfig.DEBUG)android.util.Log.d("AuralisPlaylist","loaded kind=${source.kind} count=${tracks.size} generation=$request")
            }
        }
    }
    /** Fetch the next page for the open playlist instead of stopping at the first 100. */
    fun loadMoreTracks(){
        val source=detailRequest?:return
        if(source.kind!="playlist"||!detailHasMore||detailLoading||detailLoadingMore||page!="songs")return
        val request=playlistGeneration
        detailLoadingMore=true
        val args=JSONObject().put("session",session).put("id",source.id).put("dirid",source.dirid)
            .put("offset",tracks.size).put("size",trackPageSize)
        call("playlist_page",args){result->
            if(request!=playlistGeneration||page!="songs"||detailRequest!=source)return@call
            detailLoadingMore=false
            if(!result.optBoolean("ok")){
                detailHasMore=false
                detailError=displayError(result.optString("error",context.getString(R.string.request_retry)))
                return@call
            }
            val payload=result.optJSONObject("data")?:return@call
            val more=payload.optJSONArray("songs")?.objects().orEmpty()
            if(more.isNotEmpty())tracks=tracks+more
            detailTotal=payload.optInt("total",detailTotal.coerceAtLeast(tracks.size))
            detailHasMore=payload.optBoolean("hasMore")&&more.isNotEmpty()
            detailRevision++
        }
    }
    /** Resolve every page of the open playlist, then start playback or shuffle. */
    fun playAllTracks(shuffle:Boolean){
        val source=detailRequest
        if(source==null||page!="songs"||source.kind!="playlist"||!detailHasMore){
            startTracks(tracks,shuffle);return
        }
        if(busy)return
        busy=true;status=context.getString(R.string.loading_playlist)
        val request=playlistGeneration
        val args=JSONObject().put("session",session).put("id",source.id).put("dirid",source.dirid)
        call("playlist_all",args){result->
            if(request!=playlistGeneration||page!="songs"||detailRequest!=source){busy=false;status="";return@call}
            busy=false;status=""
            if(!result.optBoolean("ok")){
                detailError=displayError(result.optString("error",context.getString(R.string.request_retry)))
                return@call
            }
            val songs=result.optJSONArray("data")?.objects().orEmpty()
            if(songs.isNotEmpty()){
                tracks=songs;detailTotal=songs.size;detailHasMore=false;detailRevision++
            }
            startTracks(if(songs.isNotEmpty())songs else tracks,shuffle)
        }
    }
    private fun startTracks(rows:List<JSONObject>,shuffle:Boolean){
        if(rows.isEmpty())return
        if(shuffle){choosePlaybackMode("SHUFFLE");play(rows,rows.indices.random())}else play(rows,0)
    }
    fun retryPlaylist(){if(!detailLoading)detailRequest?.let(::loadDetail)}
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
        playbackStatus=status
        if(PlayerService.snapshot.mid.isNotBlank())command("SELECT")
        call("play_url",JSONObject().put("song",song).put("session",session).put("quality",preferred).put("fallback",fallback)){result->
            if(request!=playGeneration)return@call
            if(preferred!=audioQuality||fallback!=qualityFallback){play(songs,index);return@call}
            if(failed(result)){pendingMid=null;pendingToken=null;playbackStatus=status;return@call}
            queue=songs.toList();prefs.edit().putString("active_queue",JSONArray(songs).toString()).apply()
            command("PLAY","quality",result.getJSONObject("data").optString("quality"),"downgraded",result.getJSONObject("data").optBoolean("downgraded").toString(),"token",token,"url",result.getJSONObject("data").getString("url"),"song",song.toString(),"lyric","","queue",JSONArray(songs).toString(),"session",session.toString(),"index",index)
            title=song.optString("title");artist=song.optString("artist");cover=song.artwork();status=""
            playbackStatus=""
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
        sleepRemainingSeconds=state.sleepRemainingSeconds;sleepAfterTrack=state.sleepAfterTrack;sleepFinished=state.sleepFinished;sleepMinutes=state.sleepMinutes
        if(pendingToken!=null){if(state.token!=pendingToken)return else {pendingMid=null;pendingToken=null}}
        if(state.mid.isBlank()){
            if(lastLiveMid.isNotBlank()||savedPlaybackDirty){savedPlayback=PlaybackStore.read(context);savedPlaybackDirty=false;lastLiveMid="";appliedQueueJson=""}
            savedPlayback?.let{saved->
                mid=saved.song.optString("mid");title=saved.song.optString("title");artist=saved.song.optString("artist");cover=saved.song.artwork()
                queue=saved.queue;currentIndex=saved.index;position=saved.position;duration=saved.duration
            }?:run{mid="";title="";artist="";cover="";currentIndex=-1;position=0;duration=0}
            lyrics="";translation=""
            playing=false;actualQuality="";qualityPending=false;qualityDowngraded=false
            return
        }
        lastLiveMid=state.mid
        if(appliedQueueJson!=state.queueJson){queue=runCatching{JSONArray(state.queueJson).objects()}.getOrDefault(emptyList());appliedQueueJson=state.queueJson}
        currentIndex=state.index
        mid=state.mid;title=state.title;artist=state.artist;cover=state.artwork
        playing=state.playing;position=state.position;duration=state.duration
        lyrics=state.lyric;translation=state.translation
        actualQuality=state.quality;qualityPending=state.qualityPending;qualityDowngraded=state.downgraded
        val serviceError=displayError(state.error)
        if(serviceError.isNotBlank()){status=serviceError;playbackStatus=serviceError}else if(lastServiceError.isNotBlank()){
            if(status==lastServiceError)status=""
            if(playbackStatus==lastServiceError)playbackStatus=""
        }
        lastServiceError=serviceError
    }
    fun setSleepTimer(minutes:Int){
        if(mid.isBlank())return
        command("SLEEP","minutes",minutes)
    }
    fun clearRecentPlays(){ListeningHistory.clear(context);recentPlays=emptyList()}
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
        it.optString("kind")=="guess"||name.contains("猜你喜欢")&&name.contains("沉浸刷歌")||(home.optLong("dailyId")>0&&it.optLong("id")==home.optLong("dailyId"))||name.contains("每日30首")||name.contains("每日三十首")
    }
    fun favorite():Boolean=if(pendingToken!=null)false else currentSong()?.let{song->favorites.any{it.optString("mid")==song.optString("mid")}}?:false
    fun toggleFavorite(){
        if(pendingToken!=null)return
        val song=currentSong()?:return
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
    private fun poll(generation:Int,sig:String,networkRetries:Int=0){
        if(generation!=loginGeneration||!qrOpen)return
        call("qr_poll",JSONObject().put("qrsig",sig)){result->
            if(generation!=loginGeneration||!qrOpen)return@call
            if(!result.optBoolean("ok")){
                val error=result.optString("error",context.getString(R.string.request_retry))
                val transient=error.contains("无法解析")||error.contains("resolve",true)||error.contains("lookup address",true)||error.contains("超时")||error.contains("timeout",true)
                if(transient&&networkRetries<3){
                    qrStatus=context.getString(R.string.qr_network_retry)
                    ui.postDelayed({poll(generation,sig,networkRetries+1)},2000L*(networkRetries+1))
                }else{status=displayError(error);qrStatus=status}
                return@call
            }
            val data=result.getJSONObject("data")
            when(data.optString("status")){
                "success"->{session=data.getJSONObject("session");secure.save(session.toString());loggedIn=true;dismissLogin();navigate("mine");refreshLibrary();refreshProfile();refreshHome()}
                "expired","canceled"->qrStatus=context.getString(R.string.qr_expired)
                else->{qrStatus=if(data.optString("status")=="confirming")context.getString(R.string.confirm_qq)else context.getString(R.string.waiting_scan);ui.postDelayed({poll(generation,sig)},1800)}
            }
        }
    }
    fun dismissLogin(){loginGeneration++;qrOpen=false}
    fun logout(){catalogPrefs.edit().clear().apply();cancelDetail();homeGeneration++;feedGeneration++;discoverySongs=emptyList();discoveryLoading=false;discoveryMore=true;discoveryError="";collectionBusy=false;collectionMessage="";epoch++;loginGeneration++;playGeneration++;pendingMid=null;pendingToken=null;backStack.clear();secure.clear();session=JSONObject();profile=JSONObject();loggedIn=false;home=JSONObject();library=JSONObject();page="home";status=context.getString(R.string.logged_out);playbackStatus="";command("STOP")}
    fun close(){prefs.unregisterOnSharedPreferenceChangeListener(historyListener);playbackPrefs.unregisterOnSharedPreferenceChangeListener(playbackListener);alive=false;loginGeneration++;worker.shutdownNow();feedWorker.shutdownNow();detailWorker.shutdownNow();ui.removeCallbacksAndMessages(null)}
}
