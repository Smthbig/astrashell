package com.astrashell.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.astrashell.ui.theme.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(onBack: () -> Unit) {
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Settings", fontWeight = FontWeight.Bold) },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.Default.ArrowBack, "Back")
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(containerColor = DarkNav)
            )
        },
        containerColor = DarkBackground
    ) { padding ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            item {
                Text("Execution Engine", style = MaterialTheme.typography.titleSmall, color = OnSurfaceVariant)
            }
            item { SettingsCard("Engine Mode", "Auto-detect", "Native / Container / VM") }
            item { SettingsCard("Rootfs Path", "Default", "Change root filesystem location") }
            item { SettingsCard("Network Mode", "Slirp", "User-mode networking") }

            item {
                Spacer(Modifier.height(8.dp))
                Text("Container", style = MaterialTheme.typography.titleSmall, color = OnSurfaceVariant)
            }
            item { SettingsCard("Init System", "None", "systemd / OpenRC / runit / s6") }
            item { SettingsCard("Max Containers", "4", "Maximum parallel containers") }
            item { SettingsCard("OverlayFS", "Enabled", "Copy-on-write filesystem") }

            item {
                Spacer(Modifier.height(8.dp))
                Text("GPU & Display", style = MaterialTheme.typography.titleSmall, color = OnSurfaceVariant)
            }
            item { SettingsCard("Display Server", "None", "Wayland / X11 / Web") }
            item { SettingsCard("GPU Acceleration", "Disabled", "Vulkan / OpenGL forwarding") }

            item {
                Spacer(Modifier.height(8.dp))
                Text("Security", style = MaterialTheme.typography.titleSmall, color = OnSurfaceVariant)
            }
            item { SettingsCard("Seccomp Profile", "Default", "System call filtering") }
            item { SettingsCard("Readonly Rootfs", "Off", "Immutable base layer") }
            item { SettingsCard("Landlock", "Disabled", "LSM-based sandboxing") }

            item {
                Spacer(Modifier.height(16.dp))
                Button(
                    onClick = { /* save */ },
                    modifier = Modifier.fillMaxWidth(),
                    colors = ButtonDefaults.buttonColors(containerColor = AccentBlue)
                ) {
                    Icon(Icons.Default.Save, null)
                    Spacer(Modifier.width(8.dp))
                    Text("Save Settings")
                }
            }
        }
    }
}

@Composable
private fun SettingsCard(title: String, value: String, description: String) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(containerColor = DarkSurface),
        shape = RoundedCornerShape(12.dp)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(title, fontWeight = FontWeight.Medium)
                Text(description, color = OnSurfaceVariant, style = MaterialTheme.typography.bodySmall)
            }
            Text(value, color = AccentBlue, style = MaterialTheme.typography.bodyMedium)
        }
    }
}
