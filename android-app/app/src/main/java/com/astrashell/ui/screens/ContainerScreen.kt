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
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.astrashell.ui.theme.*

data class ContainerInfo(
    val id: String,
    val name: String,
    val distro: String,
    val status: String,
    val uptime: String
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ContainerScreen(onBack: () -> Unit) {
    var containers by remember {
        mutableStateOf(listOf(
            ContainerInfo("1", "arch-main", "Arch Linux", "Running", "2h 15m"),
            ContainerInfo("2", "ubuntu-dev", "Ubuntu 24.04", "Stopped", "-"),
            ContainerInfo("3", "alpine-tiny", "Alpine 3.19", "Running", "45m"),
        ))
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Containers", fontWeight = FontWeight.Bold) },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.Default.ArrowBack, "Back")
                    }
                },
                actions = {
                    IconButton(onClick = { /* add container */ }) {
                        Icon(Icons.Default.Add, "Add")
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(containerColor = DarkNav)
            )
        },
        containerColor = DarkBackground,
        floatingActionButton = {
            ExtendedFloatingActionButton(
                onClick = { /* new container */ },
                containerColor = AccentBlue,
                contentColor = DarkBackground
            ) {
                Icon(Icons.Default.Add, null)
                Spacer(Modifier.width(8.dp))
                Text("New")
            }
        }
    ) { padding ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            items(containers) { container ->
                ContainerCard(container, onStart = {
                    containers = containers.map {
                        if (it.id == container.id) it.copy(status = "Running", uptime = "0m") else it
                    }
                }, onStop = {
                    containers = containers.map {
                        if (it.id == container.id) it.copy(status = "Stopped", uptime = "-") else it
                    }
                })
            }
        }
    }
}

@Composable
private fun ContainerCard(
    container: ContainerInfo,
    onStart: () -> Unit,
    onStop: () -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(containerColor = DarkSurface),
        shape = RoundedCornerShape(12.dp)
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column {
                    Text(container.name, fontWeight = FontWeight.Bold, fontSize = MaterialTheme.typography.titleMedium.fontSize)
                    Text(container.distro, color = OnSurfaceVariant, fontSize = MaterialTheme.typography.bodySmall.fontSize)
                }
                Surface(
                    shape = RoundedCornerShape(6.dp),
                    color = if (container.status == "Running") TerminalGreen.copy(alpha = 0.2f) else TerminalRed.copy(alpha = 0.2f)
                ) {
                    Text(
                        container.status,
                        modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
                        color = if (container.status == "Running") TerminalGreen else TerminalRed,
                        style = MaterialTheme.typography.bodySmall,
                        fontWeight = FontWeight.Bold
                    )
                }
            }
            Spacer(Modifier.height(8.dp))
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Text("Uptime: ${container.uptime}", color = OnSurfaceVariant, fontFamily = FontFamily.Monospace, fontSize = 12.sp)
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    if (container.status == "Running") {
                        FilledTonalButton(onClick = onStop, colors = ButtonDefaults.filledTonalButtonColors(containerColor = TerminalRed.copy(alpha = 0.2f))) {
                            Icon(Icons.Default.Stop, null, modifier = Modifier.size(16.dp))
                            Spacer(Modifier.width(4.dp))
                            Text("Stop", fontSize = 12.sp)
                        }
                    } else {
                        FilledTonalButton(onClick = onStart, colors = ButtonDefaults.filledTonalButtonColors(containerColor = TerminalGreen.copy(alpha = 0.2f))) {
                            Icon(Icons.Default.PlayArrow, null, modifier = Modifier.size(16.dp))
                            Spacer(Modifier.width(4.dp))
                            Text("Start", fontSize = 12.sp)
                        }
                    }
                    FilledTonalButton(onClick = { /* terminal */ }) {
                        Icon(Icons.Default.Terminal, null, modifier = Modifier.size(16.dp))
                    }
                }
            }
        }
    }
}
