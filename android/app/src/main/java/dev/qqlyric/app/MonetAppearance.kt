package dev.qqlyric.app

import android.content.*
import android.os.Build
import androidx.compose.runtime.*
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.core.content.ContextCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import top.yukonga.miuix.kmp.theme.*

/** All pages receive the same wallpaper/cover/custom seed and full semantic color scheme. */
@Composable fun rememberMusicTheme(m:MusicState):ThemeController {
    val context=LocalContext.current
    val configuration=LocalConfiguration.current
    val lifecycle=LocalLifecycleOwner.current.lifecycle
    var wallpaperRevision by remember{mutableIntStateOf(0)}
    DisposableEffect(context,lifecycle){
        val observer=LifecycleEventObserver{_,event->if(event==Lifecycle.Event.ON_RESUME)wallpaperRevision++}
        val receiver=object:BroadcastReceiver(){override fun onReceive(c:Context?,i:Intent?){wallpaperRevision++}}
        lifecycle.addObserver(observer)
        ContextCompat.registerReceiver(context,receiver,IntentFilter(Intent.ACTION_WALLPAPER_CHANGED),ContextCompat.RECEIVER_NOT_EXPORTED)
        onDispose{lifecycle.removeObserver(observer);context.unregisterReceiver(receiver)}
    }
    val fallback=0xff3482ff.toInt()
    val wallpaper=remember(configuration,wallpaperRevision){if(Build.VERSION.SDK_INT>=31)context.getColor(android.R.color.system_accent1_500)else fallback}
    val coverSeed by produceState<Int?>(null,m.monetEnabled,m.monetSource,m.cover){
        value=null
        if(m.monetEnabled&&m.monetSource=="artwork"&&m.cover.isNotBlank())value=withContext(Dispatchers.IO){
            ArtworkCache.load(context,m.cover)?.let{bitmap->
                val small=android.graphics.Bitmap.createScaledBitmap(bitmap,48,48,true)
                val pixels=IntArray(small.width*small.height);small.getPixels(pixels,0,small.width,0,0,small.width,small.height)
                val seed=ArtworkSeed.extract(pixels,fallback);if(small!==bitmap)small.recycle();seed
            }
        }
    }
    val style=runCatching{ThemePaletteStyle.valueOf(m.monetStyle)}.getOrDefault(ThemePaletteStyle.TonalSpot)
    val spec=runCatching{ThemeColorSpec.valueOf(m.monetSpec)}.getOrDefault(ThemeColorSpec.Spec2021)
    val mode=if(m.monetEnabled)when(m.colorMode){"light"->ColorSchemeMode.MonetLight;"dark"->ColorSchemeMode.MonetDark;else->ColorSchemeMode.MonetSystem}
             else when(m.colorMode){"light"->ColorSchemeMode.Light;"dark"->ColorSchemeMode.Dark;else->ColorSchemeMode.System}
    val seed=when(m.monetSource){
        "artwork"->Color(coverSeed?:wallpaper)
        "custom"->Color(runCatching{android.graphics.Color.parseColor("#"+m.monetColor)}.getOrDefault(fallback))
        else->if(style==ThemePaletteStyle.TonalSpot&&spec==ThemeColorSpec.Spec2021&&Build.VERSION.SDK_INT>=31)null else Color(wallpaper)
    }
    return remember(mode,seed,style,spec,configuration,wallpaperRevision){ThemeController(colorSchemeMode=mode,keyColor=seed,paletteStyle=style,colorSpec=spec)}
}
