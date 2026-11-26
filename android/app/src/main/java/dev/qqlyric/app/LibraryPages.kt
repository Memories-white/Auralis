package dev.qqlyric.app

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.*
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import top.yukonga.miuix.kmp.basic.*
import top.yukonga.miuix.kmp.icon.MiuixIcons
import top.yukonga.miuix.kmp.icon.extended.*
import top.yukonga.miuix.kmp.theme.MiuixTheme
import top.yukonga.miuix.kmp.utils.PressFeedbackType

fun LazyListScope.musicLibrary(m:MusicState,showSleep:()->Unit){
    item("profile"){
        Card(Modifier.padding(horizontal=12.dp,vertical=8.dp).fillMaxWidth()){
            Row(Modifier.padding(20.dp),verticalAlignment=Alignment.CenterVertically,horizontalArrangement=Arrangement.spacedBy(16.dp)){
                Artwork(m.avatar,stringResource(R.string.avatar),Modifier.size(56.dp).clip(CircleShape),true)
                Column(Modifier.weight(1f),verticalArrangement=Arrangement.spacedBy(6.dp)){
                    Text(if(m.loggedIn)m.profile.optString("nickname").takeIf{it.isNotBlank()&&it!="null"}?:stringResource(R.string.qq_user)else stringResource(R.string.music_with_you),style=MiuixTheme.textStyles.title3,maxLines=2,overflow=TextOverflow.Ellipsis)
                    Text(if(m.loggedIn)stringResource(R.string.account_identifier,m.uin)else stringResource(R.string.login_library_summary),style=MiuixTheme.textStyles.footnote1,color=MiuixTheme.colorScheme.onSurfaceVariantSummary)
                }
            }
            if(!m.loggedIn)TextButton(stringResource(R.string.login_qq),m::login,Modifier.padding(horizontal=20.dp).padding(bottom=16.dp).fillMaxWidth())
        }
    }
    item("library-collections"){
        val liked=stringResource(R.string.liked_music)
        val local=stringResource(R.string.local_favorites)
        val likes=m.library.optJSONArray("likes")?.objects().orEmpty()
        Column(Modifier.padding(horizontal=12.dp,vertical=8.dp),verticalArrangement=Arrangement.spacedBy(12.dp)){
            Row(horizontalArrangement=Arrangement.spacedBy(12.dp)){
                LibraryTile(liked,if(m.loggedIn)stringResource(R.string.loaded_songs,likes.size)else stringResource(R.string.login_library_short),{if(m.loggedIn)m.openSongs(likes,liked)else m.login()},Modifier.weight(1f)){Icon(MiuixIcons.Favorites,null,tint=MiuixTheme.colorScheme.primary)}
                LibraryTile(local,stringResource(R.string.song_count,m.favorites.size),{m.openSongs(m.favorites,local)},Modifier.weight(1f)){Icon(MiuixIcons.Music,null,tint=MiuixTheme.colorScheme.primary)}
            }
            Row(horizontalArrangement=Arrangement.spacedBy(12.dp)){
                LibraryTile(stringResource(R.string.recent_plays),stringResource(R.string.song_count,m.recentPlays.size),{m.navigate("recent")},Modifier.weight(1f)){TimerGlyph(stringResource(R.string.recent_plays))}
                LibraryTile(stringResource(R.string.sleep_timer),sleepTimerSummary(m),showSleep,Modifier.weight(1f)){TimerGlyph(stringResource(R.string.sleep_timer))}
            }
        }
    }
    if(m.recentPlays.isNotEmpty()){
        item("library-recent-title"){Row(Modifier.fillMaxWidth(),verticalAlignment=Alignment.CenterVertically){Box(Modifier.weight(1f)){MusicSectionTitle(stringResource(R.string.recent_plays))};TextButton(stringResource(R.string.view_all),{m.navigate("recent")},Modifier.padding(end=12.dp))}}
        itemsIndexed(m.recentPlays.take(3),key={_,s->"library-recent:${s.optString("mid")}"}){i,song->SongRow(song,playingSong(m,song)){m.play(m.recentPlays,i)}}
    }
    item("library-playlists-title"){SectionHeader(stringResource(R.string.my_playlists),m::refreshLibrary)}
    val playlists=m.library.optJSONArray("playlists")?.objects().orEmpty()
    if(playlists.isEmpty())item("library-playlists-empty"){EmptyMessage(stringResource(R.string.library_no_playlists),stringResource(if(m.loggedIn)R.string.library_refresh_hint else R.string.login_library_summary))}
    itemsIndexed(playlists,key={i,s->"library-playlist:${s.optLong("id")}:$i"}){_,song->
        Row(Modifier.padding(horizontal=12.dp).fillMaxWidth().clip(RoundedCornerShape(16.dp)).clickable(role=Role.Button){m.playlist(song.optLong("id"),song.optLong("dirid"),song.optString("title"),song.optString("cover"))}.padding(12.dp),verticalAlignment=Alignment.CenterVertically,horizontalArrangement=Arrangement.spacedBy(14.dp)){
            Artwork(song.optString("cover"),song.optString("title"),Modifier.size(64.dp).clip(RoundedCornerShape(12.dp)))
            Column(Modifier.weight(1f),verticalArrangement=Arrangement.spacedBy(6.dp)){
                Text(song.optString("title"),style=MiuixTheme.textStyles.body1,maxLines=2,overflow=TextOverflow.Ellipsis)
                Text(stringResource(if(song.optBoolean("saved"))R.string.saved_playlist_count else R.string.created_playlist_count,song.optInt("count")),style=MiuixTheme.textStyles.footnote1,color=MiuixTheme.colorScheme.onSurfaceVariantSummary)
            }
            Icon(MiuixIcons.ChevronForward,null,Modifier.size(18.dp),tint=MiuixTheme.colorScheme.onSurfaceVariantSummary)
        }
    }
}

