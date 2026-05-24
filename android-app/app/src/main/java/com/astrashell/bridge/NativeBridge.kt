package com.astrashell.bridge

/** JNI bridge to native Rust runtime */
object NativeBridge {
    private var loaded = false

    fun ensureLoaded() {
        if (!loaded) {
            System.loadLibrary("astrashell")
            loaded = true
        }
    }

    /** Initialize the runtime with given config */
    external fun runtimeInit(configJson: String): Boolean

    /** Start the runtime */
    external fun runtimeStart(): Boolean

    /** Stop the runtime */
    external fun runtimeStop(): Boolean

    /** Get runtime status */
    external fun runtimeStatus(): String

    /** Execute a command in the runtime */
    external fun runtimeExec(command: String, args: Array<String>): Int

    /** Execute a command and get output */
    external fun runtimeExecOutput(command: String, args: Array<String>): String

    /** Mount an overlay filesystem */
    external fun overlayMount(lower: String, upper: String, work: String, merged: String): Boolean

    /** Unmount overlay */
    external fun overlayUnmount(merged: String): Boolean

    /** Check kernel capabilities */
    external fun checkCapabilities(): String

    /** Create a user namespace (fake root) */
    external fun createUserNamespace(uidMap: String, gidMap: String): Boolean

    /** Initialize AVF VM */
    external fun avfInit(config: String): Int

    /** Start AVF VM */
    external fun avfStart(vmId: Int): Boolean

    /** Stop AVF VM */
    external fun avfStop(vmId: Int): Boolean

    /** Run command in AVF VM via vsock */
    external fun avfExec(vmId: Int, command: String): String

    /** Pull an OCI image */
    external fun ociPull(imageRef: String): Boolean

    /** List local images */
    external fun ociList(): String

    /** Get runtime version */
    external fun getVersion(): String
}
