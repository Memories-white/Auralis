package dev.qqlyric.app

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.*
import androidx.compose.ui.unit.dp
import top.yukonga.miuix.kmp.basic.Text
import top.yukonga.miuix.kmp.overlay.OverlayBottomSheet
import top.yukonga.miuix.kmp.preference.RadioButtonPreference
import top.yukonga.miuix.kmp.theme.MiuixTheme

@Composable fun sleepTimerSummary(m:MusicState):String=when{
    m.sleepAfterTrack->stringResource(R.string.sleep_track_summary)
    m.sleepRemainingSeconds>0->stringResource(R.string.sleep_remaining,"%02d:%02d".format(m.sleepRemainingSeconds/60,m.sleepRemainingSeconds%60))
    m.sleepFinished->stringResource(R.string.sleep_finished)
    else->stringResource(R.string.sleep_off_summary)
}

@Composable fun TimerGlyph(label:String){
    val color=MiuixTheme.colorScheme.onSurface
    Canvas(Modifier.size(24.dp).semantics{contentDescription=label}){
        val center=Offset(size.width/2,size.height/2)
        drawCircle(color,radius=size.minDimension*0.38f,center=center,style=androidx.compose.ui.graphics.drawscope.Stroke(1.8.dp.toPx()))
        drawLine(color,center,Offset(center.x,center.y-size.height*0.22f),strokeWidth=1.8.dp.toPx(),cap=androidx.compose.ui.graphics.StrokeCap.Round)
        drawLine(color,center,Offset(center.x+size.width*0.18f,center.y+size.height*0.10f),strokeWidth=1.8.dp.toPx(),cap=androidx.compose.ui.graphics.StrokeCap.Round)
    }
}

@Composable fun SleepTimerSheet(m:MusicState,show:Boolean,onClose:()->Unit){
    OverlayBottomSheet(show=show,title=stringResource(R.string.sleep_timer),onDismissRequest=onClose){
        Column(Modifier.fillMaxWidth().heightIn(max=440.dp).verticalScroll(rememberScrollState())){SleepTimerChoices(m)}
    }
}

@Composable fun SleepTimerChoices(m:MusicState){
    val c=MiuixTheme.colorScheme
    Text(sleepTimerSummary(m),Modifier.padding(horizontal=20.dp,vertical=12.dp),style=MiuixTheme.textStyles.title3,color=c.primary)
    if(m.mid.isBlank())Text(stringResource(R.string.sleep_requires_song),Modifier.padding(horizontal=20.dp,vertical=8.dp),color=c.onSurfaceVariantSummary)
    RadioButtonPreference(title=stringResource(R.string.sleep_off),selected=m.sleepRemainingSeconds==0&&!m.sleepAfterTrack,enabled=m.mid.isNotBlank(),onClick={m.setSleepTimer(0)})
    listOf(15,30,60,90).forEach{minutes->RadioButtonPreference(title=stringResource(R.string.sleep_minutes,minutes),selected=m.sleepMinutes==minutes&&m.sleepRemainingSeconds>0,enabled=m.mid.isNotBlank(),onClick={m.setSleepTimer(minutes)})}
    RadioButtonPreference(title=stringResource(R.string.sleep_after_track),summary=stringResource(R.string.sleep_track_detail),selected=m.sleepAfterTrack,enabled=m.mid.isNotBlank(),onClick={m.setSleepTimer(-1)})
    Text(stringResource(R.string.sleep_lifecycle),Modifier.padding(20.dp),style=MiuixTheme.textStyles.footnote1,color=c.onSurfaceVariantSummary)
}
