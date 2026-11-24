package dev.qqlyric.app

import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.spring
import androidx.compose.foundation.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.pager.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.CompositingStrategy
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.*
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import top.yukonga.miuix.kmp.basic.*
import top.yukonga.miuix.kmp.icon.MiuixIcons
import top.yukonga.miuix.kmp.icon.extended.*
import top.yukonga.miuix.kmp.theme.MiuixTheme

@Composable internal fun MusicPlayer(m:MusicState,close:()->Unit,queue:()->Unit,openQuality:()->Unit,showSleep:()->Unit,openActions:()->Unit,modifier:Modifier){
    val c=MiuixTheme.colorScheme
    val pager=rememberPagerState(initialPage=1,pageCount={3})
    val pageNames=listOf(stringResource(R.string.song_information),stringResource(R.string.cover_page),stringResource(R.string.lyrics_page))
    val timerActive=m.sleepRemainingSeconds>0||m.sleepAfterTrack
    Scaffold(modifier.fillMaxSize(),containerColor=c.surface,
        topBar={TopAppBar(expandable=false,modifier=Modifier.graphicsLayer{compositingStrategy=CompositingStrategy.Offscreen},title=stringResource(R.string.now_playing),navigationIcon={IconButton(onClick=close){TransportGlyph("down",stringResource(R.string.collapse_player))}},actions={
            IconButton(onClick=showSleep){Box{TimerGlyph(stringResource(R.string.sleep_timer));if(timerActive)Box(Modifier.align(Alignment.TopEnd).size(5.dp).background(c.primary,CircleShape))}}
            IconButton(onClick=queue){Icon(MiuixIcons.Playlist,stringResource(R.string.play_queue),tint=c.onSurface)}
        })},
        bottomBar={Column(Modifier.fillMaxWidth().graphicsLayer{compositingStrategy=CompositingStrategy.Offscreen}.navigationBarsPadding().padding(horizontal=24.dp,vertical=12.dp)){
            if(pager.currentPage!=2)ThinSeekBar(m.position,m.duration,{m.command("SEEK","position",it)},Modifier.fillMaxWidth())
            Row(Modifier.fillMaxWidth(),horizontalArrangement=Arrangement.SpaceBetween){Text(formatTime(m.position),style=MiuixTheme.textStyles.footnote2,color=c.onSurfaceVariantSummary);Text(formatTime(m.duration),style=MiuixTheme.textStyles.footnote2,color=c.onSurfaceVariantSummary)}
            Row(Modifier.fillMaxWidth().padding(top=8.dp),horizontalArrangement=Arrangement.SpaceEvenly,verticalAlignment=Alignment.CenterVertically){
                IconButton(onClick=m::cyclePlaybackMode,minWidth=48.dp,minHeight=48.dp){TransportGlyph(when(m.playbackMode){"SHUFFLE"->"shuffle";"LOOP"->"repeat";"SINGLE"->"repeat_one";else->"order"},stringResource(playbackLabel(m.playbackMode)),if(m.playbackMode=="ORDER")c.onSurface else c.primary)}
                IconButton(onClick={m.command("PREVIOUS")}){TransportGlyph("previous",stringResource(R.string.previous))}
                IconButton(onClick={m.command(if(m.playing)"PAUSE"else"RESUME")},backgroundColor=c.primary,minWidth=68.dp,minHeight=68.dp,cornerRadius=22.dp){Icon(if(m.playing)MiuixIcons.Pause else MiuixIcons.Play,if(m.playing)stringResource(R.string.pause)else stringResource(R.string.play),tint=c.onPrimary,modifier=Modifier.size(28.dp))}
                IconButton(onClick={m.command("NEXT")}){TransportGlyph("next",stringResource(R.string.next))}
                IconButton(onClick=queue){Icon(MiuixIcons.Playlist,stringResource(R.string.play_queue))}
            }
            if(pager.currentPage==2)ThinSeekBar(m.position,m.duration,{m.command("SEEK","position",it)},Modifier.fillMaxWidth(),framed=true)
        }}){padding->
        Column(Modifier.fillMaxSize().padding(padding).consumeWindowInsets(padding)){
            Row(Modifier.fillMaxWidth().height(20.dp).semantics{contentDescription=pageNames[pager.currentPage]},horizontalArrangement=Arrangement.spacedBy(6.dp,Alignment.CenterHorizontally),verticalAlignment=Alignment.CenterVertically){
                repeat(3){index->val width by animateDpAsState(if(index==pager.currentPage)18.dp else 5.dp,spring(dampingRatio=0.9f,stiffness=450f),label="player page indicator");Box(Modifier.width(width).height(5.dp).background(if(index==pager.currentPage)c.primary else c.onSurfaceVariantSummary,CircleShape))}
            }
            if(pager.currentPage!=1)PlayerIdentity(m,openQuality)
            if(m.playbackStatus.isNotBlank())Text(m.playbackStatus,Modifier.padding(horizontal=28.dp,vertical=8.dp),style=MiuixTheme.textStyles.footnote1,color=c.onSurfaceVariantSummary)
            if(timerActive||m.sleepFinished)Text(sleepTimerSummary(m),Modifier.padding(horizontal=28.dp,vertical=6.dp),style=MiuixTheme.textStyles.footnote2,color=c.onSurfaceVariantSummary)
            HorizontalPager(pager,Modifier.weight(1f).fillMaxWidth()){page->
                when(page){
                    0->SongInformation(m)
                    1->Column(Modifier.fillMaxSize().padding(horizontal=28.dp).padding(top=12.dp,bottom=16.dp)){
                        BoxWithConstraints(Modifier.weight(1f).fillMaxWidth(),contentAlignment=Alignment.Center){
                            val coverSize=minOf(maxWidth,maxHeight,380.dp)
                            Artwork(m.cover,m.title,Modifier.size(coverSize).clip(RoundedCornerShape(20.dp)).clickable(role=Role.Button,onClickLabel=stringResource(R.string.song_actions),onClick=openActions))
                        }
                        Spacer(Modifier.height(20.dp))
                        Row(Modifier.fillMaxWidth(),verticalAlignment=Alignment.CenterVertically,horizontalArrangement=Arrangement.spacedBy(12.dp)){
                            Column(Modifier.weight(1f),verticalArrangement=Arrangement.spacedBy(8.dp)){
                                Text(m.title.ifBlank{stringResource(R.string.not_playing)},style=MiuixTheme.textStyles.title2,maxLines=2,overflow=TextOverflow.Ellipsis)
                                Text(m.artist.ifBlank{stringResource(R.string.player_pick_song)},style=MiuixTheme.textStyles.body2,color=c.onSurfaceVariantSummary,maxLines=2,overflow=TextOverflow.Ellipsis)
                            }
                            IconButton(onClick=m::toggleFavorite){Icon(if(m.favorite())MiuixIcons.FavoritesFill else MiuixIcons.Favorites,stringResource(R.string.local_favorites),tint=if(m.favorite())c.primary else c.onSurface)}
                        }
                        Row(Modifier.fillMaxWidth().padding(top=6.dp),verticalAlignment=Alignment.CenterVertically,horizontalArrangement=Arrangement.spacedBy(12.dp)){
                            Text(m.currentSong()?.optString("album").orEmpty(),Modifier.weight(1f),style=MiuixTheme.textStyles.footnote2,color=c.onSurfaceVariantSummary,maxLines=1,overflow=TextOverflow.Ellipsis)
                            QualityBadgeButton(m,openQuality)
                        }
                    }
                    else->SynchronizedLyrics(m,pager.currentPage==2)
                }
            }
        }
    }
}

