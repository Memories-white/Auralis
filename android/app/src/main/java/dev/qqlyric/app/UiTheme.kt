package dev.qqlyric.app

import android.app.Activity
import android.content.res.ColorStateList
import android.graphics.Color
import android.graphics.Typeface
import android.graphics.drawable.GradientDrawable
import android.graphics.drawable.RippleDrawable
import android.view.View
import android.widget.Button
import android.widget.TextView
import org.json.JSONObject

enum class UiStyle(val label:String){MIUIX("MIUIX"),MATERIAL3("Material 3"),APPLE("Apple 音乐")}
data class Palette(
    val background:Int,val surface:Int,val accent:Int,val onAccent:Int,val text:Int,val muted:Int,val radius:Float,val dark:Boolean,
    val raised:Int,val action:Int,val container:Int,val onContainer:Int,val outline:Int,val error:Int,val heroEnd:Int,
    val titleSize:Float,val sectionSize:Float,val bodySize:Float,val captionSize:Float,val buttonRadius:Float,val gutter:Float,val navHeight:Float,val motionMs:Long,val style:UiStyle
)

object UiTheme {
    private var tokens:JSONObject?=null
    fun palette(activity:Activity,style:UiStyle):Palette {
        val mode=activity.getSharedPreferences("library",Activity.MODE_PRIVATE).getString("color_mode","system")
        val dark=mode=="dark"||(mode=="system"&&(activity.resources.configuration.uiMode and android.content.res.Configuration.UI_MODE_NIGHT_MASK)==android.content.res.Configuration.UI_MODE_NIGHT_YES)
        val all=tokens ?: JSONObject(activity.resources.openRawResource(R.raw.design_tokens).bufferedReader().use{it.readText()}).also{tokens=it}
        val spec=all.getJSONObject(style.name);val colors=spec.getJSONObject(if(dark)"dark"else"light")
        fun c(key:String)=Color.parseColor(colors.getString(key))
        fun n(key:String)=spec.getDouble(key).toFloat()
        return Palette(c("background"),c("surface"),c("accent"),c("onAccent"),c("text"),c("muted"),n("radius"),dark,
            c("raised"),c("action"),c("container"),c("onContainer"),c("outline"),c("error"),c("heroEnd"),
            n("titleSize"),n("sectionSize"),n("bodySize"),n("captionSize"),n("buttonRadius"),n("gutter"),n("navHeight"),spec.getLong("motionMs"),style)
    }
    fun dp(activity:Activity,value:Float)=(value*activity.resources.displayMetrics.density+0.5f).toInt()
    fun shape(color:Int,radius:Float,activity:Activity)=GradientDrawable().apply{setColor(color);cornerRadius=dp(activity,radius).toFloat()}
    fun panel(view:View,activity:Activity,palette:Palette){view.background=shape(palette.surface,palette.radius,activity);view.elevation=0f}
    fun feedback(view:View,activity:Activity,palette:Palette,radius:Float=palette.radius,color:Int=palette.surface){
        val ripple=Color.argb(if(palette.dark)36 else 24,Color.red(palette.text),Color.green(palette.text),Color.blue(palette.text))
        view.background=RippleDrawable(ColorStateList.valueOf(ripple),shape(color,radius,activity),shape(Color.WHITE,radius,activity))
        view.isFocusable=true
        view.setOnFocusChangeListener{v,focused->v.foreground=if(focused)shape(Color.TRANSPARENT,radius,activity).apply{setStroke(dp(activity,2f),palette.accent)}else null}
    }
    fun button(button:Button,activity:Activity,palette:Palette,filled:Boolean=false){
        if(button is com.google.android.material.button.MaterialButton){
            button.backgroundTintList=ColorStateList.valueOf(if(filled)palette.action else palette.container)
            button.rippleColor=ColorStateList.valueOf(Color.argb(25,Color.red(palette.text),Color.green(palette.text),Color.blue(palette.text)))
        }else feedback(button,activity,palette,palette.buttonRadius,if(filled)palette.action else if(palette.style==UiStyle.APPLE)palette.container else palette.raised)
        button.setTextColor(ColorStateList(arrayOf(intArrayOf(-android.R.attr.state_enabled),intArrayOf()),intArrayOf(palette.muted,if(filled)palette.onAccent else if(palette.style==UiStyle.APPLE)palette.accent else palette.text)))
        button.isAllCaps=false;button.minimumWidth=0;button.minWidth=0;button.stateListAnimator=null;button.textSize=if(palette.style==UiStyle.MATERIAL3)14f else 16f
        button.minHeight=dp(activity,48f);button.setPadding(dp(activity,14f),0,dp(activity,14f),0)
        button.typeface=Typeface.create("sans-serif-medium",Typeface.NORMAL)
    }
    fun text(view:TextView,palette:Palette,muted:Boolean=false,bold:Boolean=false){
        view.setTextColor(if(muted)palette.muted else palette.text)
        view.typeface=Typeface.create(if(bold)"sans-serif-medium"else"sans-serif",if(bold&&palette.style==UiStyle.APPLE)Typeface.BOLD else Typeface.NORMAL)
        view.letterSpacing=0f;view.includeFontPadding=false
    }
    fun duration(palette:Palette):Long=if(android.animation.ValueAnimator.areAnimatorsEnabled())palette.motionMs else 0L
    fun lyricDuration(palette:Palette):Long=if(android.animation.ValueAnimator.areAnimatorsEnabled())400L else 0L
    fun lyricInterpolator()=android.view.animation.Interpolator{fraction->
        fun spring(t:Double)=1-Math.exp(-8.5*t)*(Math.cos(10*t)+0.85*Math.sin(10*t))
        (spring(fraction.toDouble())/spring(1.0)).toFloat()
    }
    fun interpolator(palette:Palette):android.view.animation.Interpolator=when(palette.style){
        UiStyle.MIUIX->android.view.animation.PathInterpolator(0.34f,1.15f,0.64f,1f)
        UiStyle.APPLE->android.view.animation.PathInterpolator(0.32f,0.72f,0f,1f)
        UiStyle.MATERIAL3->android.view.animation.PathInterpolator(0.2f,0f,0f,1f)
    }
}
