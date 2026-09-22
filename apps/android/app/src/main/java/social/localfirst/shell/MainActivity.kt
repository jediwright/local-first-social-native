package social.localfirst.shell

import android.content.ActivityNotFoundException
import android.content.Intent
import android.net.Uri
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
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.lfs_core.Core
import uniffi.lfs_core.CoreException
import uniffi.lfs_core.initCore
import kotlin.concurrent.thread

/**
 * Phase 0 shell. B8 (Run 14): type -> Save -> kill -> relaunch -> text returns.
 * B4 (Run 15): resolvePds(did) over rustls + tokio via JNA, off the main thread.
 * B12 (Run 21): OAuth-path smoke. ACTION_VIEW to the Pages placeholder in the
 * system browser; the page's tap-to-continue link is lfs://oauth/callback?code=phase0,
 * routed back by the manifest intent-filter (singleTask -> onNewIntent).
 * No atproto OAuth, no PKCE, no token (Phase 2). Custom Tabs is Phase 2 wiring.
 * Persistence is the core's own SQLite file; the shell stores nothing.
 */
class MainActivity : ComponentActivity() {
    private var core: Core? = null
    private var handle: ULong = 0u
    private val oauthStatus: MutableState<String> = mutableStateOf("oauth-smoke: not run")
    private val docStatus: MutableState<String> = mutableStateOf("initializing core...")
    private val initialText: MutableState<String> = mutableStateOf("")
    private var oauthT0 = 0L

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Run 31 (A-O22): initCore is the sole FFI init path and is blocking →
        // Dispatchers.IO, never main; publish on Main (§7 shape).
        lifecycleScope.launch(Dispatchers.IO) {
            val dbPath = filesDir.resolve("lfs.sqlite").absolutePath
            try {
                val c = initCore(dbPath)
                val h = c.openDoc("note")
                val t = c.get(h, "text") ?: ""
                withContext(Dispatchers.Main) {
                    core = c
                    handle = h
                    initialText.value = t
                    docStatus.value =
                        "openDoc(\"note\") ok; text=${t.length} chars; pins=${c.pins()}"
                }
            } catch (e: CoreException) {
                // A-O22 check: typed catch; subclass on demand
                // (CoreException.Storage / .Network / .Automerge / .NoSuchHandle).
                withContext(Dispatchers.Main) { docStatus.value = "CoreException: ${e.message}" }
            } catch (e: Throwable) {
                withContext(Dispatchers.Main) {
                    docStatus.value = "${e::class.simpleName}: ${e.message}"
                }
            }
        }
        setContent {
            MaterialTheme {
                Shell(
                    initial = initialText.value,
                    initialStatus = docStatus.value,
                    oauthStatus = oauthStatus.value,
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
                    onOAuth = { startOAuthSmoke() },
                )
            }
        }
        handleRedirect(intent)
    }

    private fun startOAuthSmoke() {
        oauthT0 = System.currentTimeMillis()
        oauthStatus.value = "opening system browser..."
        try {
            startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(OAUTH_PLACEHOLDER)))
        } catch (e: ActivityNotFoundException) {
            oauthStatus.value = "oauth-smoke: no browser on this image (${e.message})"
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        handleRedirect(intent)
    }

    private fun handleRedirect(intent: Intent?) {
        val uri = intent?.data ?: return
        if (uri.scheme != "lfs") return
        val ms = if (oauthT0 == 0L) -1L else System.currentTimeMillis() - oauthT0
        oauthStatus.value = "oauth-smoke ok in $ms ms: $uri code=${uri.getQueryParameter("code")}"
    }
}

const val PROBE_DID = "did:plc:z72i7hdynmk6r22z27h6tvur"
const val OAUTH_PLACEHOLDER = "https://jediwright.github.io/local-first-social-native/phase0/oauth.html"

@Composable
fun Shell(
    initial: String,
    initialStatus: String,
    oauthStatus: String,
    onSave: (String) -> String,
    onResolve: (String, (String) -> Unit) -> Unit,
    onOAuth: () -> Unit,
) {
    var text by remember(initial) { mutableStateOf(initial) }
    var status by remember(initialStatus) { mutableStateOf(initialStatus) }
    var net by remember { mutableStateOf("resolvePds: not run") }
    Column(
        modifier = Modifier.fillMaxSize().padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Text("lfs_shell — Phase 0 B8/B4/B12", style = MaterialTheme.typography.titleLarge)
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
        Button(onClick = onOAuth) { Text("OAuth") }
        Text(oauthStatus, style = MaterialTheme.typography.bodySmall)
    }
}
