package social.localfirst.shell

import android.content.ActivityNotFoundException
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.HorizontalDivider
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.lfs_core.ChannelMembership
import uniffi.lfs_core.Core
import uniffi.lfs_core.CoreException
import uniffi.lfs_core.DocKind
import uniffi.lfs_core.Identity
import uniffi.lfs_core.Message
import uniffi.lfs_core.Ping
import uniffi.lfs_core.PingsDoc
import uniffi.lfs_core.Preferences
import uniffi.lfs_core.ProfileDoc
import uniffi.lfs_core.ThreadsDoc
import uniffi.lfs_core.TrustEntry
import uniffi.lfs_core.initCore
import java.time.Instant
import java.time.format.DateTimeParseException
import kotlin.concurrent.thread

/**
 * Phase 0 shell. B8 (Run 14): type -> Save -> kill -> relaunch -> text returns.
 * B4 (Run 15): resolvePds(did) over rustls + tokio via JNA, off the main thread.
 * B12 (Run 21): OAuth-path smoke. ACTION_VIEW to the Pages placeholder in the
 * system browser; the page's tap-to-continue link is lfs://oauth/callback?code=phase0,
 * routed back by the manifest intent-filter (singleTask -> onNewIntent).
 * No atproto OAuth, no PKCE, no token (Phase 2). Custom Tabs is Phase 2 wiring.
 * Persistence is the core's own SQLite file; the shell stores nothing.
 *
 * Run 33 (Phase 1 1a local UI): the three typed docs — profile, pings,
 * threads — are opened through openTypedDoc on the same Dispatchers.IO init
 * coroutine, read through the typed FFI records (getProfile / getPings /
 * getThreads), hoisted to MutableState and rendered as lists below the
 * Phase 0 controls. "Seed demo" writes sample docs (put* + save) so
 * kill -> relaunch shows doc content returning from SQLite. Ping
 * ephemerality is display-side only: expired pings stay in the doc, the UI
 * hides them (expiresAt vs now) — no expiry engine this run (recorded scope).
 */
