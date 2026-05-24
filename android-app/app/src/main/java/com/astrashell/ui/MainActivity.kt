package com.astrashell.ui

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.astrashell.ui.screens.*
import com.astrashell.ui.theme.AstraShellTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            AstraShellTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    AstraShellNav()
                }
            }
        }
    }
}

@Composable
fun AstraShellNav() {
    val navController = rememberNavController()

    NavHost(navController = navController, startDestination = "home") {
        composable("home") {
            HomeScreen(
                onNavigateToTerminal = { navController.navigate("terminal") },
                onNavigateToSettings = { navController.navigate("settings") },
                onNavigateToContainers = { navController.navigate("containers") },
                onNavigateToImages = { navController.navigate("images") }
            )
        }
        composable("terminal") {
            TerminalScreen(
                onBack = { navController.popBackStack() }
            )
        }
        composable("settings") {
            SettingsScreen(
                onBack = { navController.popBackStack() }
            )
        }
        composable("containers") {
            ContainerScreen(
                onBack = { navController.popBackStack() }
            )
        }
        composable("images") {
            ImageScreen(
                onBack = { navController.popBackStack() },
                onUseImage = { /* switch to selected image */ }
            )
        }
    }
}
