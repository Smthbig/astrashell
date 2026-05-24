package com.astrashell.engine

/** Detection and selection of execution engines */
object EngineDetector {
    enum class Engine(val description: String) {
        NATIVE("Native fast-path with seccomp"),
        CONTAINER("Full namespace isolation"),
        AVF("AVF-backed microVM"),
        HYBRID("Auto-selected best mode")
    }

    data class EngineCapabilities(
        val hasSeccomp: Boolean = false,
        val hasSeccompNotify: Boolean = false,
        val hasUserNamespaces: Boolean = false,
        val hasOverlayFS: Boolean = false,
        val hasFUSE: Boolean = false,
        val hasAVF: Boolean = false,
        val hasKVM: Boolean = false,
        val hasCgroupsV2: Boolean = false
    )

    /** Detect all available engines */
    external fun detectCapabilities(): EngineCapabilities

    /** Select best available engine */
    fun selectBest(caps: EngineCapabilities): Engine {
        return when {
            caps.hasAVF -> Engine.AVF
            caps.hasUserNamespaces -> Engine.CONTAINER
            caps.hasSeccomp || caps.hasSeccompNotify -> Engine.NATIVE
            else -> Engine.NATIVE
        }
    }
}