class MainActivity : ComponentActivity() {
    private var core: Core? = null
    private var handle: ULong = 0u
    private var profileHandle: ULong = 0u
    private var pingsHandle: ULong = 0u
    private var threadsHandle: ULong = 0u
    private val oauthStatus: MutableState<String> = mutableStateOf("oauth-smoke: not run")
    private val docStatus: MutableState<String> = mutableStateOf("initializing core...")
    private val initialText: MutableState<String> = mutableStateOf("")
    // Run 33 — typed doc state (written on Main only).
    private val profile: MutableState<ProfileDoc?> = mutableStateOf(null)
    private val pings: MutableState<PingsDoc?> = mutableStateOf(null)
    private val threads: MutableState<ThreadsDoc?> = mutableStateOf(null)
    private val docsStatus: MutableState<String> = mutableStateOf("docs: not loaded")
    private var oauthT0 = 0L

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Run 31 (A-O22): initCore is the sole FFI init path and is blocking →
        // Dispatchers.IO, never main; publish on Main (§7 shape).
        // Run 33: typed docs opened on the same IO coroutine, after initCore.
        lifecycleScope.launch(Dispatchers.IO) {
            val dbPath = filesDir.resolve("lfs.sqlite").absolutePath
            try {
                val c = initCore(dbPath)
                val h = c.openDoc("note")
                val t = c.get(h, "text") ?: ""
                val ph = c.openTypedDoc("profile", DocKind.PROFILE)
                val gh = c.openTypedDoc("pings", DocKind.PINGS)
                val th = c.openTypedDoc("threads", DocKind.THREADS)
                val p = c.getProfile(ph)
                val g = c.getPings(gh)
                val tr = c.getThreads(th)
                withContext(Dispatchers.Main) {
                    core = c
                    handle = h
                    profileHandle = ph
                    pingsHandle = gh
                    threadsHandle = th
                    initialText.value = t
                    docStatus.value =
                        "openDoc(\"note\") ok; text=${t.length} chars; pins=${c.pins()}"
                    profile.value = p
                    pings.value = g
                    threads.value = tr
                    docsStatus.value = docsSummary(p, g, tr)
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
                    // Run 33
                    profile = profile.value,
                    pings = pings.value,
                    threads = threads.value,
                    docsStatus = docsStatus.value,
                    onSeed = { seedDemo() },
                    onReloadDocs = { reloadDocs() },
                )
            }
        }
        handleRedirect(intent)
    }

    // ---- Run 33 typed docs -------------------------------------------------

    /** Re-read the three typed docs through the FFI surface (open handles; local hydrate, no I/O). */
    private fun reloadDocs() {
        val c = core ?: run { docsStatus.value = "docs: core not open"; return }
        try {
            val p = c.getProfile(profileHandle)
            val g = c.getPings(pingsHandle)
            val t = c.getThreads(threadsHandle)
            profile.value = p; pings.value = g; threads.value = t
            docsStatus.value = docsSummary(p, g, t)
        } catch (e: CoreException) {
            docsStatus.value = "docs CoreException: ${e.message}"
        } catch (e: Throwable) {
            docsStatus.value = "docs ${e::class.simpleName}: ${e.message}"
        }
    }

    /**
     * Write sample content through put* and persist with save (off main —
     * save is SQLite I/O), then re-read. Kill -> relaunch afterwards shows the
     * same content returning from SQLite through the typed surface.
     */
    private fun seedDemo() {
        val c = core ?: run { docsStatus.value = "docs: core not open"; return }
        docsStatus.value = "seeding..."
        val now = Instant.now()
        fun ts(offsetSec: Long): String = now.plusSeconds(offsetSec).toString()
        val profileDoc = ProfileDoc(
            identity = Identity(
                displayName = "Jedi", handle = "@jedi",
                handleRegisteredAt = ts(-86_400), avatarColor = "#3a7", createdAt = ts(-86_400),
            ),
            preferences = Preferences(defaultPingType = "here", notificationsEnabled = true, discoverable = false),
            trustGraph = mapOf(
                "contact-1" to TrustEntry(tier = "close", connectedAt = ts(-3_600), syncStatus = "synced"),
                "contact-2" to TrustEntry(tier = "contact", connectedAt = ts(-1_800), syncStatus = "pending"),
            ),
            pingHistory = listOf(
                Ping(pingId = "p1", pingType = "thinking-of-you", senderId = "self",
                    sentAt = ts(-600), expiresAt = ts(86_400 - 600), content = null),
            ),
            channelMemberships = listOf(
                ChannelMembership(channelId = "ch-local-first", joinedAt = ts(-7_200), lastPingAt = ts(-600)),
            ),
        )
        val pingsDoc = PingsDoc(
            channels = mapOf(
                "ch-local-first" to listOf(
                    Ping(pingId = "p2", pingType = "check-this", senderId = "contact-1",
                        sentAt = ts(-300), expiresAt = ts(7 * 86_400), content = "subduction thread"),
                    Ping(pingId = "p3", pingType = "here", senderId = "contact-2",
                        sentAt = ts(-120), expiresAt = ts(86_400), content = null),
                    // Already expired at seed time — must NOT render (display-side filtering).
                    Ping(pingId = "p0", pingType = "status", senderId = "contact-1",
                        sentAt = ts(-172_800), expiresAt = ts(-86_400), content = "expired"),
                ),
            ),
        )
        val threadsDoc = ThreadsDoc(
            threads = mapOf(
                "contact-1" to listOf(
                    Message(messageId = "m1", senderId = "contact-1", sentAt = ts(-240),
                        content = "elevating to a thread", assetRef = "asset-9", readAt = ts(-200)),
                    Message(messageId = "m2", senderId = "self", sentAt = ts(-180),
                        content = "yes — let's", assetRef = null, readAt = null),
                ),
            ),
        )
        lifecycleScope.launch(Dispatchers.IO) {
            val out = try {
                c.putProfile(profileHandle, profileDoc)
                c.putPings(pingsHandle, pingsDoc)
                c.putThreads(threadsHandle, threadsDoc)
                val a = c.save(profileHandle).size
                val b = c.save(pingsHandle).size
                val d = c.save(threadsHandle).size
                "seeded ($a+$b+$d bytes); kill the app and relaunch"
            } catch (e: CoreException) {
                "seed CoreException: ${e.message}"
            } catch (e: Throwable) {
                "seed ${e::class.simpleName}: ${e.message}"
            }
            withContext(Dispatchers.Main) {
                reloadDocs()
                docsStatus.value = docsStatus.value + " · " + out
            }
        }
    }

    private fun docsSummary(p: ProfileDoc, g: PingsDoc, t: ThreadsDoc): String {
        val pingCount = g.channels.values.sumOf { it.size }
        val msgCount = t.threads.values.sumOf { it.size }
        val handle = if (p.identity.handle.isEmpty()) "(empty)" else p.identity.handle
        return "docs ok: profile=$handle trust=${p.trustGraph.size} " +
            "pings=$pingCount/${g.channels.size}ch threads=${t.threads.size}/${msgCount}msg"
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

// ---- Run 33 display helpers (spec display semantics, display-side only) ------

object PingDisplay {
    /**
     * Spec: "Pings expire at expiresAt." Display-side rule this run: hide a
     * ping whose expiresAt parses to a time at or before now; an unparseable
     * timestamp is shown (never silently dropped).
     */
    fun isActive(p: Ping, now: Instant = Instant.now()): Boolean = try {
        Instant.parse(p.expiresAt).isAfter(now)
    } catch (_: DateTimeParseException) {
        true
    }

    fun ttl(p: Ping, now: Instant = Instant.now()): String = try {
        val s = Instant.parse(p.expiresAt).epochSecond - now.epochSecond
        when {
            s >= 86_400 -> "${s / 86_400}d left"
            s >= 3_600 -> "${s / 3_600}h left"
            else -> "${maxOf(s / 60, 0)}m left"
        }
    } catch (_: DateTimeParseException) {
        "ttl ?"
    }
}

@Composable
fun Shell(
    initial: String,
    initialStatus: String,
    oauthStatus: String,
    onSave: (String) -> String,
    onResolve: (String, (String) -> Unit) -> Unit,
    onOAuth: () -> Unit,
    // Run 33
    profile: ProfileDoc?,
    pings: PingsDoc?,
    threads: ThreadsDoc?,
    docsStatus: String,
    onSeed: () -> Unit,
    onReloadDocs: () -> Unit,
) {
    var text by remember(initial) { mutableStateOf(initial) }
    var status by remember(initialStatus) { mutableStateOf(initialStatus) }
    var net by remember { mutableStateOf("resolvePds: not run") }
    Column(
        modifier = Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(24.dp),
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

        // Run 33 — Phase 1 1a local UI
        HorizontalDivider()
        Text("Phase 1 docs (Run 33)", style = MaterialTheme.typography.titleMedium)
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(onClick = onSeed) { Text("Seed demo") }
            Button(onClick = onReloadDocs) { Text("Reload docs") }
        }
        Text(docsStatus, style = MaterialTheme.typography.bodySmall)
        profile?.let { ProfileSection(it) }
        pings?.let { PingsSection(it) }
        threads?.let { ThreadsSection(it) }
    }
}

