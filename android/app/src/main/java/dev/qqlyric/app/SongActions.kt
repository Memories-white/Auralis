package dev.qqlyric.app

import androidx.compose.animation.AnimatedContent
import androidx.compose.foundation.*
import androidx.compose.foundation.layout.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import org.json.JSONObject
import top.yukonga.miuix.kmp.basic.*
import top.yukonga.miuix.kmp.overlay.OverlayBottomSheet
import top.yukonga.miuix.kmp.preference.ArrowPreference
import top.yukonga.miuix.kmp.theme.MiuixTheme

/** One sheet session, with internal destinations instead of stacked modal surfaces. */
@Composable fun SongActions(m:MusicState,song:JSONObject?,show:Boolean,onClose:()->Unit){
    var destination by remember{mutableStateOf("menu")}
    LaunchedEffect(show){if(show){destination="menu";m.collectionMessage="";if(m.loggedIn&&m.library.length()==0)m.refreshLibrary()}}
    val half=LocalConfiguration.current.screenHeightDp.dp*0.5f
    val loginRequired=stringResource(R.string.login_library_summary)
    OverlayBottomSheet(show=show,title=stringResource(when(destination){"quality"->R.string.audio_quality;"playlists"->R.string.add_to_playlist;else->R.string.song_actions}),
        onDismissRequest=onClose,modifier=Modifier.heightIn(max=half),
        startAction={if(destination!="menu")TextButton(stringResource(R.string.back),{destination="menu"})}){
        AnimatedContent(destination,label="song actions destination"){page->
            Column(Modifier.fillMaxWidth().height((half-100.dp).coerceAtLeast(180.dp)).verticalScroll(rememberScrollState())){
                if(page=="quality")QualityChoices(m)
                else {
                    song?.let{
                        Text(it.optString("title"),Modifier.padding(horizontal=20.dp,vertical=8.dp),style=MiuixTheme.textStyles.title3,maxLines=2,overflow=TextOverflow.Ellipsis)
                        Text(it.optString("artist"),Modifier.padding(horizontal=20.dp).padding(bottom=12.dp),style=MiuixTheme.textStyles.footnote1,color=MiuixTheme.colorScheme.onSurfaceVariantSummary)
                    }
                    if(m.collectionMessage.isNotBlank())Text(m.collectionMessage,Modifier.padding(horizontal=20.dp,vertical=12.dp),color=MiuixTheme.colorScheme.primary)
                    if(page=="menu"){
                        ArrowPreference(title=stringResource(R.string.audio_quality),summary=qualityBadge(m.actualQuality,m.qualityDowngraded),onClick={destination="quality"})
                        ArrowPreference(title=stringResource(if(song!=null&&m.cloudLiked(song))R.string.already_liked else R.string.add_to_likes),summary=stringResource(R.string.cloud_likes_summary),enabled=song!=null&&!m.collectionBusy,onClick={song?.let{m.addCloudSong(it,201)}})
                        ArrowPreference(title=stringResource(R.string.add_to_playlist),summary=stringResource(R.string.owned_playlist_summary),enabled=song!=null&&!m.collectionBusy,onClick={destination="playlists";if(m.loggedIn)m.refreshLibrary()else m.collectionMessage=loginRequired})
                    }else{
                        if(!m.loggedIn)Text(stringResource(R.string.login_library_summary),Modifier.padding(20.dp))
                        else if(m.writablePlaylists().isEmpty())Text(stringResource(R.string.no_writable_playlists),Modifier.padding(20.dp))
                        m.writablePlaylists().forEach{playlist->ArrowPreference(title=playlist.optString("title"),summary=stringResource(R.string.song_count,playlist.optInt("count")),enabled=!m.collectionBusy,onClick={song?.let{m.addCloudSong(it,playlist.optLong("dirid"))}})}
                    }
                }
            }
        }
    }
}

@Composable fun SongInformation(m:MusicState){
    val song=m.currentSong()
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(horizontal=28.dp,vertical=20.dp),verticalArrangement=Arrangement.spacedBy(16.dp)){
        Text(stringResource(R.string.song_information),style=MiuixTheme.textStyles.title2)
        listOf(R.string.song_title to m.title,R.string.song_artist to m.artist,R.string.song_album to song?.optString("album").orEmpty(),R.string.song_duration to "%02d:%02d".format(m.duration/60000,m.duration/1000%60),R.string.audio_quality to qualityBadge(m.actualQuality,m.qualityDowngraded)).forEach{(label,value)->
            Column(verticalArrangement=Arrangement.spacedBy(6.dp)){
                Text(stringResource(label),style=MiuixTheme.textStyles.footnote1,color=MiuixTheme.colorScheme.onSurfaceVariantSummary)
                Text(value.ifBlank{stringResource(R.string.information_unavailable)},style=MiuixTheme.textStyles.body1)
            }
        }
    }
}
