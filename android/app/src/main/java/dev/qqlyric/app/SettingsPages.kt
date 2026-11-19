package dev.qqlyric.app

import android.os.Build
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.*
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import top.yukonga.miuix.kmp.basic.*
import top.yukonga.miuix.kmp.preference.*
import top.yukonga.miuix.kmp.theme.*

val SETTINGS_PAGES=setOf("settings","appearance","monet","monet-source","monet-style","monet-spec","quality","storage","about")
fun settingsTitle(page:String)=when(page){
 "settings"->R.string.settings;"appearance"->R.string.appearance;"monet"->R.string.monet_title
 "monet-source"->R.string.color_source;"monet-style"->R.string.palette_style;"monet-spec"->R.string.color_spec
 "quality"->R.string.audio_quality;"storage"->R.string.storage;else->R.string.about
}
fun appearanceLabel(mode:String)=when(mode){"light"->R.string.light_mode;"dark"->R.string.dark_mode;else->R.string.system_mode}
fun sourceLabel(source:String)=when(source){"artwork"->R.string.source_artwork;"custom"->R.string.source_custom;else->R.string.source_wallpaper}
fun playbackLabel(mode:String)=when(mode){"LOOP"->R.string.mode_loop;"SINGLE"->R.string.mode_single;"SHUFFLE"->R.string.mode_shuffle;else->R.string.mode_order}
fun paletteLabel(style:String)=when(style){
 "Neutral"->R.string.palette_neutral;"Vibrant"->R.string.palette_vibrant;"Expressive"->R.string.palette_expressive
 "Rainbow"->R.string.palette_rainbow;"FruitSalad"->R.string.palette_fruit;"Monochrome"->R.string.palette_mono
 "Fidelity"->R.string.palette_fidelity;"Content"->R.string.palette_content;"Cmf"->R.string.palette_cmf;else->R.string.palette_tonal
}
private val group=Modifier.padding(horizontal=12.dp).padding(bottom=12.dp).fillMaxWidth()

