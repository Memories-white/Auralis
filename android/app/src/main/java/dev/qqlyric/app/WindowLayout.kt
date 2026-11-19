package dev.qqlyric.app

import android.app.Activity
import android.view.View
import android.view.WindowManager

object WindowLayout {
    fun apply(activity:Activity, root:View) {
        root.isFocusableInTouchMode=true
        root.requestFocus()
        activity.window.setSoftInputMode(WindowManager.LayoutParams.SOFT_INPUT_STATE_ALWAYS_HIDDEN or WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE)
        val left=root.paddingLeft;val top=root.paddingTop;val right=root.paddingRight;val bottom=root.paddingBottom
        root.setOnApplyWindowInsetsListener { view,insets ->
            if(android.os.Build.VERSION.SDK_INT>=30){
                val bars=insets.getInsets(android.view.WindowInsets.Type.systemBars() or android.view.WindowInsets.Type.displayCutout())
                view.setPadding(left+bars.left,top+bars.top,right+bars.right,bottom+bars.bottom)
            }else{view.setPadding(left+insets.systemWindowInsetLeft,top+insets.systemWindowInsetTop,right+insets.systemWindowInsetRight,bottom+insets.systemWindowInsetBottom)}
            insets
        }
        root.requestApplyInsets()
    }
}