@Composable private fun PlayerIdentity(m:MusicState,openQuality:()->Unit){
    Row(Modifier.fillMaxWidth().padding(horizontal=28.dp,vertical=12.dp),verticalAlignment=Alignment.CenterVertically,horizontalArrangement=Arrangement.spacedBy(12.dp)){
        Artwork(m.cover,m.title,Modifier.size(44.dp).clip(RoundedCornerShape(10.dp)))
        Column(Modifier.weight(1f),verticalArrangement=Arrangement.spacedBy(4.dp)){
            Text(m.title.ifBlank{stringResource(R.string.not_playing)},style=MiuixTheme.textStyles.title3,maxLines=1,overflow=TextOverflow.Ellipsis)
            Text(m.artist,style=MiuixTheme.textStyles.footnote1,color=MiuixTheme.colorScheme.onSurfaceVariantSummary,maxLines=1,overflow=TextOverflow.Ellipsis)
        }
        QualityBadgeButton(m,openQuality)
    }
}

@Composable private fun QualityBadgeButton(m:MusicState,onClick:()->Unit){
    Box(Modifier.heightIn(min=48.dp).clip(RoundedCornerShape(10.dp)).clickable(role=Role.Button,onClick=onClick).padding(horizontal=12.dp),contentAlignment=Alignment.Center){
        Text(if(m.qualityPending)stringResource(R.string.quality_switching)else qualityBadge(m.actualQuality,m.qualityDowngraded),style=MiuixTheme.textStyles.footnote1,color=MiuixTheme.colorScheme.primary)
    }
}