/**
 * Spec ProfileView / ContactList semantics: identity, preferences, trust graph
 * with tier indicators (contacts sorted by id — Automerge maps are unordered),
 * channel memberships, recent outbound ping history.
 */
@Composable
fun ProfileSection(p: ProfileDoc) {
    val small = MaterialTheme.typography.bodySmall
    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text("Profile", style = MaterialTheme.typography.titleSmall)
        if (p.identity.handle.isEmpty()) {
            Text("(empty profile — tap Seed demo)", style = small)
        } else {
            Text("${p.identity.displayName} ${p.identity.handle}")
            Text("avatar ${p.identity.avatarColor} · created ${p.identity.createdAt}", style = small)
            Text(
                "prefs: default=${p.preferences.defaultPingType} " +
                    "notify=${if (p.preferences.notificationsEnabled) "on" else "off"} " +
                    "discoverable=${if (p.preferences.discoverable) "yes" else "no"}",
                style = small,
            )
        }
        Text("Trust graph (${p.trustGraph.size})", style = small, fontWeight = FontWeight.Bold)
        p.trustGraph.keys.sorted().forEach { id ->
            val e = p.trustGraph.getValue(id)
            Text("• $id — ${e.tier} · ${e.syncStatus} · since ${e.connectedAt}", style = small)
        }
        Text("Channels (${p.channelMemberships.size})", style = small, fontWeight = FontWeight.Bold)
        p.channelMemberships.forEach { m ->
            Text("• ${m.channelId} · joined ${m.joinedAt} · last ping ${m.lastPingAt}", style = small)
        }
        Text("Ping history (${p.pingHistory.size})", style = small, fontWeight = FontWeight.Bold)
        p.pingHistory.forEach { ping ->
            val to = if (ping.senderId == "self") "out" else ping.senderId
            Text("• ${ping.pingType} → $to · ${ping.sentAt}", style = small)
        }
    }
}

