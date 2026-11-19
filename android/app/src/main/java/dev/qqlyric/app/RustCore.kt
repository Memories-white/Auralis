package dev.qqlyric.app

object RustCore {
    init { System.loadLibrary("qqlyric_core") }
    external fun call(command: String, input: String): String
}
