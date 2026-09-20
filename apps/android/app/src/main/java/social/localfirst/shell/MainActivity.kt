package social.localfirst.shell

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import uniffi.lfs_core.Core
import uniffi.lfs_core.CoreException

/**
 * Phase 0 B8 shell: type -> Save -> kill app -> relaunch -> text returns.
 * Persistence is the core's own SQLite file; the shell stores nothing.
 */
class MainActivity : ComponentActivity() {
    private var core: Core? = null
    private var handle: ULong = 0u

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val dbPath = filesDir.resolve("lfs.sqlite").absolutePath
        var initial = ""
        val status: String = try {
            val c = Core(dbPath)
            core = c
            handle = c.openDoc("note")
            initial = c.get(handle, "text") ?: ""
            "openDoc(\"note\") ok; text=${initial.length} chars; pins=${c.pins()}"
        } catch (e: CoreException) {
            "CoreException: ${e.message}"
        } catch (e: Throwable) {
            "${e::class.simpleName}: ${e.message}"
        }
        setContent {
            MaterialTheme {
                Shell(initial, status) { text ->
                    try {
                        val c = core ?: return@Shell "core not open"
                        c.put(handle, "text", text)
                        val bytes = c.save(handle)
                        "saved: ${bytes.size} bytes; kill the app and relaunch"
                    } catch (e: Throwable) {
                        "${e::class.simpleName}: ${e.message}"
                    }
                }
            }
        }
    }
}

@Composable
fun Shell(initial: String, initialStatus: String, onSave: (String) -> String) {
    var text by remember { mutableStateOf(initial) }
    var status by remember { mutableStateOf(initialStatus) }
    Column(
        modifier = Modifier.fillMaxSize().padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Text("lfs_shell — Phase 0 B8", style = MaterialTheme.typography.titleLarge)
        OutlinedTextField(
            value = text,
            onValueChange = { text = it },
            label = { Text("note.text") },
            modifier = Modifier.fillMaxWidth(),
        )
        Button(onClick = { status = onSave(text) }) { Text("Save") }
        Text(status, style = MaterialTheme.typography.bodySmall)
    }
}
