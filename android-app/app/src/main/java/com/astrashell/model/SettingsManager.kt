package com.astrashell.model

import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.*
import androidx.datastore.preferences.preferencesDataStore
import com.astrashell.AstraShellApp
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

private val Context.dataStore: DataStore<Preferences> by preferencesDataStore(name = "astrashell_settings")

object SettingsManager {
    private val context = AstraShellApp.instance
    private val dataStore = context.dataStore

    object Keys {
        val ENGINE_MODE = stringPreferencesKey("engine_mode")
        val ROOTFS_PATH = stringPreferencesKey("rootfs_path")
        val NETWORK_MODE = stringPreferencesKey("network_mode")
        val INIT_SYSTEM = stringPreferencesKey("init_system")
        val MAX_CONTAINERS = intPreferencesKey("max_containers")
        val ENABLE_OVERLAY = booleanPreferencesKey("enable_overlay")
        val ENABLE_GPU = booleanPreferencesKey("enable_gpu")
        val DISPLAY_SERVER = stringPreferencesKey("display_server")
        val ENABLE_SOUND = booleanPreferencesKey("enable_sound")
        val SECCOMP_PROFILE = stringPreferencesKey("seccomp_profile")
        val READONLY_ROOTFS = booleanPreferencesKey("readonly_rootfs")
        val FONT_SIZE = floatPreferencesKey("font_size")
        val THEME = stringPreferencesKey("theme")
        val AUTO_START = booleanPreferencesKey("auto_start")
    }

    data class Settings(
        val engineMode: String = "auto",
        val rootfsPath: String = "",
        val networkMode: String = "slirp",
        val initSystem: String = "none",
        val maxContainers: Int = 4,
        val enableOverlay: Boolean = true,
        val enableGpu: Boolean = false,
        val displayServer: String = "none",
        val enableSound: Boolean = false,
        val seccompProfile: String = "default",
        val readonlyRootfs: Boolean = false,
        val fontSize: Float = 12f,
        val theme: String = "dark",
        val autoStart: Boolean = false
    )

    val settingsFlow: Flow<Settings> = dataStore.data.map { prefs ->
        Settings(
            engineMode = prefs[Keys.ENGINE_MODE] ?: "auto",
            rootfsPath = prefs[Keys.ROOTFS_PATH] ?: "",
            networkMode = prefs[Keys.NETWORK_MODE] ?: "slirp",
            initSystem = prefs[Keys.INIT_SYSTEM] ?: "none",
            maxContainers = prefs[Keys.MAX_CONTAINERS] ?: 4,
            enableOverlay = prefs[Keys.ENABLE_OVERLAY] ?: true,
            enableGpu = prefs[Keys.ENABLE_GPU] ?: false,
            displayServer = prefs[Keys.DISPLAY_SERVER] ?: "none",
            enableSound = prefs[Keys.ENABLE_SOUND] ?: false,
            seccompProfile = prefs[Keys.SECCOMP_PROFILE] ?: "default",
            readonlyRootfs = prefs[Keys.READONLY_ROOTFS] ?: false,
            fontSize = prefs[Keys.FONT_SIZE] ?: 12f,
            theme = prefs[Keys.THEME] ?: "dark",
            autoStart = prefs[Keys.AUTO_START] ?: false
        )
    }
}
