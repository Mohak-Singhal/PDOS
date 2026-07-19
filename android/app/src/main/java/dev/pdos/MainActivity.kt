package dev.pdos

import android.app.Activity
import android.os.Bundle
import android.util.Log

class MainActivity : Activity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        try {
            Log.d("PDOS", "Loading Runtime...")

            PDOSNative.startRuntime(filesDir = filesDir.absolutePath)

            Log.d("PDOS", "Version: ${PDOSNative.runtimeVersion()}")
            Log.d("PDOS", "Protocol: ${PDOSNative.protocolVersion()}")
            Log.d("PDOS", "Status: ${PDOSNative.runtimeStatus()}")
            Log.d("PDOS", "Nodes: ${PDOSNative.connectedNodes()}")

        } catch (e: Throwable) {
            Log.e("PDOS", "Failed", e)
        }
    }
}