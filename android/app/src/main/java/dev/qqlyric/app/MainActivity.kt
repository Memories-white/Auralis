package dev.qqlyric.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.*
import androidx.activity.enableEdgeToEdge
import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.*
import androidx.compose.foundation.gestures.animateScrollBy
import androidx.compose.foundation.interaction.collectIsDraggedAsState
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.*
import androidx.compose.foundation.pager.*
import androidx.compose.foundation.shape.*
import androidx.compose.foundation.text.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.*
import androidx.compose.ui.draw.*
import androidx.compose.ui.graphics.*
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.layout.boundsInRoot
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.platform.*
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.*
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.*
import androidx.core.view.WindowCompat
import androidx.lifecycle.repeatOnLifecycle
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.first
import org.json.JSONObject
import top.yukonga.miuix.kmp.basic.*
import top.yukonga.miuix.kmp.theme.*
import top.yukonga.miuix.kmp.icon.MiuixIcons
import top.yukonga.miuix.kmp.icon.extended.*
import top.yukonga.miuix.kmp.overlay.*
import top.yukonga.miuix.kmp.preference.*
import top.yukonga.miuix.kmp.utils.PressFeedbackType
import top.yukonga.miuix.kmp.utils.overScrollVertical

class MainActivity:ComponentActivity(){
    private lateinit var music:MusicState
    private var openRequested by mutableStateOf(false)
    override fun onCreate(savedInstanceState:Bundle?){
        super.onCreate(savedInstanceState);enableEdgeToEdge();music=MusicState(this)
        openRequested=intent.getBooleanExtra("open_player",false)
        setContent{Auralis(music,openRequested){openRequested=false}}
    }
    override fun onNewIntent(intent:android.content.Intent){super.onNewIntent(intent);setIntent(intent);openRequested=intent.getBooleanExtra("open_player",false)}
    override fun onDestroy(){music.close();super.onDestroy()}
}

