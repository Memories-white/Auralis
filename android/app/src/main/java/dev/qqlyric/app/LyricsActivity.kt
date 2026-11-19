package dev.qqlyric.app

import android.app.Activity
import android.content.Intent
import android.graphics.Color
import android.graphics.Typeface
import android.graphics.drawable.GradientDrawable
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.view.Gravity
import android.widget.Button
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import android.widget.ViewFlipper
import android.widget.ImageView
import android.widget.SeekBar
import android.view.MotionEvent
import java.util.concurrent.Executors

class LyricsActivity : androidx.appcompat.app.AppCompatActivity() {
    private val ui = Handler(Looper.getMainLooper())
    private lateinit var style: UiStyle
    private lateinit var palette: Palette
    private lateinit var scroll: ScrollView
    private lateinit var list: LinearLayout
    private lateinit var heading: TextView
    private lateinit var toggle: Button
    private lateinit var pages:PlayerPager
    private lateinit var artwork:ImageView
    private lateinit var thumbnail:ImageView
    private lateinit var progress:SeekBar
    private lateinit var time:TextView
    private var downX=0f
    private var downY=0f
    private var coverUrl=""
    private val worker=Executors.newSingleThreadExecutor()
    private var lines = emptyList<LyricLine>()
    private var shownLyric = "\u0000"
    private var active = -2
    private var running = false
    private var scrollMotion:android.animation.ValueAnimator?=null
    private var browsingUntil=0L
    private var browsing=false
    private var lyricTouchStarted=false
    private val follow=LyricFollowState()
    private val tick = object : Runnable {
        override fun run() { if (running) { refresh(); ui.postDelayed(this, if(PlayerService.isPlaying)32 else 120) } }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        window.requestFeature(android.view.Window.FEATURE_ACTIVITY_TRANSITIONS)
        super.onCreate(savedInstanceState)
        style = runCatching { UiStyle.valueOf(getSharedPreferences("library", MODE_PRIVATE).getString("ui_style", "MIUIX") ?: "MIUIX") }.getOrDefault(UiStyle.MIUIX)
        palette = UiTheme.palette(this, style)
        val root = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(UiTheme.dp(this@LyricsActivity, 20f), UiTheme.dp(this@LyricsActivity, 16f), UiTheme.dp(this@LyricsActivity, 20f), UiTheme.dp(this@LyricsActivity, 12f))
            background = GradientDrawable(GradientDrawable.Orientation.TL_BR,intArrayOf(palette.background,palette.surface,palette.container))
        }
        setContentView(root)
        findViewById<android.view.View>(android.R.id.content).transitionName="player_surface"
        setEnterSharedElementCallback(com.google.android.material.transition.platform.MaterialContainerTransformSharedElementCallback())
        fun containerTransition(entering:Boolean)=com.google.android.material.transition.platform.MaterialContainerTransform().apply{
            addTarget(android.R.id.content)
            duration=if(UiTheme.duration(palette)==0L)0 else if(entering)460L else 360L
            interpolator=android.view.animation.PathInterpolator(0.2f,0f,0f,1f)
            setAllContainerColors(palette.background)
            scrimColor=Color.TRANSPARENT
            fadeMode=com.google.android.material.transition.platform.MaterialContainerTransform.FADE_MODE_THROUGH
            startShapeAppearanceModel=com.google.android.material.shape.ShapeAppearanceModel.builder().setAllCornerSizes(if(entering)UiTheme.dp(this@LyricsActivity,24f).toFloat()else 0f).build()
            endShapeAppearanceModel=com.google.android.material.shape.ShapeAppearanceModel.builder().setAllCornerSizes(if(entering)0f else UiTheme.dp(this@LyricsActivity,24f).toFloat()).build()
        }
        window.sharedElementEnterTransition=containerTransition(true)
        window.sharedElementReturnTransition=containerTransition(false)
        window.enterTransition=null;window.returnTransition=null
        window.statusBarColor=palette.background;window.navigationBarColor=palette.background
        window.decorView.systemUiVisibility=if(palette.dark)0 else android.view.View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR or android.view.View.SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR
        WindowLayout.apply(this,root)
        val top = LinearLayout(this).apply { gravity = Gravity.CENTER_VERTICAL }
        val back = PlayerAction(this).apply {setIcon("down",palette.text,"收起播放器");setOnClickListener { finishAfterTransition() };UiTheme.feedback(this,this@LyricsActivity,palette,24f,android.graphics.Color.TRANSPARENT)}
        top.addView(back, LinearLayout.LayoutParams(UiTheme.dp(this, 56f), UiTheme.dp(this, 48f)))
        val caption = TextView(this).apply { text = "正在播放"; textSize = 12f; gravity = Gravity.CENTER;letterSpacing=0.06f }
        UiTheme.text(caption, palette, muted = true)
        top.addView(caption, LinearLayout.LayoutParams(0, -2, 1f))
        root.addView(top)
        heading = TextView(this).apply { textSize = 20f;maxLines=2;ellipsize=android.text.TextUtils.TruncateAt.END;setPadding(0,UiTheme.dp(this@LyricsActivity,16f),0,UiTheme.dp(this@LyricsActivity,16f)) }
        UiTheme.text(heading, palette, bold = true)
        val metadata=LinearLayout(this).apply{gravity=Gravity.CENTER_VERTICAL}
        thumbnail=ImageView(this).apply{scaleType=ImageView.ScaleType.CENTER_CROP;background=UiTheme.shape(palette.surface,10f,this@LyricsActivity);clipToOutline=true;visibility=android.view.View.GONE}
        metadata.addView(thumbnail,LinearLayout.LayoutParams(UiTheme.dp(this,56f),UiTheme.dp(this,56f)).apply{marginEnd=UiTheme.dp(this@LyricsActivity,12f)})
        metadata.addView(heading,LinearLayout.LayoutParams(0,-2,1f));root.addView(metadata,LinearLayout.LayoutParams(-1,UiTheme.dp(this,88f)))
        scroll = LyricsScrollView(this).apply { isFillViewport = true }
        scroll.addOnLayoutChangeListener{view,_,top,_,bottom,_,oldTop,_,oldBottom->
            val height=bottom-top
            if(height>0 && height!=oldBottom-oldTop){list.setPadding(0,height/3,0,height*2/3);follow.layoutChanged()}
        }
        list = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL; gravity = if (style == UiStyle.APPLE) Gravity.START else Gravity.CENTER_HORIZONTAL }
        scroll.addView(list)
        pages=PlayerPager(this)
        val coverPage=LinearLayout(this).apply{orientation=LinearLayout.VERTICAL;gravity=Gravity.CENTER}
        artwork=ImageView(this).apply{
            scaleType=ImageView.ScaleType.FIT_CENTER
            background=UiTheme.shape(palette.surface,palette.radius,this@LyricsActivity)
            clipToOutline=true;setImageDrawable(MusicIcon("music",palette.accent))
        }
        val edge=minOf(resources.displayMetrics.widthPixels-UiTheme.dp(this,56f),resources.displayMetrics.heightPixels-UiTheme.dp(this,340f)).coerceAtLeast(100)
        coverPage.addView(artwork,LinearLayout.LayoutParams(edge,edge))
        pages.addView(coverPage);pages.addView(scroll)
        root.addView(pages,LinearLayout.LayoutParams(-1,0,1f))
        progress=SeekBar(this).apply{
            max=1000
            progressTintList=android.content.res.ColorStateList.valueOf(palette.accent)
            thumbTintList=android.content.res.ColorStateList.valueOf(palette.accent)
            setOnSeekBarChangeListener(object:SeekBar.OnSeekBarChangeListener{
                override fun onProgressChanged(bar:SeekBar?,value:Int,user:Boolean){}
                override fun onStartTrackingTouch(bar:SeekBar?){}
                override fun onStopTrackingTouch(bar:SeekBar?){startService(Intent(this@LyricsActivity,PlayerService::class.java).setAction("SEEK").putExtra("position",((bar?.progress ?:0).toLong()*PlayerService.durationMs/1000).toInt()))}
            })
        }
        root.addView(progress)
        time=TextView(this).apply{textSize=12f;setTextColor(palette.muted);setPadding(12,0,12,16)}
        root.addView(time)
        val bottom = LinearLayout(this).apply { gravity = Gravity.CENTER;setPadding(0,UiTheme.dp(this@LyricsActivity,16f),0,UiTheme.dp(this@LyricsActivity,8f)) }
        val previous = PlayerAction(this).apply { setIcon("previous",palette.text,"上一首");setOnClickListener { command("PREVIOUS") } }
        toggle = PlayerAction(this).apply {setIcon("play",palette.onAccent,"播放");setOnClickListener { command(if (PlayerService.isPlaying) "PAUSE" else "RESUME") } }
        val next = PlayerAction(this).apply {setIcon("next",palette.text,"下一首");setOnClickListener { command("NEXT") } }
        listOf(previous, toggle, next).forEach {
            UiTheme.feedback(it,this,palette,if(style==UiStyle.MIUIX)20f else 40f,if(it===toggle)palette.action else Color.TRANSPARENT)
            val size=if(it===toggle)72f else 56f
            bottom.addView(it,LinearLayout.LayoutParams(UiTheme.dp(this,size),UiTheme.dp(this,size)).apply{marginStart=UiTheme.dp(this@LyricsActivity,20f);marginEnd=UiTheme.dp(this@LyricsActivity,20f)})
        }
        root.addView(bottom)
    }

    private fun command(action: String) { startService(Intent(this, PlayerService::class.java).setAction(action)) }

    private fun refresh() {
        val label="${PlayerService.currentTitle}\n${PlayerService.currentArtist}"
        if(heading.text.toString()!=label)heading.text=android.text.SpannableString(label).apply{
            val start=PlayerService.currentTitle.length+1
            if(start<length)setSpan(android.text.style.RelativeSizeSpan(0.72f),start,length,android.text.Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
        }
        (toggle as PlayerAction).setIcon(if(PlayerService.isPlaying)"pause"else"play",palette.onAccent,if(PlayerService.isPlaying)"暂停"else"播放")
        progress.progress=if(PlayerService.durationMs>0)(PlayerService.positionMs.toLong()*1000/PlayerService.durationMs).toInt()else 0
        fun format(ms:Int)="%02d:%02d".format(ms/60000,ms/1000%60)
        val timeLabel="${format(PlayerService.positionMs)}   /   ${format(PlayerService.durationMs)}"
        if(time.text.toString()!=timeLabel)time.text=timeLabel
        if(coverUrl!=PlayerService.currentArtwork){
            coverUrl=PlayerService.currentArtwork;val expected=coverUrl
            if(expected.startsWith("https://"))worker.execute{
                val bitmap=ArtworkCache.load(this,expected)
                ui.post{if(!isFinishing && expected==coverUrl && bitmap!=null){artwork.setImageBitmap(bitmap);thumbnail.setImageBitmap(bitmap);artwork.alpha=0f;artwork.animate().alpha(1f).setDuration(UiTheme.duration(palette)).start()}}
            }
        }
        val source = PlayerService.currentLyric
        if (source != shownLyric) {
            shownLyric = source
            lines = Lrc.parse(source)
            active = -2
            render()
        }
        val current = Lrc.active(lines, PlayerService.positionMs)
        if(browsing && android.os.SystemClock.uptimeMillis()>=browsingUntil){browsing=false;updateLineFocus(true)}
        if (current != active) {
            active = current
            updateLineFocus(true)
        }
        if(current>=0 && pages.displayedChild==1){
            val target=list.getChildAt(current)
            if(target!=null && target.height>0 && scroll.height>0){
                val y=(list.top+target.top-scroll.height/3).coerceAtLeast(0)
                if(follow.requestScroll(android.os.SystemClock.uptimeMillis(),true,y))animateScroll(y)
            }
        }
    }
    private fun updateLineFocus(animated:Boolean){
        for(i in 0 until list.childCount){
            val line=list.getChildAt(i) as? LyricLineView ?: continue
            val visible=line.bottom>=scroll.scrollY-scroll.paddingTop && line.top<=scroll.scrollY+scroll.height
            line.focusDistance(if(active<0)0 else kotlin.math.abs(i-active),browsing,animated&&visible&&pages.displayedChild==1)
        }
    }
    private fun animateScroll(target:Int){
        scrollMotion?.cancel()
        if(UiTheme.duration(palette)==0L){scroll.scrollTo(0,target);return}
        scrollMotion=android.animation.ValueAnimator.ofInt(scroll.scrollY,target).apply{
            duration=UiTheme.lyricDuration(palette);interpolator=UiTheme.lyricInterpolator()
            addUpdateListener{scroll.scrollTo(0,it.animatedValue as Int)};start()
        }
    }
    private fun render() {
        list.removeAllViews()
        if (lines.isEmpty()) {
            val empty = TextView(this).apply { text = "暂无同步歌词"; textSize = 21f; gravity = Gravity.CENTER; setPadding(16, 80, 16, 80) }
            UiTheme.text(empty, palette, muted = true)
            list.addView(empty)
            return
        }
        val trans = Lrc.parse(PlayerService.currentTranslation)
        lines.forEachIndexed { index, line ->
            val match = trans.firstOrNull { kotlin.math.abs(it.timeMs - line.timeMs) <= 500 && it.text.isNotBlank() && it.text.trim()!="//" }
            val textView = LyricLineView(this,palette).apply {
                val full=line.text + if(match!=null)"\n${match.text}" else ""
                text=android.text.SpannableString(full).apply{if(match!=null)setSpan(android.text.style.RelativeSizeSpan(0.62f),line.text.length+1,full.length,android.text.Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)}
                textSize = if(style==UiStyle.APPLE)28f else 24f
                setTypeface(Typeface.create("sans-serif-medium",Typeface.NORMAL))
                setLineSpacing(UiTheme.dp(this@LyricsActivity,4f).toFloat(),1f)
                gravity = Gravity.START
                setPadding(UiTheme.dp(this@LyricsActivity, 12f), UiTheme.dp(this@LyricsActivity, 18f),
                    UiTheme.dp(this@LyricsActivity, 36f), UiTheme.dp(this@LyricsActivity, 18f))
                alpha = 1f
                setOnClickListener { startService(Intent(this@LyricsActivity, PlayerService::class.java)
                    .setAction("SEEK").putExtra("position", line.timeMs)) }
            }
            UiTheme.text(textView, palette, muted = true)
            list.addView(textView, LinearLayout.LayoutParams(-1, -2))
        }
    }
    override fun onResume() { super.onResume();running=true;browsing=false;active=-2;follow.resume();ui.removeCallbacks(tick);ui.post(tick) }
    override fun onPause() { running = false;scrollMotion?.cancel();ui.removeCallbacks(tick);super.onPause() }
    override fun dispatchTouchEvent(event:MotionEvent):Boolean {
        if(event.actionMasked==MotionEvent.ACTION_DOWN){
            downX=event.x;downY=event.y
            val region=android.graphics.Rect()
            lyricTouchStarted=::pages.isInitialized && pages.displayedChild==1 && scroll.getGlobalVisibleRect(region) && region.contains(event.rawX.toInt(),event.rawY.toInt())
            if(lyricTouchStarted){scrollMotion?.cancel();follow.touchDown();browsing=true;browsingUntil=Long.MAX_VALUE;updateLineFocus(true)}
        }
        // Observe the gesture before clickable lyric children consume DOWN/UP.
        if(lyricTouchStarted&&(event.actionMasked==MotionEvent.ACTION_UP||event.actionMasked==MotionEvent.ACTION_CANCEL)){
            lyricTouchStarted=false
            val now=android.os.SystemClock.uptimeMillis();follow.touchEnd(now);browsingUntil=now+1800
        }
        if(event.actionMasked==MotionEvent.ACTION_UP && kotlin.math.abs(event.x-downX)>UiTheme.dp(this,70f) && kotlin.math.abs(event.x-downX)>kotlin.math.abs(event.y-downY)*1.5f){
            val toLyrics=event.x<downX
            pages.showPage(if(toLyrics)1 else 0,if(UiTheme.duration(palette)==0L)0 else 360L,android.view.animation.PathInterpolator(0.2f,0f,0f,1f))
            captionForPage(toLyrics)
            thumbnail.visibility=if(toLyrics)android.view.View.VISIBLE else android.view.View.GONE
            progress.visibility=if(toLyrics)android.view.View.INVISIBLE else android.view.View.VISIBLE
            time.visibility=if(toLyrics)android.view.View.INVISIBLE else android.view.View.VISIBLE
            active=-2;browsing=false;browsingUntil=0;follow.resume()
            val cancel=MotionEvent.obtain(event);cancel.action=MotionEvent.ACTION_CANCEL;super.dispatchTouchEvent(cancel);cancel.recycle()
            return true
        }
        return super.dispatchTouchEvent(event)
    }
    private fun captionForPage(lyrics:Boolean){heading.textSize=if(lyrics)20f else 24f}
    @Deprecated("Deprecated in Java") override fun onBackPressed(){finishAfterTransition()}
    override fun onDestroy(){scrollMotion?.cancel();worker.shutdownNow();super.onDestroy()}
}
