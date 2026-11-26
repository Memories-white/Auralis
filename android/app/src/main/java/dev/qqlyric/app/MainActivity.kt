package dev.qqlyric.app

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.*

/** Android lifecycle and system entry points; the application UI is Rust/WASM. */
class MainActivity:ComponentActivity(){
    private lateinit var music:MusicState
    private var openRequested by mutableStateOf(false)
    override fun onCreate(savedInstanceState:Bundle?){
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        music=MusicState(this)
        openRequested=intent.getBooleanExtra("open_player",false)
        setContent{RustApp(music,openRequested){openRequested=false}}
    }
    override fun onNewIntent(intent:Intent){
        super.onNewIntent(intent)
        setIntent(intent)
        openRequested=intent.getBooleanExtra("open_player",false)
    }
    override fun onDestroy(){music.close();super.onDestroy()}
}