@Composable private fun Auralis(m:MusicState,openRequested:Boolean,onOpenHandled:()->Unit){
    val theme=rememberMusicTheme(m)
    MiuixTheme(controller=theme){
        val c=MiuixTheme.colorScheme
        val activity=LocalContext.current as MainActivity
        val dark=when(m.colorMode){"light"->false;"dark"->true;else->isSystemInDarkTheme()}
        SideEffect{WindowCompat.getInsetsController(activity.window,activity.window.decorView).apply{isAppearanceLightStatusBars=!dark;isAppearanceLightNavigationBars=!dark}}
        val lifecycle=androidx.lifecycle.compose.LocalLifecycleOwner.current.lifecycle
        LaunchedEffect(m,lifecycle){lifecycle.repeatOnLifecycle(androidx.lifecycle.Lifecycle.State.STARTED){while(isActive){m.sample();delay(if(m.playing)32 else 180)}}}
        var player by rememberSaveable{mutableStateOf(false)}
        LaunchedEffect(openRequested){if(openRequested){player=true;onOpenHandled()}}
        var queue by remember{mutableStateOf(false)}
        var qualityMenu by remember{mutableStateOf(false)}
        var confirmLogout by remember{mutableStateOf(false)}
        var sleepMenu by remember{mutableStateOf(false)}
        var clearRecent by remember{mutableStateOf(false)}
        var songActions by remember{mutableStateOf(false)}
        var actionSong by remember{mutableStateOf<JSONObject?>(null)}
        LaunchedEffect(m.mid){songActions=false}
        val lists=remember{mutableMapOf<String,LazyListState>()}
        BackHandler(player||m.page!="home"){if(player)player=false else m.back()}
        var playerOrigin by remember { mutableStateOf(Rect.Zero) }
        val expansion = remember { Animatable(0f) }
        LaunchedEffect(player) { expansion.animateTo(if (player) 1f else 0f, spring(dampingRatio=0.86f, stiffness=420f)) }
        BoxWithConstraints(Modifier.fillMaxSize().background(c.surface)) {
            MusicShell(m,lists,Modifier.onGloballyPositioned { playerOrigin=it.boundsInRoot() },{player=true},{queue=true},{confirmLogout=true},{sleepMenu=true},{clearRecent=true})
            if(player || expansion.value > 0f) {
                val density=LocalDensity.current
                val width=with(density){maxWidth.toPx()};val height=with(density){maxHeight.toPx()}
                MusicPlayer(m,{player=false},{queue=true},{qualityMenu=true},{sleepMenu=true},{actionSong=m.currentSong()?.let{JSONObject(it.toString())};songActions=true},Modifier.graphicsLayer {
                    val p=expansion.value.coerceIn(0f,1f)
                    val origin=playerOrigin.takeIf{it.width>0f}?:Rect(0f,height*0.85f,width,height)
                    transformOrigin=TransformOrigin(0f,0f)
                    translationX=origin.left*(1-p);translationY=origin.top*(1-p)
                    scaleX=origin.width/width+(1-origin.width/width)*p
                    scaleY=origin.height/height+(1-origin.height/height)*p
                    alpha=p;clip=true;shape=RoundedCornerShape(16.dp*(1-p))
                })
            }
        }
        Scaffold(Modifier.fillMaxSize(),containerColor=Color.Transparent,contentWindowInsets=WindowInsets(0,0,0,0)){
            SleepTimerSheet(m,sleepMenu,{sleepMenu=false})
            OverlayDialog(show=clearRecent,title=stringResource(R.string.clear_recent),summary=stringResource(R.string.clear_recent_summary),onDismissRequest={clearRecent=false}){
                Row(Modifier.fillMaxWidth(),horizontalArrangement=Arrangement.spacedBy(20.dp)){
                    TextButton(stringResource(R.string.cancel),{clearRecent=false},Modifier.weight(1f))
                    TextButton(stringResource(R.string.clear_history),{m.clearRecentPlays();clearRecent=false},Modifier.weight(1f))
                }
            }
            SongActions(m,actionSong,songActions,{songActions=false})
            OverlayDialog(show=m.qrOpen,title=stringResource(R.string.login_qq),summary=m.qrStatus,onDismissRequest=m::dismissLogin){
                Column(Modifier.fillMaxWidth(),horizontalAlignment=Alignment.CenterHorizontally,verticalArrangement=Arrangement.spacedBy(16.dp)){
                    m.qrBitmap?.let{Image(it.asImageBitmap(),stringResource(R.string.login_qr),Modifier.size(220.dp))}
                    Row(horizontalArrangement=Arrangement.spacedBy(12.dp)){TextButton(stringResource(R.string.get_qr_again),m::login);TextButton(stringResource(R.string.cancel),m::dismissLogin)}
                }
            }
            OverlayDialog(show=confirmLogout,title=stringResource(R.string.logout_title),summary=stringResource(R.string.logout_summary),onDismissRequest={confirmLogout=false}){
                Row(Modifier.fillMaxWidth(),horizontalArrangement=Arrangement.spacedBy(12.dp)){TextButton(stringResource(R.string.cancel),{confirmLogout=false},Modifier.weight(1f));TextButton(stringResource(R.string.logout),{confirmLogout=false;m.logout()},Modifier.weight(1f))}
            }
            OverlayDialog(show=qualityMenu,title=stringResource(R.string.audio_quality),onDismissRequest={qualityMenu=false}){Column(Modifier.fillMaxWidth().heightIn(max=500.dp).verticalScroll(rememberScrollState())){QualityChoices(m)}}
            OverlayBottomSheet(show=queue,title=stringResource(R.string.play_queue_count,m.queue.size),onDismissRequest={queue=false}){
                LazyColumn(Modifier.fillMaxWidth().heightIn(max=480.dp)){itemsIndexed(m.queue){index,song->SongRow(song,index==m.currentIndex){queue=false;m.play(m.queue,index)}}}
            }
        }
    }
}

