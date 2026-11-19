package dev.qqlyric.app

import android.graphics.Canvas
import android.graphics.ColorFilter
import android.graphics.Paint
import android.graphics.Path
import android.graphics.PixelFormat
import android.graphics.drawable.Drawable

/** Resolution independent line icons, drawn on a 24-unit grid. */
class MusicIcon(private val kind: String, color: Int) : Drawable() {
    private val paint=Paint(Paint.ANTI_ALIAS_FLAG).apply {
        this.color=color;style=Paint.Style.STROKE;strokeWidth=1.8f
        strokeCap=Paint.Cap.ROUND;strokeJoin=Paint.Join.ROUND
    }
    override fun draw(canvas:Canvas) {
        canvas.save();canvas.translate(bounds.left.toFloat(),bounds.top.toFloat())
        canvas.scale(bounds.width()/24f,bounds.height()/24f)
        when(kind) {
            "down" -> {canvas.drawLine(5f,9f,12f,16f,paint);canvas.drawLine(12f,16f,19f,9f,paint)}
            "home" -> { val p=Path().apply {moveTo(3f,11f);lineTo(12f,3f);lineTo(21f,11f);moveTo(5f,10f);lineTo(5f,21f);lineTo(19f,21f);lineTo(19f,10f);moveTo(10f,21f);lineTo(10f,15f);lineTo(14f,15f);lineTo(14f,21f)};canvas.drawPath(p,paint) }
            "search" -> {canvas.drawCircle(10.5f,10.5f,6.5f,paint);canvas.drawLine(15.5f,15.5f,21f,21f,paint)}
            "mine" -> {canvas.drawCircle(12f,7f,4f,paint);canvas.drawArc(4f,13f,20f,28f,180f,180f,false,paint)}
            "play" -> {canvas.drawPath(Path().apply{moveTo(8f,4f);lineTo(20f,12f);lineTo(8f,20f);close()},paint)}
            "pause" -> {canvas.drawLine(8f,5f,8f,19f,paint);canvas.drawLine(16f,5f,16f,19f,paint)}
            "shuffle" -> {
                canvas.drawPath(Path().apply{moveTo(3f,5f);lineTo(7f,5f);cubicTo(11f,5f,13f,19f,17f,19f);lineTo(21f,19f);moveTo(3f,19f);lineTo(7f,19f);cubicTo(11f,19f,13f,5f,17f,5f);lineTo(21f,5f);moveTo(18f,2f);lineTo(21f,5f);lineTo(18f,8f);moveTo(18f,16f);lineTo(21f,19f);lineTo(18f,22f)},paint)
            }
            "repeat","repeat_one" -> {
                canvas.drawPath(Path().apply{moveTo(6f,6f);lineTo(20f,6f);lineTo(17f,3f);moveTo(20f,6f);lineTo(17f,9f);moveTo(20f,11f);lineTo(20f,17f);lineTo(4f,17f);lineTo(7f,20f);moveTo(4f,17f);lineTo(7f,14f);moveTo(4f,12f);lineTo(4f,6f)},paint)
                if(kind=="repeat_one")canvas.drawLine(12f,10f,12f,14f,paint)
            }
            "order" -> {canvas.drawLine(3f,5f,20f,5f,paint);canvas.drawLine(3f,11f,20f,11f,paint);canvas.drawPath(Path().apply{moveTo(3f,17f);lineTo(20f,17f);moveTo(17f,14f);lineTo(20f,17f);lineTo(17f,20f)},paint)}
            "next","previous" -> {
                if(kind=="previous"){canvas.translate(24f,0f);canvas.scale(-1f,1f)}
                canvas.drawPath(Path().apply{moveTo(5f,5f);lineTo(16f,12f);lineTo(5f,19f);close()},paint)
                canvas.drawLine(20f,5f,20f,19f,paint)
            }
            "heart" -> {canvas.drawPath(Path().apply{moveTo(12f,21f);cubicTo(9f,18f,2f,13f,2f,7f);cubicTo(2f,1f,10f,1f,12f,6f);cubicTo(14f,1f,22f,1f,22f,7f);cubicTo(22f,13f,15f,18f,12f,21f)},paint)}
            "queue" -> {canvas.drawLine(3f,5f,21f,5f,paint);canvas.drawLine(3f,11f,21f,11f,paint);canvas.drawLine(3f,17f,13f,17f,paint);canvas.drawPath(Path().apply{moveTo(17f,15f);lineTo(22f,18f);lineTo(17f,21f);close()},paint)}
            else -> {canvas.drawLine(10f,5f,10f,17f,paint);canvas.drawLine(10f,5f,20f,3f,paint);canvas.drawLine(20f,3f,20f,15f,paint);canvas.drawOval(4f,15f,10f,20f,paint);canvas.drawOval(14f,13f,20f,18f,paint)}
        }
        canvas.restore()
    }
    override fun setAlpha(alpha:Int) {paint.alpha=alpha}
    override fun setColorFilter(filter:ColorFilter?) {paint.colorFilter=filter}
    @Deprecated("Deprecated in Java") override fun getOpacity()=PixelFormat.TRANSLUCENT
}
