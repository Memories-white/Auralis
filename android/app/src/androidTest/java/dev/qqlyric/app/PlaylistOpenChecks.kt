package dev.qqlyric.app

import android.app.Activity
import android.app.Instrumentation
import android.os.Bundle
import android.os.SystemClock
import org.json.JSONObject
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger

/** Controlled slow/failing responses exercise entry behavior without a live service. */
internal fun Instrumentation.runPlaylistOpenChecks(){
    val firstStarted=CountDownLatch(1);val firstRelease=CountDownLatch(1)
    val secondStarted=CountDownLatch(1);val secondRelease=CountDownLatch(1)
    val thirdStarted=CountDownLatch(1);val thirdRelease=CountDownLatch(1)
    val guesses=AtomicInteger()
    var model:MusicState?=null
    val report=Bundle()
    var result=Activity.RESULT_CANCELED
    fun await(latch:CountDownLatch){check(latch.await(5,TimeUnit.SECONDS)){"Request did not reach expected phase"}}
    fun awaitMain(label:String,condition:()->Boolean){
        val deadline=SystemClock.elapsedRealtime()+5000
        while(true){var ready=false;runOnMainSync{ready=condition()};if(ready)return;check(SystemClock.elapsedRealtime()<deadline){label};Thread.sleep(20)}
    }
    val transport:(String,String)->String={name,_->when(name){
        "guess"->if(guesses.incrementAndGet()==1){
            firstStarted.countDown();await(firstRelease)
            """{"ok":false,"error":"连接超时，请重试"}"""
        }else{
            secondStarted.countDown();await(secondRelease)
            """{"ok":true,"data":[{"mid":"stale-result","title":"Stale"}]}"""
        }
        "playlist"->{thirdStarted.countDown();await(thirdRelease);"""{"ok":true,"data":[{"mid":"current-result","title":"Current"}]}"""}
        "home"->"""{"ok":true,"data":{"cards":[],"daily":[]}}"""
        "recommend"->"""{"ok":true,"data":{"songs":[],"hasMore":false}}"""
        else->"""{"ok":false,"error":"Unused by this test"}"""
    }}
    try{
        val card=JSONObject().put("kind","guess").put("id",99).put("title","猜你喜欢-沉浸刷歌")
        runOnMainSync{
            model=MusicState(targetContext,transport)
            model!!.loggedIn=true
            model!!.recommendedPlaylist(card)
            check(model!!.page=="songs"&&model!!.detailLoading&&model!!.tracks.isEmpty()){"Entry waited for the network"}
            val revision=model!!.detailRevision
            model!!.recommendedPlaylist(card)
            check(model!!.detailRevision==revision){"Duplicate tap reopened the page"}
        }
        await(firstStarted);check(guesses.get()==1)
        firstRelease.countDown()
        awaitMain("Failure did not appear on the detail page"){!model!!.detailLoading&&model!!.detailError.isNotBlank()}
        runOnMainSync{
            check(model!!.page=="songs")
            model!!.retryPlaylist()
            check(model!!.detailLoading&&model!!.detailError.isBlank())
        }
        await(secondStarted)
        runOnMainSync{
            model!!.back();check(model!!.page=="home"&&!model!!.detailLoading)
            model!!.playlist(42,0,"New playlist")
            check(model!!.page=="songs"&&model!!.detailLoading)
        }
        secondRelease.countDown();await(thirdStarted)
        // The serial detail worker has queued the old callback before starting request three.
        runOnMainSync{check(model!!.detailTitle=="New playlist"&&model!!.tracks.isEmpty()&&model!!.detailLoading){"Late response replaced the new destination"}}
        thirdRelease.countDown()
        awaitMain("Current response was not applied"){!model!!.detailLoading&&model!!.tracks.size==1}
        runOnMainSync{check(model!!.tracks.single().optString("mid")=="current-result"&&model!!.detailError.isBlank())}
        report.putString("stream","PASS: immediate playlist entry, loading, duplicate taps, inline failure, retry, back and stale-response rejection.\n")
        result=Activity.RESULT_OK
    }catch(error:Throwable){report.putString("stream","Playlist checks failed: ${error.message}\n")}
    finally{
        runOnMainSync{model?.close()}
        firstRelease.countDown();secondRelease.countDown();thirdRelease.countDown()
        finish(result,report)
    }
}
