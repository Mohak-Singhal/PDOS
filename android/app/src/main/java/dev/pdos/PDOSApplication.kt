package dev.pdos

import android.app.Application
import android.content.Context
import android.net.wifi.WifiManager
import android.os.Build
import android.util.Log

class PDOSApplication : Application() {

    private var multicastLock: WifiManager.MulticastLock? = null

    override fun onCreate() {
        super.onCreate()

        PDOSNative.setDeviceName(Build.MODEL)

        try {
            val wifi = applicationContext.getSystemService(Context.WIFI_SERVICE) as WifiManager
            multicastLock = wifi.createMulticastLock("PDOS")
            multicastLock?.setReferenceCounted(false)
            multicastLock?.acquire()
            Log.d("PDOS", "MulticastLock acquired")
        } catch (e: Throwable) {
            Log.e("PDOS", "Failed to acquire MulticastLock", e)
        }
    }

    override fun onTerminate() {
        multicastLock?.let {
            if (it.isHeld) {
                it.release()
                Log.d("PDOS", "MulticastLock released")
            }
        }
        super.onTerminate()
    }
}