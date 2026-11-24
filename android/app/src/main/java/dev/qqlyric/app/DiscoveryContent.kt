package dev.qqlyric.app

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import top.yukonga.miuix.kmp.basic.*
import top.yukonga.miuix.kmp.icon.MiuixIcons
import top.yukonga.miuix.kmp.icon.extended.*
import top.yukonga.miuix.kmp.theme.MiuixTheme
import top.yukonga.miuix.kmp.utils.PressFeedbackType

/** Content only: the existing shell owns scroll, insets, navigation and the player. */
fun LazyListScope.homeDiscovery(m:MusicState,showQueue:()->Unit){
    val daily=m.home.optJSONArray("daily")?.objects().orEmpty()
    item("daily-feature"){
        Card(Modifier.padding(horizontal=12.dp,vertical=12.dp).fillMaxWidth(),onClick=m::daily,pressFeedbackType=PressFeedbackType.Sink){
            Row(Modifier.padding(20.dp),verticalAlignment=Alignment.CenterVertically,horizontalArrangement=Arrangement.spacedBy(16.dp)){
                Column(Modifier.weight(1f),verticalArrangement=Arrangement.spacedBy(10.dp)){
                    Text(stringResource(R.string.made_for_you),style=MiuixTheme.textStyles.footnote1,color=MiuixTheme.colorScheme.primary)
                    Text(stringResource(R.string.daily_mix),style=MiuixTheme.textStyles.title2)
                    Text(stringResource(if(m.loggedIn)R.string.daily_summary else R.string.daily_login_summary),style=MiuixTheme.textStyles.footnote2,color=MiuixTheme.colorScheme.onSurfaceVariantSummary,maxLines=2,overflow=TextOverflow.Ellipsis)
                }
                Box(Modifier.size(112.dp)){
                    Artwork(daily.firstOrNull()?.artwork().orEmpty(),stringResource(R.string.daily_cover),Modifier.fillMaxSize().clip(RoundedCornerShape(16.dp)))
                    IconButton(onClick={if(daily.isEmpty())m.daily()else m.play(daily,0)},modifier=Modifier.align(Alignment.BottomEnd),backgroundColor=MiuixTheme.colorScheme.primary,minWidth=48.dp,minHeight=48.dp){Icon(MiuixIcons.Play,stringResource(R.string.play_daily),tint=MiuixTheme.colorScheme.onPrimary)}
                }
            }
        }
    }
    item("discovery-shortcuts"){
        val favorites=stringResource(R.string.local_favorites)
        Row(Modifier.fillMaxWidth().padding(horizontal=12.dp).padding(bottom=12.dp),horizontalArrangement=Arrangement.spacedBy(12.dp)){
            Card(Modifier.weight(1f),onClick={m.openSongs(m.favorites,favorites)},pressFeedbackType=PressFeedbackType.Sink){Row(Modifier.heightIn(min=48.dp).padding(14.dp),verticalAlignment=Alignment.CenterVertically,horizontalArrangement=Arrangement.spacedBy(10.dp)){Icon(MiuixIcons.Favorites,null,Modifier.size(20.dp),tint=MiuixTheme.colorScheme.primary);Text(favorites,style=MiuixTheme.textStyles.footnote1)}}
            Card(Modifier.weight(1f),onClick=showQueue,pressFeedbackType=PressFeedbackType.Sink){Row(Modifier.heightIn(min=48.dp).padding(14.dp),verticalAlignment=Alignment.CenterVertically,horizontalArrangement=Arrangement.spacedBy(10.dp)){Icon(MiuixIcons.Playlist,null,Modifier.size(20.dp),tint=MiuixTheme.colorScheme.primary);Text(stringResource(R.string.play_queue),style=MiuixTheme.textStyles.footnote1)}}
        }
    }
    item("recommend-header"){SectionHeader(stringResource(R.string.recommended_songs),m::refreshHome)}
    item("songs-shelf"){if(m.discoverySongs.isNotEmpty())AlbumShelf(m.discoverySongs.take(6),false){_,i->m.play(m.discoverySongs,i)}else DiscoveryLoadState(m)}
    item("playlists-header"){MusicSectionTitle(stringResource(R.string.recommended_playlists))}
    item("playlists-shelf"){AlbumShelf(m.recommendations(),true){s,_->m.recommendedPlaylist(s)}}
    item("more-music"){MusicSectionTitle(stringResource(R.string.keep_discovering))}
    itemsIndexed(m.discoverySongs.drop(6),key={i,s->"discovery:${s.optString("mid")}:$i"}){i,s->SongRow(s,playingSong(m,s)){m.play(m.discoverySongs,i+6)}}
    item("discovery-load"){if(m.discoverySongs.isNotEmpty())DiscoveryLoadState(m)}
    if(!m.loggedIn)item("login-discovery"){EmptyMessage(stringResource(R.string.music_knows_you),stringResource(R.string.sync_qq_summary));TextButton(stringResource(R.string.login_qq),m::login,Modifier.padding(horizontal=12.dp).fillMaxWidth())}
}

