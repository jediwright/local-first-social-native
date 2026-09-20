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
import kotlin.concurrent.thread

/**
 * Phase 0 shell. B8 (Run 14): type -> Save -> kill -> relaunch -> text returns.
 * B4 (Run 15): resolvePds(did) over rustls + tokio via JNA, off the main thread.
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
                Shell(
                    initial = initial,
                    initialStatus = status,
                    onSave = { text ->
                        try {
                            val c = core ?: return@Shell "core not open"
                            c.put(handle, "text", text)
                            val bytes = c.save(handle)
                            "saved: ${bytes.size} bytes; kill the app and relaunch"
                        } catch (e: Throwable) {
                            "${e::class.simpleName}: ${e.message}"
                        }
                    },
                    onResolve = { did, report ->
                        val c = core
                        if (c == null) { report("core not open"); return@Shell }
                        report("resolving $did ...")
                        val t0 = System.currentTimeMillis()
                        thread(name = "resolvePds") {
                            val out = try {
                                val pds = c.resolvePds(did)
                                "resolvePds ok in ${System.currentTimeMillis() - t0} ms: $pds"
                            } catch (e: CoreException) {
                                "CoreException after ${System.currentTimeMillis() - t0} ms: ${e.message}"
                            } catch (e: Throwable) {
                                "${e::class.simpleName} after ${System.currentTimeMillis() - t0} ms: ${e.message}"
                            }
                            runOnUiThread { report(out) }
                        }
                    },
                )
            }
        }
    }
}

const val PROBE_DID = "did:plc:z72i7hdynmk6r22z27h6tvur"

@Composable
fun Shell(
    initial: String,
    initialStatus: String,
    onSave: (String) -> String,
    onResolve: (String, (String) -> Unit) -> Unit,
) {
    var text by remember { mutableStateOf(initial) }
    var status by remember { mutableStateOf(initialStatus) }
    var net by remember { mutableStateOf("resolvePds: not run") }
    Column(
        modifier = Modifier.fillMaxSize().padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Text("lfs_shell — Phase 0 B8/B4", style = MaterialTheme.typography.titleLarge)
        OutlinedTextField(
            value = text,
            onValueChange = { text = it },
            label = { Text("note.text") },
            modifier = Modifier.fillMaxWidth(),
        )
        Button(onClick = { status = onSave(text) }) { Text("Save") }
        Text(status, style = MaterialTheme.typography.bodySmall)
        Button(onClick = { onResolve(PROBE_DID) { net = it } }) { Text("resolvePds") }
        Text(net, style = MaterialTheme.typography.bodySmall)
    }
}
