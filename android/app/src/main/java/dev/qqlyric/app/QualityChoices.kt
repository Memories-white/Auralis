package dev.qqlyric.app

import androidx.compose.foundation.layout.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import top.yukonga.miuix.kmp.basic.Text
import top.yukonga.miuix.kmp.preference.*
import top.yukonga.miuix.kmp.theme.MiuixTheme

fun qualityLabel(value:String)=when(value){"sq"->R.string.quality_sq;"hq"->R.string.quality_hq;"standard"->R.string.quality_standard;else->R.string.quality_auto}
@Composable fun qualityBadge(value:String,downgraded:Boolean):String {
    val label=when(value){"sq"->"SQ";"hq"->"HQ";"standard"->stringResource(R.string.quality_standard_short);else->stringResource(R.string.audio_quality)}
    return if(downgraded)"$label ↓"else label
}
@Composable fun QualityChoices(m:MusicState){
    val colors=MiuixTheme.colorScheme
    Column(Modifier.fillMaxWidth()){
        val current=if(m.actualQuality.isBlank())stringResource(R.string.quality_none)else stringResource(R.string.quality_actual,stringResource(qualityLabel(m.actualQuality)))
        Text(if(m.qualityPending)stringResource(R.string.quality_requesting)else current,Modifier.padding(horizontal=20.dp,vertical=12.dp),style=MiuixTheme.textStyles.footnote1,color=colors.onSurfaceVariantSummary)
        if(m.qualityDowngraded)Text(stringResource(R.string.quality_downgraded),Modifier.padding(horizontal=20.dp).padding(bottom=8.dp),style=MiuixTheme.textStyles.footnote1,color=colors.primary)
        listOf("auto","sq","hq","standard").forEach{value->
            RadioButtonPreference(title=stringResource(qualityLabel(value)),summary=stringResource(when(value){"auto"->R.string.quality_auto_summary;"sq"->R.string.quality_sq_summary;"hq"->R.string.quality_hq_summary;else->R.string.quality_standard_summary}),selected=m.audioQuality==value,onClick={m.chooseAudioQuality(value)})
        }
        SwitchPreference(checked=m.qualityFallback,onCheckedChange=m::allowQualityFallback,title=stringResource(R.string.quality_fallback),summary=stringResource(R.string.quality_fallback_summary),enabled=m.audioQuality=="sq"||m.audioQuality=="hq")
        Text(stringResource(R.string.quality_apply_summary),Modifier.padding(20.dp),style=MiuixTheme.textStyles.footnote1,color=colors.onSurfaceVariantSummary)
    }
}
