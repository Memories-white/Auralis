package dev.qqlyric.app

import android.animation.ValueAnimator
import android.app.Activity
import android.graphics.RenderEffect
import android.graphics.Shader
import android.os.Build
import android.widget.TextView
import androidx.core.graphics.ColorUtils

/** Each line owns its focus transition; the active line never receives a blur. */
class LyricLineView(private val host:Activity,private val palette:Palette):TextView(host){
    private var transition:ValueAnimator?=null
    private var focus=1f
    private var target=-1f
    private var effectStep=-1
    private val reader=host.getSystemService(android.content.Context.ACCESSIBILITY_SERVICE) as android.view.accessibility.AccessibilityManager

    fun focusDistance(distance:Int,browsing:Boolean,animated:Boolean){
        val next=if(browsing||reader.isTouchExplorationEnabled)0f else distance.coerceIn(0,4)/4f
        isSelected=distance==0
        if(isSelected && Build.VERSION.SDK_INT>=31){setRenderEffect(null);effectStep=0}
        if(next==target)return
        target=next;transition?.cancel()
        val duration=if(animated)UiTheme.lyricDuration(palette) else 0L
        if(duration==0L){focus=next;paintFocus();return}
        transition=ValueAnimator.ofFloat(focus,next).apply{
            this.duration=duration
            interpolator=UiTheme.lyricInterpolator()
            addUpdateListener{focus=it.animatedValue as Float;paintFocus()}
            start()
        }
    }
    private fun paintFocus(){
        val strength=focus.coerceIn(0f,1f)
        setTextColor(ColorUtils.blendARGB(palette.text,palette.muted,strength))
        alpha=1f-strength*0.38f
        // Fixed layout metrics prevent a font-size reflow from moving the target
        // mid-transition. Visually the singing line grows 12.5% over its neighbors.
        scaleX=1.08f-(focus.coerceAtLeast(-0.06f)/0.25f).coerceAtMost(1f)*0.12f
        scaleY=scaleX;pivotX=paddingLeft.toFloat()
        if(Build.VERSION.SDK_INT>=31){
            val step=if(isSelected)0 else (strength*UiTheme.dp(host,3.2f)*4).toInt()
            if(step!=effectStep){
                effectStep=step
                setRenderEffect(if(step==0)null else RenderEffect.createBlurEffect(step/4f,step/4f,Shader.TileMode.CLAMP))
            }
        }
    }
    override fun onDetachedFromWindow(){transition?.cancel();super.onDetachedFromWindow()}
}
