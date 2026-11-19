package dev.qqlyric.app

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.*
import androidx.compose.foundation.*
import androidx.compose.foundation.interaction.collectIsDraggedAsState
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.*
import androidx.compose.runtime.*
import androidx.compose.runtime.snapshots.Snapshot
import androidx.compose.ui.Modifier
import androidx.compose.ui.MotionDurationScale
import androidx.compose.ui.draw.*
import androidx.compose.ui.geometry.*
import androidx.compose.ui.graphics.*
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.first
import top.yukonga.miuix.kmp.basic.Text
import top.yukonga.miuix.kmp.theme.MiuixTheme
import kotlin.math.abs

private data class LyricFlight(val revision:Int=0,val distance:Float=0f,val animated:Boolean=false)
/** Last presented transform, sampled only in the draw phase, for interruptible FLIP continuity. */
private class PresentedRow { var translation=0f;var velocity=0f }
private class RowFlight(val offset:Animatable<Float,AnimationVector1D>,val velocity:Float)

@Composable fun SynchronizedLyrics(m:MusicState,visible:Boolean){
    val lines=remember(m.lyrics){Lrc.parse(m.lyrics)}
    val translations=remember(m.translation){Lrc.parse(m.translation)}
    val active=Lrc.active(lines,m.position)
    val list=rememberLazyListState()
    val dragging by list.interactionSource.collectIsDraggedAsState()
    var browsing by remember{mutableStateOf(false)}
    var flight by remember(m.mid){mutableStateOf(LyricFlight())}
    val c=MiuixTheme.colorScheme
    LaunchedEffect(dragging){if(dragging)browsing=true else if(browsing){delay(1800);browsing=false}}
    LaunchedEffect(active,browsing,lines,visible){
        if(browsing||!visible){flight=LyricFlight(flight.revision+1);return@LaunchedEffect}
        if(active<0)return@LaunchedEffect
        snapshotFlow{list.layoutInfo.viewportSize.height}.first{it>0}
        val item=list.layoutInfo.visibleItemsInfo.firstOrNull{it.index==active}
        val distance=item?.offset?.toFloat()?:0f
        val animate=(coroutineContext[MotionDurationScale]?.scaleFactor?:1f)>0f&&item!=null&&abs(distance)>1f&&abs(distance)<list.layoutInfo.viewportSize.height
        // Move the lazy layout atomically, then spring each visible row from its old visual
        // position. Upcoming rows start later: the viewport no longer moves as one rigid block.
        Snapshot.withMutableSnapshot{
            flight=LyricFlight(flight.revision+1,distance,animate)
            list.requestScrollToItem(active,0)
        }
    }
    BoxWithConstraints(Modifier.fillMaxSize()){
        if(lines.isEmpty())Column(Modifier.fillMaxWidth().padding(28.dp,40.dp),verticalArrangement=Arrangement.spacedBy(8.dp)){
            Text(stringResource(R.string.no_lyrics),style=MiuixTheme.textStyles.title3)
            Text(stringResource(R.string.no_lyrics_summary),color=c.onSurfaceVariantSummary)
        }else LazyColumn(state=list,modifier=Modifier.fillMaxSize().drawWithContent{
            drawContext.canvas.saveLayer(Rect(Offset.Zero,size),Paint())
            drawContent()
            drawRect(Brush.verticalGradient(0f to Color.Transparent,0.10f to Color.Black,0.87f to Color.Black,1f to Color.Transparent),blendMode=BlendMode.DstIn)
            drawContext.canvas.restore()
        },contentPadding=PaddingValues(start=24.dp,end=48.dp,top=maxHeight*0.38f,bottom=maxHeight*0.62f)){
            itemsIndexed(lines,key={index,line->"${line.timeMs}:$index"}){index,line->
                val selected=index==active
                val distance=abs(index-active).coerceAtMost(5)
                val presented=remember(m.mid,index){PresentedRow()}
                val row=remember(flight.revision){RowFlight(Animatable(if(flight.animated)presented.translation+flight.distance else 0f),presented.velocity)}
                LaunchedEffect(row,visible,browsing){
                    if(!flight.animated||!visible||browsing){row.offset.snapTo(0f);return@LaunchedEffect}
                    val ahead=(index-active).coerceIn(0,5)
                    val speed=coroutineContext[MotionDurationScale]?.scaleFactor?:1f
                    if(speed<=0f){row.offset.snapTo(0f);return@LaunchedEffect}
                    delay((ahead*38L*speed).toLong())
                    row.offset.animateTo(0f,spring(dampingRatio=if(index<active)0.88f else 0.74f,stiffness=if(index<active)150f else 120f-ahead*6f),initialVelocity=row.velocity)
                }
                // Focus changes continuously: an incoming line clears as it settles, while
                // the outgoing line gently diffuses rather than abruptly becoming blurred.
                val targetBlur=if(selected||browsing||active<0)0f else when(distance){1->if(index<active)0.85f else 0.55f;2->1.35f;3->2.2f;else->3.2f}
                val blurRadius by animateFloatAsState(targetBlur,tween(if(selected)320 else 480,easing=CubicBezierEasing(0.2f,0f,0f,1f)),label="lyric focal blur")
                val depth by animateFloatAsState(if(browsing||active<0)0f else distance.toFloat(),tween(420),label="lyric depth")
                val emphasis=remember(m.mid,index){Animatable(0.96f)}
                LaunchedEffect(selected,browsing,visible){
                    val speed=coroutineContext[MotionDurationScale]?.scaleFactor?:1f
                    if(!visible||speed<=0f){emphasis.snapTo(if(selected)1.075f else 0.96f);return@LaunchedEffect}
                    if(selected&&!browsing){
                        delay((45*speed).toLong())
                        emphasis.animateTo(1.105f,spring(dampingRatio=0.70f,stiffness=120f))
                        emphasis.animateTo(1.075f,spring(dampingRatio=0.9f,stiffness=110f))
                    }else emphasis.animateTo(0.96f,spring(dampingRatio=0.9f,stiffness=135f))
                }
                val ink by animateColorAsState(if(selected||browsing)c.onSurface else c.onSurfaceVariantSummary,tween(240),label="lyric ink")
                val trans=translations.firstOrNull{abs(it.timeMs-line.timeMs)<=500&&it.text.trim()!="//"}
                Column(Modifier.fillMaxWidth().padding(horizontal=8.dp).graphicsLayer{
                    translationY=row.offset.value
                    presented.translation=translationY;presented.velocity=row.offset.velocity
                    scaleX=emphasis.value;scaleY=emphasis.value;transformOrigin=TransformOrigin(0f,0.5f)
                    alpha=if(selected||browsing)1f else 1-depth.coerceIn(0f,5f)*0.065f
                }.blur(blurRadius.coerceAtLeast(0f).dp,edgeTreatment=BlurredEdgeTreatment.Unbounded)
                    .clickable{browsing=false;m.command("SEEK","position",line.timeMs)}.padding(horizontal=4.dp,vertical=20.dp),verticalArrangement=Arrangement.spacedBy(8.dp)){
                    Text(line.text,style=MiuixTheme.textStyles.title2.copy(fontWeight=FontWeight.SemiBold),color=ink)
                    if(trans!=null)Text(trans.text,style=MiuixTheme.textStyles.body2,color=ink)
                }
            }
        }
    }
}
