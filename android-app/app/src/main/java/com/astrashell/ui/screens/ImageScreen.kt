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
import androidx.compose.ui.unit.sp
import com.astrashell.ui.theme.*

data class DistroImage(
    val id: String,
    val name: String,
    val version: String,
    val size: String,
    val status: String
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ImageScreen(
    onBack: () -> Unit,
    onUseImage: (String) -> Unit
) {
    var images by remember {
        mutableStateOf(listOf(
            DistroImage("1", "Arch Linux", "2025.04.01", "1.2 GB", "Downloaded"),
            DistroImage("2", "Ubuntu", "24.04 LTS", "2.1 GB", "Downloaded"),
            DistroImage("3", "Alpine", "3.19", "85 MB", "Downloaded"),
            DistroImage("4", "Debian", "12 Bookworm", "1.8 GB", "Available"),
            DistroImage("5", "Fedora", "40", "2.3 GB", "Available"),
        ))
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Images", fontWeight = FontWeight.Bold) },
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
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            item {
                // Import card
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    colors = CardDefaults.cardColors(containerColor = SurfaceVariant),
                    shape = RoundedCornerShape(12.dp),
                    onClick = { /* import image */ }
                ) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(16.dp),
                        horizontalArrangement = Arrangement.Center,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Icon(Icons.Default.FileOpen, "Import", tint = AccentBlue)
                        Spacer(Modifier.width(12.dp))
                        Text("Import OCI Image", color = AccentBlue, fontWeight = FontWeight.Medium)
                    }
                }
            }

            items(images) { image ->
                ImageCard(image, onUse = { onUseImage(image.id) })
            }
        }
    }
}

@Composable
private fun ImageCard(image: DistroImage, onUse: () -> Unit) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(containerColor = DarkSurface),
        shape = RoundedCornerShape(12.dp)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(image.name, fontWeight = FontWeight.Bold)
                Text(image.version, color = OnSurfaceVariant, style = MaterialTheme.typography.bodySmall)
                Spacer(Modifier.height(4.dp))
                Text("${image.size} | ${image.status}", color = OnSurfaceVariant, fontFamily = FontFamily.Monospace, fontSize = 11.sp)
            }
            Column(horizontalAlignment = Alignment.CenterVertically) {
                if (image.status == "Downloaded") {
                    FilledTonalButton(
                        onClick = onUse,
                        colors = ButtonDefaults.filledTonalButtonColors(containerColor = TerminalGreen.copy(alpha = 0.2f))
                    ) {
                        Text("Use", color = TerminalGreen)
                    }
                } else {
                    FilledTonalButton(
                        onClick = { /* download */ },
                        colors = ButtonDefaults.filledTonalButtonColors(containerColor = AccentBlue.copy(alpha = 0.2f))
                    ) {
                        Icon(Icons.Default.Download, null, modifier = Modifier.size(16.dp))
                        Spacer(Modifier.width(4.dp))
                        Text("Pull", color = AccentBlue, fontSize = 12.sp)
                    }
                }
            }
        }
    }
}