@Composable private fun MusicShell(m:MusicState,states:MutableMap<String,LazyListState>,playerSurface:Modifier,openPlayer:()->Unit,showQueue:()->Unit,logout:()->Unit,showSleep:()->Unit,clearRecent:()->Unit){
    val c=MiuixTheme.colorScheme
    val scroll=MiuixScrollBehavior()
    val list=states.getOrPut(m.page){LazyListState()}
    val scope=rememberCoroutineScope()
    val showTop by remember(list){derivedStateOf{list.firstVisibleItemIndex>=5}}
    val currentTrackIndex=m.tracks.indexOfFirst{it.optString("mid")==m.mid}
    LaunchedEffect(m.page,m.detailRevision){if(m.page=="songs")list.scrollToItem(0)}
    LaunchedEffect(m.page,list){
        if(m.page=="home")snapshotFlow{val info=list.layoutInfo;info.totalItemsCount>0&&(info.visibleItemsInfo.lastOrNull()?.index?:0)>=info.totalItemsCount-5}.collect{nearEnd->
            if(nearEnd&&m.home.length()>0&&m.discoveryError.isBlank())m.loadMoreDiscovery()
        }
    }
    // Opaque semantic chrome is the conservative fallback while static capture and
    // continuous recording disagree on this device. Keep this separate from lyric blur.
    val title=when(m.page){"mine"->stringResource(R.string.my_music);"recent"->stringResource(R.string.recent_plays);"search"->stringResource(R.string.search);"songs"->m.detailTitle;else->if(m.page in SETTINGS_PAGES)stringResource(settingsTitle(m.page))else "Auralis"}
    val secondary=m.page!="home"&&m.page!="mine"
    Scaffold(containerColor=c.surface,topBar={
        Box(Modifier.background(c.surface)){
            TopAppBar(title=title,expandable=LocalConfiguration.current.screenWidthDp<840,color=Color.Transparent,scrollBehavior=scroll,
                navigationIcon={if(secondary)IconButton(onClick=m::back){Icon(MiuixIcons.Back,stringResource(R.string.back))}},
                actions={
                    if(m.page=="home"||m.page=="mine")IconButton(onClick={m.navigate("search")}){Icon(MiuixIcons.Search,stringResource(R.string.search),tint=c.onSurface)}
                    if(m.page=="mine")IconButton(onClick={m.navigate("settings")}){Icon(MiuixIcons.Settings,stringResource(R.string.settings))}
                    if(m.page=="home"||m.page=="mine")IconButton(onClick={if(m.loggedIn)m.navigate("mine")else m.login()}){
                        if(m.loggedIn)Artwork(m.avatar,stringResource(R.string.my_account),Modifier.size(32.dp).clip(CircleShape),true)else Icon(MiuixIcons.Contacts,stringResource(R.string.login))
                    }
                })
        }
    },bottomBar={if(m.page !in SETTINGS_PAGES)Column(Modifier.background(c.surface).then(if(secondary)Modifier.navigationBarsPadding()else Modifier).graphicsLayer{compositingStrategy=CompositingStrategy.Offscreen}){
        MiniPlayer(m,playerSurface.padding(horizontal=12.dp,vertical=6.dp),openPlayer,showQueue)
        if(m.page=="home"||m.page=="mine")NavigationBar(showDivider=false){
            listOf(Triple("home",stringResource(R.string.home),MiuixIcons.Home),Triple("mine",stringResource(R.string.mine),MiuixIcons.Contacts)).forEach{(page,label,icon)->
                NavigationBarItem(selected=(if(m.page=="settings")"mine"else if(m.page=="songs")m.parentPage else m.page)==page,onClick={m.navigate(page)},icon=icon,label=label)
            }
        }
    }}){padding->
        Box(Modifier.fillMaxSize()){
        LazyColumn(state=list,modifier=Modifier.fillMaxSize()
            .nestedScroll(scroll.nestedScrollConnection).overScrollVertical(onOffsetChanged={scroll.state.overscrollOffset=it}).consumeWindowInsets(padding),
            contentPadding=PaddingValues(top=padding.calculateTopPadding(),bottom=padding.calculateBottomPadding()+20.dp)){
            if(m.status.isNotBlank())item("status"){Text(m.status,Modifier.padding(horizontal=28.dp,vertical=12.dp),color=c.onBackgroundVariant,style=MiuixTheme.textStyles.footnote1)}
            if(m.page in SETTINGS_PAGES)settingsContent(m,logout)
            when(m.page){
                "home"->homeDiscovery(m,showQueue)
                "search"->searchDiscovery(m)
                "mine"->musicLibrary(m,showSleep)
                "recent"->recentListening(m,clearRecent)
                "songs"->{
                    item("playlist-count"){PlaylistOverview(m)}
                    if(m.tracks.isEmpty()&&!m.detailLoading&&m.detailError.isBlank())item("songs-empty"){EmptyMessage(stringResource(R.string.no_songs),stringResource(R.string.retry_later))}
                    itemsIndexed(m.tracks,key={i,s->"song:${s.optString("mid")}:$i"}){i,song->SongRow(song,playingSong(m,song)){m.play(m.tracks,i)}}
                }
            }
        }
        if(m.page=="home"||m.page=="songs")Column(Modifier.align(Alignment.BottomEnd).padding(bottom=padding.calculateBottomPadding()+12.dp,end=20.dp),verticalArrangement=Arrangement.spacedBy(8.dp)){
            if(m.page=="songs"&&currentTrackIndex>=0)TextButton(stringResource(R.string.locate_playing),{scope.launch{list.animateScrollToItem(currentTrackIndex+1+if(m.status.isNotBlank())1 else 0)}})
            AnimatedVisibility(showTop,enter=fadeIn()+expandVertically(),exit=fadeOut()+shrinkVertically()){
                TextButton(stringResource(R.string.back_to_top),{scope.launch{list.animateScrollToItem(0)}})
            }
        }
        }
    }
}

