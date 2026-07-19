package dev.pdos

object PDOSNative {

    init {
        System.loadLibrary("runtime")
    }

    external fun startRuntime(filesDir: String)

    external fun stopRuntime()

    external fun runtimeStatus(): Int

    external fun protocolVersion(): Int

    external fun runtimeVersion(): String

    external fun connectedNodes(): Int
}