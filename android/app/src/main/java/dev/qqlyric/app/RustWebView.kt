package dev.qqlyric.app

import android.annotation.SuppressLint
import android.graphics.Bitmap
import android.net.Uri
import android.os.Build
import android.provider.Settings
import android.webkit.*
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.*
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.view.WindowCompat
import androidx.lifecycle.*
import kotlinx.coroutines.*
import org.json.JSONArray
import org.json.JSONObject
import top.yukonga.miuix.kmp.theme.MiuixTheme
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.util.concurrent.Executors

// Native bridge allowlist only; page composition and destinations live in Rust.
private val SETTINGS_ROUTES=setOf("settings","appearance","monet","monet-source","monet-style","monet-spec","quality","storage","about","ui-customize","ui-dock","ui-player","ui-list","ui-motion","lyrics-style","lyric-type","lyric-effect")

/** Only window/theme/system bridges live here. All visible application pages and
 * event/motion logic are compiled from the Rust ui crate into the bundled WASM module. */
@Composable fun RustApp(m:MusicState,openRequested:Boolean,onOpenHandled:()->Unit){
    val context=LocalContext.current
    val activity=context as MainActivity
    val lifecycle=androidx.lifecycle.compose.LocalLifecycleOwner.current.lifecycle
    val density=LocalDensity.current
    val fontScale=LocalConfiguration.current.fontScale
    val theme=rememberMusicTheme(m)
    val dark=when(m.colorMode){"light"->false;"dark"->true;else->isSystemInDarkTheme()}
    MiuixTheme(controller=theme){
        val colors=MiuixTheme.colorScheme
        val host=remember(m,context){RustHost(activity,m)}
        val top=with(density){WindowInsets.statusBars.union(WindowInsets.displayCutout).union(WindowInsets.captionBar).getTop(this).toDp().value.toInt()}
        val bottom=with(density){WindowInsets.navigationBars.getBottom(this).toDp().value.toInt()}
        val palette=JSONObject().apply{
            fun color(key:String,argb:Int){put(key,String.format("#%06X",argb and 0xffffff))}
            color("bg",colors.surface.toArgb());color("card",colors.surfaceContainer.toArgb())
            color("fg",colors.onSurface.toArgb());color("muted",colors.onSurfaceVariantSummary.toArgb())
            color("primary",colors.primary.toArgb());color("on-primary",colors.onPrimary.toArgb());color("border",colors.surfaceContainerHighest.toArgb())
        }.toString()
        SideEffect{
            host.palette=palette;host.safeTop=top;host.safeBottom=bottom
            host.settings.textZoom=(fontScale*100).toInt().coerceIn(75,200)
            host.setBackgroundColor(colors.surface.toArgb())
            WindowCompat.getInsetsController(activity.window,activity.window.decorView).apply{isAppearanceLightStatusBars=!dark;isAppearanceLightNavigationBars=!dark}
        }
        LaunchedEffect(m,lifecycle,host){lifecycle.repeatOnLifecycle(Lifecycle.State.STARTED){
            while(isActive){m.sample();host.publish();delay(if(m.playing)48 else 180)}
        }}
        LaunchedEffect(openRequested){if(openRequested){host.openPlayer();onOpenHandled()}}
        DisposableEffect(host,lifecycle){
            val observer=LifecycleEventObserver{_,event->when(event){Lifecycle.Event.ON_RESUME->{host.onResume();host.resumeTimers()};Lifecycle.Event.ON_PAUSE->{host.onPause();host.pauseTimers()};else->{}}}
            lifecycle.addObserver(observer)
            onDispose{lifecycle.removeObserver(observer);host.close()}
        }
        BackHandler{host.evaluateJavascript("window.auralisBack ? window.auralisBack() : false"){handled->if(handled!="true")activity.finish()}}
        AndroidView(factory={host},modifier=Modifier.fillMaxSize().windowInsetsPadding(WindowInsets.safeDrawing.only(WindowInsetsSides.Horizontal)))
    }
}

