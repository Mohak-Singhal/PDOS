package dev.pdos

import android.app.Service
import android.content.Intent
import android.os.IBinder
import android.util.Log

class RuntimeService : Service() {

    private var started = false

    override fun onCreate() {
        super.onCreate()

        Log.d("PDOS", "RuntimeService created")
    }

    override fun onStartCommand(
        intent: Intent?,
        flags: Int,
        startId: Int
    ): Int {

        if (!started) {

            started = true

            Log.d("PDOS", "Starting Rust runtime...")

            PDOSNative.startRuntime(filesDir = filesDir.absolutePath)

            Log.d("PDOS", "Version: ${PDOSNative.runtimeVersion()}")
            Log.d("PDOS", "Protocol: ${PDOSNative.protocolVersion()}")
            Log.d("PDOS", "Status: ${PDOSNative.runtimeStatus()}")
            Log.d("PDOS", "Nodes: ${PDOSNative.connectedNodes()}")
        }

        return START_STICKY
    }

    override fun onDestroy() {

        Log.d("PDOS", "Stopping Rust runtime")

        PDOSNative.stopRuntime()

        started = false

        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null
}