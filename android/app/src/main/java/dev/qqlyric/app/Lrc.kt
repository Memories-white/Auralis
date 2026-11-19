package dev.qqlyric.app

data class LyricLine(val timeMs: Int, val text: String)

object Lrc {
    private val stamp = Regex("\\[(\\d{1,2}):(\\d{2})(?:[.:](\\d{1,3}))?]")
    fun parse(source: String): List<LyricLine> = source.lineSequence().flatMap { raw ->
        val matches = stamp.findAll(raw).toList()
        val words = stamp.replace(raw, "").trim()
        matches.asSequence().map { match ->
            val min = match.groupValues[1].toIntOrNull() ?: 0
            val sec = match.groupValues[2].toIntOrNull() ?: 0
            val frac = match.groupValues[3].padEnd(3, '0').take(3).toIntOrNull() ?: 0
            LyricLine((min * 60 + sec) * 1000 + frac, words)
        }
    }.filter { it.text.isNotEmpty() }.sortedBy { it.timeMs }.toList()

    fun active(lines: List<LyricLine>, positionMs: Int): Int {
        if (lines.isEmpty()) return -1
        var low = 0
        var high = lines.lastIndex
        var found = -1
        while (low <= high) {
            val middle = (low + high) ushr 1
            if (lines[middle].timeMs <= positionMs) { found = middle; low = middle + 1 }
            else high = middle - 1
        }
        return found
    }
}
