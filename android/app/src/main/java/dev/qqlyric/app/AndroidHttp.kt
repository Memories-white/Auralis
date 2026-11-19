package dev.qqlyric.app

import android.util.Base64
import org.json.JSONArray
import org.json.JSONObject
import java.net.URL
import javax.net.ssl.HttpsURLConnection

/** Uses the platform DNS/proxy/TLS path; called only from the Rust worker. */
object AndroidHttp {
    @JvmStatic fun execute(input:String):String {
        var connection:HttpsURLConnection?=null
        var host="QQ 服务"
        return try {
            val request=JSONObject(input)
            val url=URL(request.getString("url"));host=url.host
            require(url.protocol=="https"){"仅支持 HTTPS"}
            connection=(url.openConnection() as HttpsURLConnection).apply{
                connectTimeout=15000;readTimeout=15000;instanceFollowRedirects=false
                requestMethod=request.getString("method")
                val headers=request.getJSONObject("headers")
                for(key in headers.keys())setRequestProperty(key,headers.getString(key))
            }
            if(connection.requestMethod=="POST"){
                val body=Base64.decode(request.optString("body"),Base64.NO_WRAP)
                connection.doOutput=true;connection.setFixedLengthStreamingMode(body.size)
                connection.outputStream.use{it.write(body)}
            }
            val status=connection.responseCode
            val headers=JSONObject()
            for((key,values)in connection.headerFields)if(key!=null && values!=null)headers.put(key,JSONArray(values))
            val stream=if(status>=400)connection.errorStream else connection.inputStream
            val bytes=stream?.use{
                val output=java.io.ByteArrayOutputStream();val buffer=ByteArray(8192)
                while(true){val count=it.read(buffer);if(count<0)break;require(output.size()+count<=8*1024*1024){"服务响应过大"};output.write(buffer,0,count)}
                output.toByteArray()
            } ?: byteArrayOf()
            JSONObject().put("status",status).put("headers",headers).put("body",Base64.encodeToString(bytes,Base64.NO_WRAP)).toString()
        }catch(e:Exception){
            val message=when(e){
                is java.net.UnknownHostException->"无法解析 $host（Android 网络），请检查当前网络后重试"
                is java.net.SocketTimeoutException->"连接 $host 超时，请重试"
                is javax.net.ssl.SSLException->"连接 $host 时证书或 TLS 握手失败"
                else->"连接 $host 失败（${e.javaClass.simpleName}）"
            }
            JSONObject().put("error",message).toString()
        }finally{connection?.disconnect()}
    }
}