@Composable internal fun MusicSectionTitle(title:String){Text(title,Modifier.padding(start=24.dp,end=16.dp,top=20.dp,bottom=12.dp).semantics{heading()},style=MiuixTheme.textStyles.title3,color=MiuixTheme.colorScheme.onSurface)}
@Composable internal fun SectionHeader(title:String,refresh:()->Unit){Row(Modifier.fillMaxWidth(),verticalAlignment=Alignment.CenterVertically){Box(Modifier.weight(1f)){MusicSectionTitle(title)};IconButton(onClick=refresh,modifier=Modifier.padding(end=16.dp,top=8.dp)){Icon(MiuixIcons.Refresh,stringResource(R.string.refresh),Modifier.size(22.dp),tint=MiuixTheme.colorScheme.onSurfaceVariantSummary)}}}
@Composable internal fun EmptyMessage(title:String,summary:String){Column(Modifier.fillMaxWidth().padding(28.dp,40.dp),verticalArrangement=Arrangement.spacedBy(8.dp)){Text(title,style=MiuixTheme.textStyles.title3);Text(summary,color=MiuixTheme.colorScheme.onSurfaceVariantSummary)}}
internal fun playingSong(m:MusicState,song:JSONObject)=m.mid.isNotBlank()&&m.mid==song.optString("mid")

