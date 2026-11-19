package dev.qqlyric.app

import android.animation.ValueAnimator
import android.content.Context
import android.view.View
import android.view.animation.Interpolator
import android.widget.FrameLayout

/** Both pages stay measured, so a swipe does not replace layout mid-animation. */
class PlayerPager(context:Context):FrameLayout(context){
    var displayedChild=0
        private set
    private var motion:ValueAnimator?=null
    private var generation=0
    override fun onViewAdded(child:View){
        super.onViewAdded(child)
        child.visibility=if(indexOfChild(child)==displayedChild)View.VISIBLE else View.INVISIBLE
        child.layoutParams=LayoutParams(LayoutParams.MATCH_PARENT,LayoutParams.MATCH_PARENT)
    }
    fun showPage(index:Int,duration:Long,curve:Interpolator){
        if(index==displayedChild||index !in 0 until childCount)return
        val token=++generation;motion?.cancel()
        val old=getChildAt(displayedChild);val next=getChildAt(index)
        val direction=if(index>displayedChild)1 else -1
        displayedChild=index
        old.visibility=View.VISIBLE;next.visibility=View.VISIBLE
        fun settle(){for(i in 0 until childCount)getChildAt(i).apply{translationX=0f;alpha=1f;visibility=if(i==displayedChild)View.VISIBLE else View.INVISIBLE}}
        if(duration==0L||width==0){settle();return}
        motion=ValueAnimator.ofFloat(0f,1f).apply{
            this.duration=duration;interpolator=curve
            addUpdateListener{
                val t=it.animatedValue as Float
                old.translationX=-direction*width*t;next.translationX=direction*width*(1-t)
                old.alpha=1-t*0.2f;next.alpha=0.8f+t*0.2f
            }
            addListener(object:android.animation.AnimatorListenerAdapter(){override fun onAnimationEnd(animation:android.animation.Animator){if(token==generation)settle()}})
            start()
        }
    }
    override fun onDetachedFromWindow(){generation++;motion?.cancel();super.onDetachedFromWindow()}
}
