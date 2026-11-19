package dev.qqlyric.app

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.focusable
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.*
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.key.*
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.*
import androidx.compose.ui.unit.dp
import top.yukonga.miuix.kmp.theme.MiuixTheme

/** Media-specific thin track inside a separate 48dp gesture/accessibility target. */
@Composable fun ThinSeekBar(position:Int,duration:Int,onSeek:(Int)->Unit,modifier:Modifier=Modifier,framed:Boolean=false,expansion:Float=1f,interactive:Boolean=true,onScrubbing:(Boolean)->Unit={}){
    val colors=MiuixTheme.colorScheme
    val fraction=if(duration>0)(position.toFloat()/duration).coerceIn(0f,1f)else 0f
    val currentFraction by rememberUpdatedState(fraction)
    val currentDuration by rememberUpdatedState(duration)
    val currentSeek by rememberUpdatedState(onSeek)
    val currentInteractive by rememberUpdatedState(interactive)
    val currentScrubbing by rememberUpdatedState(onScrubbing)
    var preview by remember{mutableStateOf<Float?>(null)}
    var width by remember{mutableIntStateOf(1)}
    val label=stringResource(R.string.playback_progress)
    val commit={value:Float->if(currentDuration>0&&currentInteractive)currentSeek((value.coerceIn(0f,1f)*currentDuration).toInt())}
    Canvas(modifier.height(48.dp).onSizeChanged{width=it.width.coerceAtLeast(1)}
        .semantics{contentDescription=label;progressBarRangeInfo=ProgressBarRangeInfo(fraction,0f..1f);if(duration<=0||!interactive)disabled();setProgress{if(duration>0&&interactive){commit(it);true}else false}}
        .focusable(enabled=duration>0&&interactive)
        .onKeyEvent{event->if(event.type==KeyEventType.KeyDown&&(event.key==Key.DirectionLeft||event.key==Key.DirectionRight)&&duration>0&&interactive){currentSeek((position+if(event.key==Key.DirectionLeft)-5000 else 5000).coerceIn(0,duration));true}else false}
        .pointerInput(interactive){if(interactive)detectTapGestures{point->commit(point.x/width)}}
        .pointerInput(interactive){if(interactive)detectHorizontalDragGestures(
            onDragStart={if(currentDuration>0&&currentInteractive){preview=(it.x/width).coerceIn(0f,1f);currentScrubbing(true)}},
            onDragCancel={preview=null;currentScrubbing(false)},onDragEnd={preview?.let(commit);preview=null;currentScrubbing(false)},
            onHorizontalDrag={change,_->if(currentDuration>0&&currentInteractive){change.consume();preview=(change.position.x/width).coerceIn(0f,1f)}}
        )}){
        val p=expansion.coerceIn(0f,1f)
        val inset=if(framed)1.dp.toPx()else (1f+5f*p).dp.toPx()
        val left=inset;val right=(size.width-inset).coerceAtLeast(left)
        val y=size.height/2;val trackHeight=if(framed)8.dp.toPx()else (4f-p).dp.toPx()
        val radius=CornerRadius(trackHeight/2)
        drawRoundRect(colors.surfaceContainerHighest,Offset(left,y-trackHeight/2),Size(right-left,trackHeight),radius)
        val played=preview?:currentFraction
        if(played>0)drawRoundRect(colors.primary,Offset(left,y-trackHeight/2),Size((right-left)*played,trackHeight),radius)
        if(framed)drawRoundRect(colors.onSurfaceVariantSummary,Offset(left,y-trackHeight/2),Size(right-left,trackHeight),radius,style=Stroke(1.dp.toPx()))
        else {
            if(p<1f)drawRoundRect(colors.onSurfaceVariantSummary.copy(alpha=colors.onSurfaceVariantSummary.alpha*(1f-p)),Offset(left,y-trackHeight/2),Size(right-left,trackHeight),radius,style=Stroke(0.7.dp.toPx()))
            if(p>0f)drawCircle(colors.primary,(if(preview!=null)7.dp.toPx()else 5.dp.toPx())*p,Offset(left+(right-left)*played,y))
        }
    }
}


/** Always visible on the compact song bar, independent of its temporary expanded controls. */
@Composable fun CompactPlaybackProgress(position:Int,duration:Int,modifier:Modifier=Modifier){
    val c=MiuixTheme.colorScheme
    val fraction=if(duration>0)(position.toFloat()/duration).coerceIn(0f,1f)else 0f
    val label=stringResource(R.string.playback_progress)
    Canvas(modifier.height(4.dp).semantics{contentDescription=label;progressBarRangeInfo=ProgressBarRangeInfo(fraction,0f..1f)}){
        val corners=CornerRadius(size.height/2)
        drawRoundRect(c.surfaceContainerHighest,cornerRadius=corners)
        if(fraction>0f)drawRoundRect(c.primary,size=Size(size.width*fraction,size.height),cornerRadius=corners)
        drawRoundRect(c.onSurfaceVariantSummary,size=size,cornerRadius=corners,style=Stroke(0.7.dp.toPx()))
    }
}
