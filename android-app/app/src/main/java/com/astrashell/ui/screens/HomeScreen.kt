package com.astrashell.ui.screens

import androidx.compose.animation.*
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.astrashell.ui.theme.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(
    onNavigateToTerminal: () -> Unit,
    onNavigateToSettings: () -> Unit,
    onNavigateToContainers: () -> Unit,
    onNavigateToImages: () -> Unit
) {
    var runtimeStatus by remember { mutableStateOf("Stopped") }
    var currentEngine by remember { mutableStateOf("auto-detect") }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text("AstraShell", fontWeight = FontWeight.Bold)
                },
                actions = {
                    IconButton(onClick = onNavigateToSettings) {
                        Icon(Icons.Default.Settings, "Settings")
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = DarkNav
                )
            )
        },
        containerColor = DarkBackground
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            // Status card
            Card(
                modifier = Modifier.fillMaxWidth(),
                colors = CardDefaults.cardColors(containerColor = DarkSurface)
            ) {
                Column(modifier = Modifier.padding(20.dp)) {
                    Text("Runtime Status", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(8.dp))
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        Text("Engine:", color = OnSurfaceVariant)
                        Text(currentEngine, fontFamily = FontFamily.Monospace)
                    }
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        Text("Status:", color = OnSurfaceVariant)
                        val statusColor = if (runtimeStatus == "Running") TerminalGreen else TerminalRed
                        Text(runtimeStatus, color = statusColor, fontWeight = FontWeight.Bold)
                    }
                }
            }

            // Quick actions
            Text("Quick Actions", style = MaterialTheme.typography.titleSmall, color = OnSurfaceVariant)

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                ActionButton(
                    icon = Icons.Default.Terminal,
                    label = "Terminal",
                    onClick = onNavigateToTerminal,
                    modifier = Modifier.weight(1f)
                )
                ActionButton(
                    icon = Icons.Default.Container,
                    label = "Containers",
                    onClick = onNavigateToContainers,
                    modifier = Modifier.weight(1f)
                )
                ActionButton(
                    icon = Icons.Default.Save,
                    label = "Images",
                    onClick = onNavigateToImages,
                    modifier = Modifier.weight(1f)
                )
            }

            // Status cards
            Text("System", style = MaterialTheme.typography.titleSmall, color = OnSurfaceVariant)

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                InfoCard("Memory", "Available", Modifier.weight(1f))
                InfoCard("CPU", "Idle", Modifier.weight(1f))
                InfoCard("Storage", "OK", Modifier.weight(1f))
            }

            Spacer(Modifier.weight(1f))

            // Start/Stop button
            Button(
                onClick = {
                    runtimeStatus = if (runtimeStatus == "Running") "Stopped" else "Running"
                },
                modifier = Modifier
                    .fillMaxWidth()
                    .height(56.dp),
                colors = ButtonDefaults.buttonColors(
                    containerColor = if (runtimeStatus == "Running") TerminalRed else TerminalGreen
                ),
                shape = RoundedCornerShape(12.dp)
            ) {
                Icon(
                    if (runtimeStatus == "Running") Icons.Default.Stop else Icons.Default.PlayArrow,
                    contentDescription = null
                )
                Spacer(Modifier.width(8.dp))
                Text(
                    if (runtimeStatus == "Running") "Stop Runtime" else "Start Runtime",
                    fontSize = 16.sp
                )
            }
        }
    }
}

@Composable
private fun ActionButton(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    Card(
        modifier = modifier.clickable(onClick = onClick),
        colors = CardDefaults.cardColors(containerColor = SurfaceVariant),
        shape = RoundedCornerShape(12.dp)
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Icon(icon, label, tint = AccentBlue, modifier = Modifier.size(28.dp))
            Spacer(Modifier.height(8.dp))
            Text(label, fontSize = 12.sp, color = OnSurfaceVariant)
        }
    }
}

@Composable
private fun InfoCard(title: String, value: String, modifier: Modifier = Modifier) {
    Card(
        modifier = modifier,
        colors = CardDefaults.cardColors(containerColor = SurfaceVariant)
    ) {
        Column(
            modifier = Modifier.padding(12.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Text(value, fontSize = 18.sp, fontWeight = FontWeight.Bold, color = TerminalGreen)
            Text(title, fontSize = 11.sp, color = OnSurfaceVariant)
        }
    }
}
