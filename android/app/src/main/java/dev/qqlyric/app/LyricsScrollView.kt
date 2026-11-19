package dev.qqlyric.app

import android.content.Context
import android.graphics.*
import android.widget.ScrollView

/** Transparent edge fade, confined to lyrics; never masks metadata or controls. */
class LyricsScrollView(context:Context):ScrollView(context){
    private val mask=Paint(Paint.ANTI_ALIAS_FLAG).apply{xfermode=PorterDuffXfermode(PorterDuff.Mode.DST_IN)}
    init{isVerticalScrollBarEnabled=false;overScrollMode=OVER_SCROLL_NEVER;clipToPadding=false}
    override fun onSizeChanged(w:Int,h:Int,oldw:Int,oldh:Int){
        super.onSizeChanged(w,h,oldw,oldh)
        mask.shader=LinearGradient(0f,0f,0f,h.toFloat().coerceAtLeast(1f),
            intArrayOf(Color.TRANSPARENT,Color.BLACK,Color.BLACK,Color.TRANSPARENT),floatArrayOf(0f,0.10f,0.86f,1f),Shader.TileMode.CLAMP)
    }
    override fun dispatchDraw(canvas:Canvas){
        val checkpoint=canvas.saveLayer(0f,scrollY.toFloat(),width.toFloat(),(scrollY+height).toFloat(),null)
        super.dispatchDraw(canvas)
        canvas.save();canvas.translate(0f,scrollY.toFloat());canvas.drawRect(0f,0f,width.toFloat(),height.toFloat(),mask);canvas.restore()
        canvas.restoreToCount(checkpoint)
    }
}