@SuppressLint("SetJavaScriptEnabled")
internal class RustHost(private val activity:MainActivity,private val music:MusicState):WebView(activity){
    var palette="{}"
    var safeTop=0
    var safeBottom=0
    private val uiStore=activity.getSharedPreferences("rust-appearance",android.content.Context.MODE_PRIVATE)
    private var uiPreferences=runCatching{JSONObject(uiStore.getString("value","{}").orEmpty())}.getOrDefault(JSONObject())
    private var ready=false
    private var closed=false
    private var requestedPlayer=false
    private var savedRefreshRate:Float?=null
    private fun animationRate(active:Boolean){
        if(Build.VERSION.SDK_INT>=35)setRequestedFrameRate(if(active)android.view.View.REQUESTED_FRAME_RATE_CATEGORY_HIGH else android.view.View.REQUESTED_FRAME_RATE_CATEGORY_DEFAULT)
        if(active&&savedRefreshRate==null){
            val attributes=activity.window.attributes
            savedRefreshRate=attributes.preferredRefreshRate
            @Suppress("DEPRECATION") val screen=if(Build.VERSION.SDK_INT>=30)activity.display else activity.windowManager.defaultDisplay
            val mode=screen?.mode
            val powerSave=(activity.getSystemService(android.content.Context.POWER_SERVICE) as android.os.PowerManager).isPowerSaveMode
            val rate=if(powerSave)60f else screen?.supportedModes?.filter{it.physicalWidth==mode?.physicalWidth&&it.physicalHeight==mode?.physicalHeight}?.maxOfOrNull{it.refreshRate}?:60f
            attributes.preferredRefreshRate=rate
            activity.window.attributes=attributes
        }else if(!active){
            savedRefreshRate?.let{rate->val attributes=activity.window.attributes;attributes.preferredRefreshRate=rate;activity.window.attributes=attributes}
            savedRefreshRate=null
        }
    }
    private var contentKeys:List<Any?>?=null
    private var lastPulse=""
    @Volatile private var cacheBytes=0L
    private val worker=Executors.newSingleThreadExecutor()
    init{
        settings.javaScriptEnabled=true
        settings.allowFileAccess=false;settings.allowContentAccess=false
        settings.domStorageEnabled=false;settings.databaseEnabled=false
        settings.blockNetworkLoads=true
        settings.cacheMode=WebSettings.LOAD_DEFAULT
        settings.setSupportZoom(false)
        overScrollMode=OVER_SCROLL_NEVER
        addJavascriptInterface(Bridge(),"AuralisNative")
        if(BuildConfig.DEBUG)WebView.setWebContentsDebuggingEnabled(true)
        webViewClient=object:WebViewClient(){
            override fun shouldOverrideUrlLoading(view:WebView,request:WebResourceRequest)=true
            override fun shouldInterceptRequest(view:WebView,request:WebResourceRequest):WebResourceResponse=resource(request.url)
        }
        webChromeClient=object:WebChromeClient(){
            override fun onConsoleMessage(message:ConsoleMessage):Boolean{
                if(BuildConfig.DEBUG&&message.messageLevel()==ConsoleMessage.MessageLevel.ERROR)android.util.Log.e("AuralisUI",message.message().take(500))
                return true
            }
        }
        worker.execute{cacheBytes=java.io.File(activity.cacheDir,"artwork-v1").listFiles()?.sumOf{it.length()}?:0L}
        loadUrl("https://auralis.local/index.html")
    }
    fun openPlayer(){requestedPlayer=true;if(ready){send("open_player",JSONObject());requestedPlayer=false}}
    fun close(){animationRate(false);closed=true;worker.shutdownNow();removeJavascriptInterface("AuralisNative");stopLoading();destroy()}
    private fun empty()=WebResourceResponse("text/plain","UTF-8",404,"Not Found",mapOf("Cache-Control" to "no-store"),ByteArrayInputStream(ByteArray(0)))
    private fun resource(uri:Uri):WebResourceResponse{
        if(uri.scheme!="https"||uri.host!="auralis.local")return empty()
        return runCatching{
            when(uri.path){
                "/artwork"->{
                    val url=uri.getQueryParameter("url").orEmpty()
                    val avatar=uri.getQueryParameter("avatar")=="true"
                    val bytes=ArtworkCache.loadEncoded(activity,url,avatar)?:return empty()
                    WebResourceResponse("image/webp",null,200,"OK",mapOf("Cache-Control" to "private,max-age=86400"),ByteArrayInputStream(bytes))
                }
                else->{
                    val path=uri.path.orEmpty().removePrefix("/")
                    if(path.contains("..")||path.contains('\\'))return empty()
                    val mime=when{path.endsWith(".wasm")->"application/wasm";path.endsWith(".js")->"text/javascript";path.endsWith(".html")->"text/html";else->return empty()}
                    WebResourceResponse(mime,if(mime=="application/wasm")null else "UTF-8",200,"OK",mapOf("Cache-Control" to "no-store"),activity.assets.open("ui/$path"))
                }
            }
        }.getOrElse{empty()}
    }
    private fun source(id:String):List<JSONObject> =when(id){
        "discovery"->music.discoverySongs;"tracks"->music.tracks;"recent"->music.recentPlays
        "favorites"->music.favorites;"likes"->music.library.optJSONArray("likes")?.objects().orEmpty()
        "playlists"->music.library.optJSONArray("playlists")?.objects().orEmpty();"recommendations"->music.recommendations()
        "search"->music.searchResults;"queue"->music.queue;else->emptyList()
    }
    private fun clean(song:JSONObject):JSONObject=JSONObject().apply{
        listOf("mid","id","title","artist","album","albumMid","coverUrl","duration","vip").forEach{key->if(song.has(key))put(key,song.get(key))}
    }
    private fun packet(type:String,state:JSONObject)=JSONObject().put("type",type).put("state",state).toString()
    private fun send(type:String,state:JSONObject){if(ready&&!closed)evaluateJavascript("window.auralisUpdate(${JSONObject.quote(packet(type,state))})",null)}
    fun publish(){
        if(!ready||closed)return
        val m=music
        val keys=listOf(m.page,m.home,m.library,m.discoverySongs,m.tracks,m.favorites,m.recentPlays,m.searchResults,m.searchHistory,m.queue,m.profile,m.mid,
            m.loggedIn,m.avatar,m.uin,m.detailTitle,m.detailCover,m.detailLoading,m.detailError,m.status,m.busy,m.discoveryLoading,m.discoveryError,m.discoveryMore,
            m.detailTotal,m.detailHasMore,m.detailLoadingMore,
            m.searchText,m.searchedQuery,m.monetEnabled,m.monetColor,m.monetSource,m.monetStyle,m.monetSpec,m.colorMode,m.audioQuality,m.qualityFallback,
            m.qrOpen,m.qrImage,m.qrStatus,m.lyrics,m.translation,m.wordLyrics,m.collectionBusy,m.collectionMessage,uiPreferences.toString(),palette,safeTop,safeBottom,cacheBytes)
        if(keys!=contentKeys){
            contentKeys=keys
            val artists=m.discoverySongs.map{it.optString("artist")}.filter{it.isNotBlank()}.distinct().take(8)
            val sources=listOf("discovery","tracks","recent","favorites","likes","search","queue","playlists","recommendations")
            val state=JSONObject().apply{
                put("page",m.page);put("loggedIn",m.loggedIn);put("nickname",m.profile.optString("nickname"));put("avatar",m.avatar);put("uin",m.uin)
                put("homeReady",m.home.length()>0);put("dailyCover",m.home.optJSONArray("daily")?.optJSONObject(0)?.artwork().orEmpty())
                put("libraryReady",m.library.length()>0)
                put("detailTitle",m.detailTitle);put("detailCover",m.detailCover);put("detailLoading",m.detailLoading);put("detailError",m.detailError)
                put("detailTotal",m.detailTotal);put("detailHasMore",m.detailHasMore);put("detailLoadingMore",m.detailLoadingMore)
                put("query",m.searchText);put("searchedQuery",m.searchedQuery);put("busy",m.busy);put("status",m.status)
                put("discoveryLoading",m.discoveryLoading);put("discoveryError",m.discoveryError);put("discoveryMore",m.discoveryMore)
                put("colorMode",m.colorMode);put("monet",m.monetEnabled);put("monetSource",m.monetSource);put("monetStyle",m.monetStyle);put("monetSpec",m.monetSpec);put("monetColor",m.monetColor)
                put("uiPreferences",uiPreferences)
                put("quality",m.audioQuality);put("qualityFallback",m.qualityFallback);put("version",BuildConfig.VERSION_NAME)
                put("wallpaperSupported",Build.VERSION.SDK_INT>=31);put("artworkAvailable",m.cover.isNotBlank())
                put("qrOpen",m.qrOpen);put("qrStatus",m.qrStatus);put("qrImage",m.qrImage)
                put("lyrics",m.lyrics);put("translation",m.translation);put("wordLyrics",m.wordLyrics);put("collectionBusy",m.collectionBusy);put("collectionMessage",m.collectionMessage)
                put("currentSong",m.currentSong()?.let(::clean)?:JSONObject())
                put("history",JSONArray(m.searchHistory));put("artists",JSONArray(artists));put("colors",JSONObject(palette));put("safeTop",safeTop);put("safeBottom",safeBottom);put("cacheBytes",cacheBytes)
                put("reducedMotion",Settings.Global.getFloat(activity.contentResolver,Settings.Global.ANIMATOR_DURATION_SCALE,1f)==0f)
                val counts=JSONObject().put("history",m.searchHistory.size).put("artists",artists.size)
                sources.forEach{id->val rows=source(id);put(id,JSONArray(rows.map{if(id=="playlists"||id=="recommendations")it else clean(it)}));counts.put(id,rows.size)}
                // The header shows how many songs the playlist really has, not how many
                // pages have been fetched so far.
                if(m.page=="songs"&&m.detailTotal>0)counts.put("tracks",m.detailTotal)
                val likedTotal=m.library.optInt("total",0)
                if(likedTotal>counts.optInt("likes",0))counts.put("likes",likedTotal)
                put("counts",counts)
            }
            send("content",state)
        }
        val pulse=JSONObject().apply{
            put("mid",m.mid);put("title",m.title);put("artist",m.artist);put("cover",m.cover)
            put("position",m.position);put("duration",m.duration);put("playing",m.playing);put("mode",m.playbackMode)
            put("actualQuality",m.actualQuality);put("qualityPending",m.qualityPending);put("favorite",m.favorite());put("playbackStatus",m.playbackStatus)
            put("sleepRemaining",m.sleepRemainingSeconds);put("sleepAfterTrack",m.sleepAfterTrack);put("sleepMinutes",m.sleepMinutes)
        }
        if(pulse.toString()!=lastPulse){lastPulse=pulse.toString();send("playback",pulse)}
        if(requestedPlayer){send("open_player",JSONObject());requestedPlayer=false}
    }
    private inner class Bridge{
        @JavascriptInterface fun postMessage(input:String){
            if(input.length>65536||closed)return
            val event=runCatching{JSONObject(input)}.getOrNull()?:return
            post{if(!closed)handle(event)}
        }
    }
    private fun handle(event:JSONObject){
        val m=music
        when(event.optString("type")){
            "animation_rate"->animationRate(event.optBoolean("active"))
            "ready"->{ready=true;contentKeys=null;lastPulse="";publish()}
            "navigate"->{val page=event.optString("page");if(page in setOf("home","mine","search")+SETTINGS_ROUTES)m.navigate(page)}
            "back"->m.back()
            "play"->{val rows=source(event.optString("source"));val requested=event.optInt("index",-1);val mid=event.optString("mid");val index=if(mid.isNotBlank())rows.indexOfFirst{it.optString("mid")==mid}else requested;if(index in rows.indices)m.play(rows,index)}
            "playlist"->{val item=JSONObject().put("id",event.optLong("id")).put("dirid",event.optLong("dirid")).put("title",event.optString("title")).put("cover",event.optString("cover")).put("kind",event.optString("kind"));if(event.optString("source")=="recommendations")m.recommendedPlaylist(item)else m.playlist(item.optLong("id"),item.optLong("dirid"),item.optString("title"),item.optString("cover"))}
            "open_collection"->{val kind=event.optString("source");when(kind){
                "likes"->m.playlist(0,201,activity.getString(R.string.liked_music))
                "favorites"->m.openSongs(source("favorites"),activity.getString(R.string.local_favorites))
            }}
            "load_more_tracks"->m.loadMoreTracks()
            "play_all"->m.playAllTracks(event.optBoolean("shuffle"))
            "transport"->{val action=event.optString("action");if(action in setOf("PAUSE","RESUME","PREVIOUS","NEXT"))m.command(action)}
            "seek"->m.command("SEEK","position",event.optInt("position").coerceIn(0,m.duration.coerceAtLeast(0)))
            "search"->{m.searchText=event.optString("query").take(200);m.search()}
            "login"->m.login();"dismiss_login"->m.dismissLogin();"logout"->m.logout()
            "daily"->m.daily();"refresh_home"->if(!m.busy)m.refreshHome();"refresh_library"->if(!m.busy)m.refreshLibrary()
            "retry_playlist"->m.retryPlaylist();"load_more"->m.loadMoreDiscovery();"clear_search"->m.clearSearchHistory();"clear_recent"->m.clearRecentPlays()
            "appearance"->{val value=event.optString("value");if(value in setOf("system","light","dark"))m.setAppearance(value)}
            "ui_preferences"->{event.optJSONObject("value")?.takeIf{it.toString().length<=4096}?.let{uiPreferences=it;uiStore.edit().putString("value",it.toString()).apply()}}
            "toggle_monet"->m.setMonet(!m.monetEnabled)
            "monet"->{val key=event.optString("key");val value=event.optString("value");if(key in setOf("source","style","spec")||key=="color"&&value.matches(Regex("[0-9A-Fa-f]{6}")))m.setMonetOption(key,value)}
            "quality"->m.chooseAudioQuality(event.optString("value"));"toggle_fallback"->m.allowQualityFallback(!m.qualityFallback)
            "favorite"->m.toggleFavorite();"cycle_mode"->m.cyclePlaybackMode()
            "prepare_actions"->{m.collectionMessage="";if(m.loggedIn&&m.library.length()==0)m.refreshLibrary()}
            "sleep"->{val minutes=event.optInt("minutes");if(minutes in setOf(-1,0,15,30,60,90))m.setSleepTimer(minutes)}
            "add_cloud"->{
                val song=m.currentSong()?.takeIf{event.optString("mid")==m.mid}
                if(song!=null)m.addCloudSong(song,event.optLong("dirid"))
                else m.collectionMessage=activity.getString(R.string.song_add_select_again)
            }
        }
        publish()
    }
}