fun LazyListScope.settingsContent(m:MusicState,logout:()->Unit){
 when(m.page){
 "settings"->{
  item("appearance-title"){SmallTitle(stringResource(R.string.appearance_group))}
  item("appearance-group"){Card(group){
   ArrowPreference(title=stringResource(R.string.appearance),summary=stringResource(appearanceLabel(m.colorMode)),onClick={m.navigate("appearance")})
   SwitchPreference(checked=m.monetEnabled,onCheckedChange=m::setMonet,title=stringResource(R.string.monet_title),summary=stringResource(R.string.monet_summary))
   ArrowPreference(title=stringResource(R.string.monet_options),summary=stringResource(sourceLabel(m.monetSource))+" · "+stringResource(paletteLabel(m.monetStyle)),enabled=m.monetEnabled,onClick={m.navigate("monet")})
  }}
  item("player-title"){SmallTitle(stringResource(R.string.playback_group))}
  item("player-group"){Card(group){ArrowPreference(title=stringResource(R.string.audio_quality),summary=stringResource(qualityLabel(m.audioQuality)),onClick={m.navigate("quality")})}}
  item("app-title"){SmallTitle(stringResource(R.string.application_group))}
  item("app-group"){Card(group){
   ArrowPreference(title=stringResource(R.string.image_cache),summary=stringResource(R.string.cache_short),onClick={m.navigate("storage")})
   ArrowPreference(title=stringResource(R.string.about),summary="Auralis · "+BuildConfig.VERSION_NAME,onClick={m.navigate("about")})
  }}
  if(m.loggedIn)item("account-group"){SmallTitle(stringResource(R.string.account_group));Card(group){ArrowPreference(title=stringResource(R.string.logout),summary=stringResource(R.string.logout_summary),onClick=logout)}}
 }
 "quality"->{item{SmallTitle(stringResource(R.string.audio_quality));Card(group){QualityChoices(m)}}}
 "appearance"->{item{SmallTitle(stringResource(R.string.appearance_group));Card(group){
  listOf("system","light","dark").forEach{mode->RadioButtonPreference(title=stringResource(appearanceLabel(mode)),selected=m.colorMode==mode,onClick={m.setAppearance(mode)})}
 }}}
 "monet"->{
  item{SmallTitle(stringResource(R.string.monet_title));Card(group){
   SwitchPreference(checked=m.monetEnabled,onCheckedChange=m::setMonet,title=stringResource(R.string.monet_title),summary=stringResource(R.string.monet_summary))
   ArrowPreference(title=stringResource(R.string.color_source),summary=stringResource(sourceLabel(m.monetSource)),enabled=m.monetEnabled,onClick={m.navigate("monet-source")})
   ArrowPreference(title=stringResource(R.string.palette_style),summary=stringResource(paletteLabel(m.monetStyle)),enabled=m.monetEnabled,onClick={m.navigate("monet-style")})
   ArrowPreference(title=stringResource(R.string.color_spec),summary=if(m.monetSpec=="Spec2025")"2025"else"2021",enabled=m.monetEnabled,onClick={m.navigate("monet-spec")})
  }}
  item{SmallTitle(stringResource(R.string.color_preview));Card(group){
   val c=MiuixTheme.colorScheme
   Column(Modifier.padding(20.dp),verticalArrangement=Arrangement.spacedBy(16.dp)){
    Text(stringResource(R.string.preview_title),style=MiuixTheme.textStyles.title3)
    Text(stringResource(R.string.preview_summary),color=c.onSurfaceVariantSummary,style=MiuixTheme.textStyles.footnote1)
    Row(Modifier.fillMaxWidth(),horizontalArrangement=Arrangement.spacedBy(12.dp)){
     listOf(c.primary,c.secondary,c.tertiaryContainer,c.surfaceContainerHighest).forEach{Box(Modifier.weight(1f).height(48.dp).background(it,CircleShape))}
    }
   }
  }}
  if(m.monetSource=="custom")item{CustomColor(m)}
  if(m.monetSource=="artwork"&&m.cover.isBlank())item{SettingNote(stringResource(R.string.artwork_seed_fallback))}
  if(Build.VERSION.SDK_INT<31&&m.monetSource=="wallpaper")item{SettingNote(stringResource(R.string.wallpaper_fallback))}
 }
 "monet-source"->{item{SmallTitle(stringResource(R.string.color_source));Card(group){
  listOf("wallpaper","artwork","custom").forEach{source->RadioButtonPreference(title=stringResource(sourceLabel(source)),summary=stringResource(when(source){"wallpaper"->R.string.wallpaper_summary;"artwork"->R.string.artwork_summary;else->R.string.custom_summary}),selected=m.monetSource==source,enabled=m.monetEnabled,onClick={m.setMonetOption("source",source)})}
 }};if(m.monetSource=="custom")item{CustomColor(m)}}
 "monet-style"->{
  item{SmallTitle(stringResource(R.string.palette_style))}
  item{Card(group){ThemePaletteStyle.entries.forEach{style->RadioButtonPreference(title=stringResource(paletteLabel(style.name)),selected=m.monetStyle==style.name,enabled=m.monetEnabled,onClick={m.setMonetOption("style",style.name)})}}}
 }
 "monet-spec"->{item{SmallTitle(stringResource(R.string.color_spec));Card(group){
  ThemeColorSpec.entries.forEach{spec->RadioButtonPreference(title=if(spec==ThemeColorSpec.Spec2025)"2025"else"2021",summary=stringResource(if(spec==ThemeColorSpec.Spec2025)R.string.spec_2025_summary else R.string.spec_2021_summary),selected=m.monetSpec==spec.name,enabled=m.monetEnabled,onClick={m.setMonetOption("spec",spec.name)})}
 }}}
 "storage"->{item{
  val context=LocalContext.current
  val bytes by produceState(0L){value=withContext(Dispatchers.IO){java.io.File(context.cacheDir,"artwork-v1").listFiles()?.sumOf{it.length()}?:0L}}
  SmallTitle(stringResource(R.string.image_cache));Card(group){Column(Modifier.padding(20.dp),verticalArrangement=Arrangement.spacedBy(12.dp)){
   Text(stringResource(R.string.cache_used,bytes/1048576f),style=MiuixTheme.textStyles.title3)
   Text(stringResource(R.string.image_cache_summary),color=MiuixTheme.colorScheme.onSurfaceVariantSummary)
  }}
 }}
 "about"->{item{SmallTitle(stringResource(R.string.about));Card(group){Column(Modifier.padding(20.dp),verticalArrangement=Arrangement.spacedBy(12.dp)){
  Text("Auralis",style=MiuixTheme.textStyles.title2);Text(BuildConfig.VERSION_NAME);Text(stringResource(R.string.about_summary),color=MiuixTheme.colorScheme.onSurfaceVariantSummary)
 }}}}
 }
}
@Composable private fun SettingNote(text:String){Text(text,Modifier.padding(horizontal=28.dp,vertical=12.dp),style=MiuixTheme.textStyles.footnote1,color=MiuixTheme.colorScheme.onSurfaceVariantSummary)}
@Composable private fun CustomColor(m:MusicState){
 val c=MiuixTheme.colorScheme
 var draft by remember(m.monetColor){mutableStateOf(m.monetColor)}
 val valid=draft.removePrefix("#").matches(Regex("[a-fA-F0-9]{6}"))
 SmallTitle(stringResource(R.string.source_custom))
 Card(group){Column(Modifier.padding(16.dp),verticalArrangement=Arrangement.spacedBy(16.dp)){
  TextField(value=draft,onValueChange={value->draft=value.take(7);val clean=draft.removePrefix("#");if(clean.matches(Regex("[a-fA-F0-9]{6}")))m.setMonetOption("color",clean.uppercase())},label=stringResource(R.string.color_hex),singleLine=true,enabled=m.monetEnabled,modifier=Modifier.fillMaxWidth())
  if(!valid)Text(stringResource(R.string.color_hex_error),style=MiuixTheme.textStyles.footnote1,color=c.error)
  val presets=listOf("3482FF","7165C8","A34C68","B46A33","50836A","586C83","9C4263","77804D")
  presets.chunked(4).forEach{colors->Row(Modifier.fillMaxWidth(),horizontalArrangement=Arrangement.SpaceBetween){colors.forEach{hex->
   val color=Color(android.graphics.Color.parseColor("#$hex"));val selected=m.monetColor.equals(hex,true)
   IconButton(onClick={m.setMonetOption("color",hex)},enabled=m.monetEnabled,backgroundColor=color,minWidth=48.dp,minHeight=48.dp,cornerRadius=24.dp,modifier=Modifier.semantics{contentDescription="#$hex";this.selected=selected}.then(if(selected)Modifier.border(2.dp,c.onSurface,CircleShape)else Modifier)){
    if(selected)Text("✓",color=if(color.luminance()>0.45f)Color.Black else Color.White)
   }
  }}}
 }}
}
