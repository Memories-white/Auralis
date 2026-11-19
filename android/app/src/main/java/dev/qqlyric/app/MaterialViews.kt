package dev.qqlyric.app
import android.app.Activity
import android.content.Context
import android.content.res.ColorStateList
import android.content.res.Configuration
import android.view.ContextThemeWrapper
import android.widget.Button
import com.google.android.material.button.MaterialButton
import com.google.android.material.card.MaterialCardView
import com.google.android.material.bottomnavigation.BottomNavigationView

object MaterialViews{
    fun context(activity:Activity,p:Palette):Context{
        val config=Configuration().apply{uiMode=if(p.dark)Configuration.UI_MODE_NIGHT_YES else Configuration.UI_MODE_NIGHT_NO}
        return ContextThemeWrapper(activity,R.style.Theme_Auralis_Material).apply{applyOverrideConfiguration(config)}
    }
    fun button(activity:Activity,p:Palette)=MaterialButton(context(activity,p))
    fun card(activity:Activity,p:Palette)=MaterialCardView(context(activity,p)).apply{
        setCardBackgroundColor(p.surface);cardElevation=0f;radius=UiTheme.dp(activity,p.radius).toFloat();strokeWidth=0
    }
    fun navigation(activity:Activity,p:Palette,onSelect:(String)->Unit)=BottomNavigationView(context(activity,p)).apply{
        setBackgroundColor(p.surface)
        itemIconTintList=ColorStateList(arrayOf(intArrayOf(android.R.attr.state_checked),intArrayOf()),intArrayOf(p.onContainer,p.muted))
        itemTextColor=ColorStateList(arrayOf(intArrayOf(android.R.attr.state_checked),intArrayOf()),intArrayOf(p.text,p.muted))
        itemActiveIndicatorColor=ColorStateList.valueOf(p.container)
        labelVisibilityMode=com.google.android.material.navigation.NavigationBarView.LABEL_VISIBILITY_LABELED
        itemTextAppearanceActive=R.style.TextAppearance_Auralis_Navigation
        itemTextAppearanceInactive=R.style.TextAppearance_Auralis_Navigation
        listOf("home" to "首页","search" to "搜索","mine" to "我的").forEachIndexed{index,pair->menu.add(0,index+1,index,pair.second).icon=MusicIcon(pair.first,p.text)}
        // WindowLayout already reserves system bars on the outer root. Material's
        // default inset listener would reserve the gesture bar a second time,
        // leaving too little height for the destination labels in the 80dp bar.
        androidx.core.view.ViewCompat.setOnApplyWindowInsetsListener(this){_,insets->insets}
        setPadding(0,0,0,0)
        setOnItemSelectedListener{item->onSelect(listOf("home","search","mine")[item.itemId-1]);true}
    }
}
