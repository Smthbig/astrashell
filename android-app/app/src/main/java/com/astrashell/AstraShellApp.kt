package com.astrashell

import android.app.Application
import android.content.Context
import dagger.hilt.android.HiltAndroidApp

@HiltAndroidApp
class AstraShellApp : Application() {
    companion object {
        lateinit var instance: AstraShellApp
            private set
    }

    override fun onCreate() {
        super.onCreate()
        instance = this
        initializeNativeRuntime()
    }

    private fun initializeNativeRuntime() {
        // Load the native runtime library
        System.loadLibrary("astrashell")
    }
}
