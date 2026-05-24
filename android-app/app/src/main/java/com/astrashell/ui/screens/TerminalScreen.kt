package com.astrashell.ui.screens

import android.graphics.Typeface
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import com.astrashell.ui.theme.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TerminalScreen(
    onBack: () -> Unit
) {
    var fontSize by remember { mutableFloatStateOf(12f) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Terminal", fontWeight = FontWeight.Bold) },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.Default.ArrowBack, "Back")
                    }
                },
                actions = {
                    IconButton(onClick = { fontSize = (fontSize - 1f).coerceAtLeast(8f) }) {
                        Icon(Icons.Default.TextDecrease, "Decrease font")
                    }
                    IconButton(onClick = { fontSize = (fontSize + 1f).coerceAtMost(24f) }) {
                        Icon(Icons.Default.TextIncrease, "Increase font")
                    }
                    IconButton(onClick = { /* clear */ }) {
                        Icon(Icons.Default.DeleteSweep, "Clear")
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = DarkNav
                )
            )
        },
        containerColor = TerminalBg
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
        ) {
            // Terminal view (AndroidView wrapping Termux's TerminalView)
            Box(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxWidth()
                    .background(TerminalBg)
            ) {
                // Terminal emulator surface
                // In production, this uses Termux's TerminalView
                Column(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(8.dp)
                ) {
                    Text(
                        "AstraShell Terminal v0.1",
                        color = TerminalGreen,
                        fontFamily = FontFamily.Monospace,
                        fontSize = (fontSize + 2).sp
                    )
                    Text(
                        "Type 'help' for commands",
                        color = OnSurfaceVariant,
                        fontFamily = FontFamily.Monospace,
                        fontSize = fontSize.sp
                    )
                    Spacer(Modifier.height(16.dp))
                    Text(
                        "$ ",
                        color = TerminalGreen,
                        fontFamily = FontFamily.Monospace,
                        fontSize = fontSize.sp
                    )
                }
            }

            // Input bar
            Surface(
                modifier = Modifier.fillMaxWidth(),
                color = DarkSurface,
                tonalElevation = 4.dp
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    OutlinedTextField(
                        value = "",
                        onValueChange = { },
                        modifier = Modifier.weight(1f),
                        placeholder = {
                            Text("Enter command...", color = OnSurfaceVariant)
                        },
                        textStyle = LocalTextStyle.current.copy(
                            fontFamily = FontFamily.Monospace,
                            fontSize = fontSize.sp,
                            color = TerminalFg
                        ),
                        colors = OutlinedTextFieldDefaults.colors(
                            focusedBorderColor = AccentBlue,
                            unfocusedBorderColor = SurfaceVariant,
                            cursorColor = AccentBlue
                        ),
                        singleLine = true,
                        shape = RoundedCornerShape(8.dp)
                    )
                    Spacer(Modifier.width(8.dp))
                    FilledTonalIconButton(onClick = { /* send */ }) {
                        Icon(Icons.Default.Send, "Send")
                    }
                }
            }
        }
    }
}