@Composable private fun LibraryTile(title:String,summary:String,onClick:()->Unit,modifier:Modifier,icon:@Composable ()->Unit){
    Card(modifier,onClick=onClick,pressFeedbackType=PressFeedbackType.Sink){
        Column(Modifier.fillMaxWidth().heightIn(min=116.dp).padding(16.dp),verticalArrangement=Arrangement.spacedBy(8.dp)){
            Box(Modifier.size(24.dp)){icon()}
            Text(title,style=MiuixTheme.textStyles.body1,maxLines=1,overflow=TextOverflow.Ellipsis)
            Text(summary,style=MiuixTheme.textStyles.footnote2,color=MiuixTheme.colorScheme.onSurfaceVariantSummary,maxLines=2,overflow=TextOverflow.Ellipsis)
        }
    }
}

fun LazyListScope.recentListening(m:MusicState,clear:()->Unit){
    item("recent-overview"){
        Card(Modifier.padding(12.dp).fillMaxWidth()){
            Column(Modifier.padding(20.dp),verticalArrangement=Arrangement.spacedBy(12.dp)){
                Text(stringResource(R.string.recent_count,m.recentPlays.size),style=MiuixTheme.textStyles.title3)
                Text(stringResource(R.string.recent_privacy),style=MiuixTheme.textStyles.footnote1,color=MiuixTheme.colorScheme.onSurfaceVariantSummary)
                TextButton(stringResource(R.string.clear_recent),clear,enabled=m.recentPlays.isNotEmpty())
            }
        }
    }
    if(m.recentPlays.isEmpty())item("recent-empty"){EmptyMessage(stringResource(R.string.recent_empty),stringResource(R.string.recent_empty_summary))}
    itemsIndexed(m.recentPlays,key={_,s->"recent:${s.optString("mid")}"}){i,song->SongRow(song,playingSong(m,song)){m.play(m.recentPlays,i)}}
}

@Composable fun PlaylistOverview(m:MusicState){
    Card(Modifier.padding(horizontal=12.dp,vertical=8.dp).fillMaxWidth()){
        Row(Modifier.padding(20.dp),verticalAlignment=Alignment.CenterVertically,horizontalArrangement=Arrangement.spacedBy(16.dp)){
            Artwork(m.detailCover,m.detailTitle,Modifier.size(80.dp).clip(RoundedCornerShape(14.dp)))
            Column(Modifier.weight(1f),verticalArrangement=Arrangement.spacedBy(10.dp)){
                Text(if(m.detailLoading)stringResource(R.string.loading_playlist)else if(m.detailError.isNotBlank())stringResource(R.string.playlist_load_failed)else stringResource(R.string.loaded_songs,m.tracks.size),style=MiuixTheme.textStyles.title3)
                Text(stringResource(if(m.detailLoading)R.string.playlist_loading_hint else if(m.detailError.isNotBlank())R.string.playlist_retry_hint else R.string.playlist_ready),style=MiuixTheme.textStyles.footnote1,color=MiuixTheme.colorScheme.onSurfaceVariantSummary)
            }
        }
        if(m.detailLoading)Row(Modifier.fillMaxWidth().padding(horizontal=20.dp).padding(bottom=20.dp).semantics{liveRegion=LiveRegionMode.Polite},verticalAlignment=Alignment.CenterVertically,horizontalArrangement=Arrangement.spacedBy(12.dp)){
            InfiniteProgressIndicator(color=MiuixTheme.colorScheme.primary,modifier=Modifier.semantics{progressBarRangeInfo=ProgressBarRangeInfo.Indeterminate})
            Text(stringResource(R.string.playlist_loading_hint),style=MiuixTheme.textStyles.footnote1,color=MiuixTheme.colorScheme.onSurfaceVariantSummary)
        }
        if(m.detailError.isNotBlank())Column(Modifier.fillMaxWidth().padding(horizontal=20.dp).padding(bottom=20.dp).semantics{liveRegion=LiveRegionMode.Polite},verticalArrangement=Arrangement.spacedBy(12.dp)){
            Text(m.detailError,style=MiuixTheme.textStyles.footnote1,color=MiuixTheme.colorScheme.onSurfaceVariantSummary)
            TextButton(stringResource(R.string.retry),m::retryPlaylist)
        }
        Row(Modifier.fillMaxWidth().padding(horizontal=16.dp).padding(bottom=16.dp),horizontalArrangement=Arrangement.spacedBy(12.dp)){
            TextButton(stringResource(R.string.play_all),{m.play(m.tracks,0)},Modifier.weight(1f),enabled=m.tracks.isNotEmpty()&&!m.detailLoading)
            TextButton(stringResource(R.string.shuffle_play),{if(m.tracks.isNotEmpty()){m.choosePlaybackMode("SHUFFLE");m.play(m.tracks,m.tracks.indices.random())}},Modifier.weight(1f),enabled=m.tracks.isNotEmpty()&&!m.detailLoading)
        }
    }
}