/**
 * Spec PingFeed / PingBubble semantics: active channel pings with a TTL
 * indicator; expired entries hidden at display time (recorded scope — no
 * expiry engine in the core this run). Channels sorted; pings by sentAt.
 */
@Composable
fun PingsSection(g: PingsDoc) {
    val small = MaterialTheme.typography.bodySmall
    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text("Pings", style = MaterialTheme.typography.titleSmall)
        if (g.channels.isEmpty()) Text("(no channels)", style = small)
        g.channels.keys.sorted().forEach { ch ->
            val all = g.channels.getValue(ch)
            val active = all.filter { PingDisplay.isActive(it) }.sortedBy { it.sentAt }
            Text("#$ch — ${active.count()} active / ${all.size} stored", style = small, fontWeight = FontWeight.Bold)
            active.forEach { ping ->
                val content = ping.content?.let { " “$it”" } ?: ""
                Text("${ping.pingType} from ${ping.senderId}$content · ${PingDisplay.ttl(ping)}", style = small)
            }
        }
    }
}

/**
 * Spec ThreadList / ThreadDetail semantics: threads by contact (sorted by last
 * activity), messages in sentAt order, asset ref and read marker shown.
 */
@Composable
fun ThreadsSection(t: ThreadsDoc) {
    val small = MaterialTheme.typography.bodySmall
    val ordered = t.threads.keys.sortedByDescending { c -> t.threads.getValue(c).maxOfOrNull { it.sentAt } ?: "" }
    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text("Threads", style = MaterialTheme.typography.titleSmall)
        if (t.threads.isEmpty()) Text("(no threads)", style = small)
        ordered.forEach { contact ->
            val msgs = t.threads.getValue(contact).sortedBy { it.sentAt }
            Text("$contact — ${msgs.size} messages", style = small, fontWeight = FontWeight.Bold)
            msgs.forEach { m ->
                val who = if (m.senderId == "self") "you" else m.senderId
                val asset = m.assetRef?.let { " · asset $it" } ?: ""
                val read = m.readAt?.let { " · read $it" } ?: " · unread"
                Text("$who: ${m.content}", style = small)
                Text("${m.sentAt}$asset$read", style = MaterialTheme.typography.labelSmall)
            }
        }
    }
}
