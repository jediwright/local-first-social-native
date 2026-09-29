package social.localfirst.shell

import android.content.Context
import androidx.activity.ComponentActivity
import androidx.compose.runtime.Composable
import uniffi.lfs_core.Core

/**
 * Frontier F-1 — release stub (charter rule 7). The F-1 screen, its controls
 * and its custody live in the debug source set only; a release build gets
 * these no-ops, so nothing that shows or deletes F-1 secrets compiles in.
 */
object F1Debug {
    @Suppress("UNUSED_PARAMETER")
    suspend fun onLaunch(ctx: Context, c: Core) {}

    @Suppress("UNUSED_PARAMETER")
    @Composable
    fun Section(activity: ComponentActivity, core: () -> Core?) {}
}