@Composable internal fun Artwork(url:String,label:String,modifier:Modifier=Modifier,avatar:Boolean=false){
    val context=LocalContext.current
    val bitmap by produceState(ArtworkCache.peek(url,avatar),url,avatar){value=withContext(Dispatchers.IO){ArtworkCache.load(context,url,avatar)}}
    Box(modifier.background(MiuixTheme.colorScheme.surfaceContainer),contentAlignment=Alignment.Center){
        if(bitmap!=null)Image(bitmap!!.asImageBitmap(),label,Modifier.fillMaxSize(),contentScale=androidx.compose.ui.layout.ContentScale.Crop)
        else Icon(if(avatar)MiuixIcons.Contacts else MiuixIcons.Music,label,Modifier.size(28.dp),tint=MiuixTheme.colorScheme.onSurfaceVariantSummary)
    }
}
@Composable internal fun AlbumShelf(songs:List<JSONObject>,playlists:Boolean,onClick:(JSONObject,Int)->Unit){
    if(songs.isEmpty()){EmptyMessage(stringResource(R.string.recommendation_empty),stringResource(R.string.recommendation_empty_summary));return}
    LazyRow(contentPadding=PaddingValues(horizontal=24.dp),horizontalArrangement=Arrangement.spacedBy(16.dp)){
        itemsIndexed(songs.take(12),key={i,s->"${s.optString("mid",s.optString("id"))}:$i"}){i,song->
            Column(Modifier.width(156.dp).clip(RoundedCornerShape(16.dp)).clickable(role=androidx.compose.ui.semantics.Role.Button){onClick(song,i)}){
                Box(Modifier.fillMaxWidth().aspectRatio(1f)){
                    Artwork(if(playlists)song.optString("cover")else song.artwork(),song.optString("title"),Modifier.fillMaxSize().clip(RoundedCornerShape(16.dp)))
                    if(!playlists)Box(Modifier.align(Alignment.BottomEnd).padding(10.dp).size(32.dp).clip(CircleShape).background(MiuixTheme.colorScheme.primary),contentAlignment=Alignment.Center){Icon(MiuixIcons.Play,null,Modifier.size(18.dp),tint=MiuixTheme.colorScheme.onPrimary)}
                }
                Column(Modifier.padding(top=10.dp,bottom=8.dp),verticalArrangement=Arrangement.spacedBy(4.dp)){
                    Text(song.optString("title"),style=MiuixTheme.textStyles.body1,maxLines=2,minLines=2,overflow=TextOverflow.Ellipsis)
                    Text(if(playlists)stringResource(R.string.curated_playlist)else song.optString("artist"),style=MiuixTheme.textStyles.footnote1,color=MiuixTheme.colorScheme.onSurfaceVariantSummary,maxLines=1,overflow=TextOverflow.Ellipsis)
                }
            }
        }
    }
}
@Composable internal fun SongRow(song:JSONObject,selected:Boolean,onClick:()->Unit){
    val c=MiuixTheme.colorScheme
    Row(Modifier.padding(horizontal=12.dp).fillMaxWidth().clip(RoundedCornerShape(16.dp)).background(if(selected)c.surfaceContainer else Color.Transparent).clickable(role=androidx.compose.ui.semantics.Role.Button,onClick=onClick).padding(horizontal=12.dp,vertical=10.dp),verticalAlignment=Alignment.CenterVertically,horizontalArrangement=Arrangement.spacedBy(14.dp)){
        Artwork(song.artwork(),song.optString("title"),Modifier.size(52.dp).clip(RoundedCornerShape(10.dp)))
        Column(Modifier.weight(1f),verticalArrangement=Arrangement.spacedBy(5.dp)){
            Text(song.optString("title"),style=MiuixTheme.textStyles.body1,color=if(selected)c.primary else c.onSurface,maxLines=1,overflow=TextOverflow.Ellipsis)
            Text(song.optString("artist"),style=MiuixTheme.textStyles.footnote1,color=c.onSurfaceVariantSummary,maxLines=1,overflow=TextOverflow.Ellipsis)
        }
        if(selected)Icon(MiuixIcons.Play,stringResource(R.string.now_playing),Modifier.size(18.dp),tint=c.primary)
        else if(song.optBoolean("vip"))Text("VIP",style=MiuixTheme.textStyles.footnote2,color=c.onSurfaceVariantSummary)
    }
}
@Composable private fun MiniPlayer(m:MusicState,modifier:Modifier,open:()->Unit,queue:()->Unit){
    var expanded by remember { mutableStateOf(false) }
    var scrubbing by remember { mutableStateOf(false) }
    var interaction by remember { mutableIntStateOf(0) }
    LaunchedEffect(m.mid,m.playing,scrubbing,interaction) {
        if(scrubbing){expanded=true;return@LaunchedEffect}
        if(m.playing||expanded){expanded=true;delay(4500)}
        expanded=false
    }
    val progress by animateFloatAsState(if(expanded)1f else 0f,spring(dampingRatio=0.88f,stiffness=320f),label="song bar expansion")
    val p=progress.coerceIn(0f,1f)
    val c=MiuixTheme.colorScheme
    val transport:(String)->Unit={action->interaction++;m.command(action)}
    Card(modifier.fillMaxWidth(),onClick=open,pressFeedbackType=PressFeedbackType.Sink){
        Box(Modifier.fillMaxWidth().height((72f+128f*p).dp)){
            Row(Modifier.fillMaxWidth().height((60f+24f*p).dp).padding(horizontal=12.dp),verticalAlignment=Alignment.CenterVertically,horizontalArrangement=Arrangement.spacedBy(12.dp)){
                Artwork(m.cover,m.title,Modifier.size((44f+12f*p).dp).clip(RoundedCornerShape(10.dp)))
                Column(Modifier.weight(1f),verticalArrangement=Arrangement.spacedBy(4.dp)){
                    Text(m.title.ifBlank{stringResource(R.string.select_song)},style=MiuixTheme.textStyles.body1,maxLines=1,overflow=TextOverflow.Ellipsis)
                    if(m.artist.isNotBlank())Text(m.artist,style=MiuixTheme.textStyles.footnote2,color=c.onSurfaceVariantSummary,maxLines=1,overflow=TextOverflow.Ellipsis)
                }
                if(p<0.99f)Row(Modifier.width((96f*(1f-p)).dp).clipToBounds().graphicsLayer{alpha=1f-p},verticalAlignment=Alignment.CenterVertically){
                    IconButton(onClick={transport(if(m.playing)"PAUSE"else"RESUME")},enabled=p<0.1f,backgroundColor=c.primary){Icon(if(m.playing)MiuixIcons.Pause else MiuixIcons.Play,if(m.playing)stringResource(R.string.pause)else stringResource(R.string.play),tint=c.onPrimary)}
                    IconButton(onClick=queue,enabled=p<0.1f){Icon(MiuixIcons.Playlist,stringResource(R.string.play_queue))}
                }
                if(p>0.99f)IconButton(onClick={expanded=false}){TransportGlyph("down",stringResource(R.string.collapse_player))}
            }
            // One persistent seek node: its center travels from the compact bottom edge
            // to below the artwork row, while the footer grows underneath it.
            ThinSeekBar(m.position,m.duration,{m.command("SEEK","position",it);interaction++},
                Modifier.fillMaxWidth().padding(horizontal=16.dp).offset(y=(42f+42f*p).dp),
                expansion=p,interactive=p>0.95f,onScrubbing={scrubbing=it})
            if(p>0.01f)Column(Modifier.fillMaxWidth().offset(y=120.dp).padding(horizontal=20.dp).graphicsLayer{alpha=p}){
                Row(Modifier.fillMaxWidth(),horizontalArrangement=Arrangement.SpaceBetween){Text(formatTime(m.position),style=MiuixTheme.textStyles.footnote2,color=c.onSurfaceVariantSummary);Text(formatTime(m.duration),style=MiuixTheme.textStyles.footnote2,color=c.onSurfaceVariantSummary)}
                Row(Modifier.fillMaxWidth().padding(top=4.dp),horizontalArrangement=Arrangement.SpaceBetween,verticalAlignment=Alignment.CenterVertically){
                    IconButton(onClick={m.toggleFavorite();interaction++},enabled=p>0.95f){Icon(if(m.favorite())MiuixIcons.FavoritesFill else MiuixIcons.Favorites,stringResource(R.string.local_favorites))}
                    IconButton(onClick={transport("PREVIOUS")},enabled=p>0.95f){TransportGlyph("previous",stringResource(R.string.previous))}
                    IconButton(onClick={transport(if(m.playing)"PAUSE"else"RESUME")},enabled=p>0.95f,backgroundColor=c.primary,minWidth=48.dp,minHeight=48.dp){Icon(if(m.playing)MiuixIcons.Pause else MiuixIcons.Play,if(m.playing)stringResource(R.string.pause)else stringResource(R.string.play),tint=c.onPrimary)}
                    IconButton(onClick={transport("NEXT")},enabled=p>0.95f){TransportGlyph("next",stringResource(R.string.next))}
                    IconButton(onClick=queue,enabled=p>0.95f){Icon(MiuixIcons.Playlist,stringResource(R.string.play_queue))}
                }
            }
        }
    }
}
internal fun formatTime(ms:Int)="%02d:%02d".format(ms.coerceAtLeast(0)/60000,ms.coerceAtLeast(0)/1000%60)
@Composable internal fun TransportGlyph(kind:String,label:String,tint:Color=MiuixTheme.colorScheme.onSurface){
    val color=tint.toArgb()
    Canvas(Modifier.size(24.dp).semantics{contentDescription=label}){val icon=MusicIcon(kind,color);icon.setBounds(0,0,size.width.toInt(),size.height.toInt());icon.draw(drawContext.canvas.nativeCanvas)}
}
