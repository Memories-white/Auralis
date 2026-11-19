package dev.qqlyric.app
import android.content.Context
import android.graphics.Canvas
import android.widget.Button
class PlayerAction(context:Context):Button(context){
    private var icon:MusicIcon?=null
    private var iconKey=""
    init{minimumWidth=0;minWidth=0;setPadding(0,0,0,0)}
    fun setIcon(kind:String,color:Int,label:String){
        contentDescription=label
        val key="$kind/$color"
        if(key!=iconKey){iconKey=key;icon=MusicIcon(kind,color);invalidate()}
    }
    override fun onDraw(canvas:Canvas){
        val size=(22*resources.displayMetrics.density).toInt().coerceAtMost(minOf(width,height))
        val x=(width-size)/2;val y=(height-size)/2
        icon?.setBounds(x,y,x+size,y+size);icon?.draw(canvas)
    }
}