fun LazyListScope.searchDiscovery(m:MusicState){
    val showResults=m.searchedQuery.isNotBlank()&&m.searchText.trim()==m.searchedQuery
    val daily=m.discoverySongs
    item("search-input"){
        val keyboard=LocalSoftwareKeyboardController.current
        val focus=androidx.compose.ui.platform.LocalFocusManager.current
        val submit={keyboard?.hide();focus.clearFocus();m.search()}
        TextField(value=m.searchText,onValueChange={m.searchText=it},label=stringResource(R.string.search_hint),useLabelAsPlaceholder=true,singleLine=true,
            leadingIcon={Box(Modifier.padding(start=16.dp,end=8.dp),contentAlignment=Alignment.Center){Icon(MiuixIcons.Search,null,Modifier.size(22.dp))}},
            trailingIcon={IconButton(onClick=submit,modifier=Modifier.padding(end=6.dp)){Icon(MiuixIcons.ChevronForward,stringResource(R.string.search))}},
            modifier=Modifier.fillMaxWidth().padding(horizontal=12.dp,vertical=8.dp),
            keyboardOptions=KeyboardOptions(imeAction=ImeAction.Search),keyboardActions=KeyboardActions(onSearch={submit()}))
    }
    if(showResults){
        if(m.searchResults.isEmpty()&&!m.busy)item("search-no-results"){EmptyMessage(stringResource(R.string.no_search_results),stringResource(R.string.try_search_again))}
        itemsIndexed(m.searchResults,key={i,s->"search:${s.optString("mid")}:$i"}){i,s->SongRow(s,playingSong(m,s)){m.play(m.searchResults,i)}}
    }else{
        if(m.searchHistory.isNotEmpty()){
            item("history-header"){Row(Modifier.fillMaxWidth(),verticalAlignment=Alignment.CenterVertically){Box(Modifier.weight(1f)){SmallTitle(stringResource(R.string.search_history))};TextButton(stringResource(R.string.clear_history),m::clearSearchHistory,Modifier.padding(end=12.dp))}}
            item("history-chips"){LazyRow(contentPadding=PaddingValues(horizontal=12.dp),horizontalArrangement=Arrangement.spacedBy(8.dp)){
                items(m.searchHistory,key={it}){query->DiscoveryChip(query){m.searchText=query;m.search()}}
            }}
        }
        val artists=daily.map{it.optString("artist")}.filter{it.isNotBlank()}.distinct().take(8)
        if(artists.isNotEmpty()){
            item("artists-header"){SmallTitle(stringResource(R.string.discover_artists))}
            item("artists-chips"){LazyRow(contentPadding=PaddingValues(horizontal=12.dp),horizontalArrangement=Arrangement.spacedBy(8.dp)){
                items(artists,key={it}){artist->DiscoveryChip(artist){m.searchText=artist;m.search()}}
            }}
        }
        item("search-recommend-header"){SectionHeader(stringResource(R.string.search_recommendations),m::refreshHome)}
        if(daily.isNotEmpty()){
            item("search-featured"){AlbumShelf(daily.take(6),false){_,i->m.play(daily,i)}}
            item("search-list-header"){MusicSectionTitle(stringResource(R.string.listen_now))}
            itemsIndexed(daily.drop(6).take(6),key={i,s->"search-discover:${s.optString("mid")}:$i"}){i,s->SongRow(s,playingSong(m,s)){m.play(daily,i+6)}}
        }else item("search-discovery-empty"){
            EmptyMessage(stringResource(R.string.search_empty),stringResource(if(m.loggedIn)R.string.recommendation_empty_summary else R.string.sync_qq_summary))
            TextButton(stringResource(if(m.loggedIn)R.string.refresh else R.string.login_qq),{if(m.loggedIn)m.refreshHome()else m.login()},Modifier.padding(horizontal=12.dp).fillMaxWidth())
        }
    }
}

@Composable private fun DiscoveryChip(label:String,onClick:()->Unit){
    Card(Modifier.widthIn(max=180.dp).heightIn(min=48.dp),onClick=onClick,pressFeedbackType=PressFeedbackType.Sink){
        Box(Modifier.padding(horizontal=16.dp).heightIn(min=48.dp),contentAlignment=Alignment.CenterStart){
            Text(label,style=MiuixTheme.textStyles.body1,maxLines=1,overflow=TextOverflow.Ellipsis)
        }
    }
}

@Composable internal fun DiscoveryLoadState(m:MusicState){
    Column(Modifier.fillMaxWidth().padding(20.dp),horizontalAlignment=Alignment.CenterHorizontally,verticalArrangement=Arrangement.spacedBy(8.dp)){
        Text(when{m.discoveryLoading->stringResource(R.string.discovery_loading);m.discoveryError.isNotBlank()->m.discoveryError;!m.loggedIn->stringResource(R.string.login_recommendations);!m.discoveryMore->stringResource(R.string.discovery_end);else->stringResource(R.string.discovery_scroll)},style=MiuixTheme.textStyles.footnote1,color=MiuixTheme.colorScheme.onSurfaceVariantSummary)
        if(m.discoveryError.isNotBlank())TextButton(stringResource(R.string.retry),m::loadMoreDiscovery)
    }
}
