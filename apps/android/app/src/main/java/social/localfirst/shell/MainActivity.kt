package social.localfirst.shell

import android.content.ActivityNotFoundException
import android.content.Intent
import android.net.Uri
import android.content.ClipData
import android.content.ClipDescription
import android.content.ClipboardManager
import android.content.Context
import android.os.Build
import android.os.Bundle
import android.os.PersistableBundle
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
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
import uniffi.lfs_core.ColdKeyExport
import uniffi.lfs_core.DocKind
import uniffi.lfs_core.GrantLevel
import uniffi.lfs_core.GroupMember
import uniffi.lfs_core.Identity
import uniffi.lfs_core.IdentityConfig
import uniffi.lfs_core.Message
import uniffi.lfs_core.Ping
import uniffi.lfs_core.PingsDoc
import uniffi.lfs_core.Preferences
import uniffi.lfs_core.ProfileDoc
import uniffi.lfs_core.RootingLevel
import uniffi.lfs_core.ThreadsDoc
import uniffi.lfs_core.TrustEntry
import uniffi.lfs_core.initCore
import java.security.KeyStore
import java.time.Instant
import java.time.format.DateTimeParseException
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec
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
 *
 * Run 39 (plan §9 row 5, grant/revoke on device): the FIRST shell surface of
 * the identity ceremony and of the membership FFI (F-38-1). Custody per spec
 * §5.2 as ruled — the PROPERTY, not a library: the device seed
 * (DeviceKeyExport.deviceSecret) is encrypted at rest under an AndroidKeyStore
 * AES-GCM key (setUserAuthenticationRequired not required in Phase 1) and the
 * ciphertext lives in app-owned SharedPreferences; ColdKeyExport is shown ONCE
 * for off-device custody (copy), never stored. On launch, an enrolled identity
 * with a custodied seed is rebuilt through reloadIdentity, so grant state
 * survives kill -> relaunch. Rooting level is a config value (SharedPreferences
 * key "rootingLevel", default "edit"), not a constant (H5). Blocking FFI calls
 * run on Dispatchers.IO.
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
    // Run 39 — identity + membership state
    private val identityStatus: MutableState<String> = mutableStateOf("identity: unknown")
    private val coldKeysOnce: MutableState<ColdKeyExport?> = mutableStateOf(null)
    private val membershipVersion: MutableState<ULong> = mutableStateOf(0u)
    private val deviceLevel: MutableState<String> = mutableStateOf("-")
    private val members: MutableState<List<GroupMember>> = mutableStateOf(emptyList())
    private val membershipStatus: MutableState<String> = mutableStateOf("")
    // Run 40 — recovery from the cold key (plan §9 row 6; D-40 record shell input):
    // the seed is pasted as 64 hex and DECODED HERE to bytes — no hex-typed seed
    // crosses FFI; the core re-checks 32 bytes. The admin seed is never stored.
    private val recoveryStatus: MutableState<String> = mutableStateOf("")
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
                // Run 37 — pings adopt the Run 35 cleanup-on-load path: the shell supplies the clock (RFC 3339, Instant.toString()).
                val gh = c.openTypedDocAt("pings", DocKind.PINGS, Instant.now().toString())
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
                reloadIdentityIfCustodied(c)
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
                    // Run 39
                    identityStatus = identityStatus.value,
                    coldKeysOnce = coldKeysOnce.value,
                    onDismissColdKeys = { coldKeysOnce.value = null },
                    onCopy = { label, hex -> copyToClipboard(label, hex) },
                    onCeremony = { runCeremony() },
                    onReloadIdentity = { core?.let { c -> lifecycleScope.launch(Dispatchers.IO) { reloadIdentityIfCustodied(c) } } },
                    membershipVersion = membershipVersion.value,
                    deviceLevel = deviceLevel.value,
                    members = members.value,
                    membershipStatus = membershipStatus.value,
                    onGrant = { level -> grant(level) },
                    onRevoke = { fp -> revoke(fp) },
                    // Run 40
                    recoveryStatus = recoveryStatus.value,
                    onRecover = { hex -> recoverIdentity(hex) },
                    onForgetDeviceSeed = { forgetDeviceSeed() },
                )
            }
        }
        handleRedirect(intent)
    }

    // ---- Run 39 identity ceremony, custody, membership -----------------------

    /** The rooting level as CONFIGURATION (H5): a stored value with a default, never a constant in code. */
    private fun configuredRootingLevel(): RootingLevel {
        val v = getSharedPreferences(PREFS, Context.MODE_PRIVATE).getString("rootingLevel", "edit") ?: "edit"
        return if (v.equals("admin", ignoreCase = true)) RootingLevel.ADMIN else RootingLevel.EDIT
    }

    /** IO thread. If an identity row exists and the seed is custodied, rebuild the device hive (identity, then groups). */
    private suspend fun reloadIdentityIfCustodied(c: Core) {
        withContext(Dispatchers.Main) { identityStatus.value = "identity: checking..." }
        if (!c.identityEnrolled()) {
            withContext(Dispatchers.Main) { identityStatus.value = "identity: not enrolled (run the ceremony)" }
            return
        }
        val seed = SeedCustody.read(this)
        if (seed == null) {
            withContext(Dispatchers.Main) { identityStatus.value = "identity: enrolled, NO device seed custodied (recovery = Run 40)" }
            return
        }
        val msg = try {
            c.reloadIdentity(seed)
            "identity: reloaded from Keystore-wrapped seed (${seed.size} bytes)"
        } catch (e: Throwable) {
            "identity: reload error: ${e::class.simpleName}: ${e.message}"
        }
        val snap = snapshot(c)
        withContext(Dispatchers.Main) { identityStatus.value = msg; apply(snap) }
    }

    /** The ceremony, once. Device seed -> Keystore-wrapped storage; cold keys -> shown once. */
    private fun runCeremony() {
        val c = core ?: run { identityStatus.value = "core not open"; return }
        val level = configuredRootingLevel()
        identityStatus.value = "identity: running ceremony (rooting $level)..."
        lifecycleScope.launch(Dispatchers.IO) {
            try {
                val rec = c.runIdentityCeremony(IdentityConfig(rootingLevel = level))
                val stored = SeedCustody.store(this@MainActivity, rec.deviceKey.deviceSecret)
                val snap = snapshot(c)
                withContext(Dispatchers.Main) {
                    identityStatus.value = "identity: enrolled — ${rec.adminDelegations} admin delegations, floor ${rec.floorStatus}, device seed ${if (stored) "custodied" else "CUSTODY WRITE FAILED"}"
                    coldKeysOnce.value = rec.coldKeys
                    apply(snap)
                }
            } catch (e: Throwable) {
                withContext(Dispatchers.Main) { identityStatus.value = "identity: ceremony error: ${e::class.simpleName}: ${e.message}" }
            }
        }
    }

    private fun grant(level: GrantLevel) {
        val c = core ?: return
        lifecycleScope.launch(Dispatchers.IO) {
            val msg = try {
                "grant $level ok -> version ${c.grantMember(DEMO_GROUP, level)}"
            } catch (e: Throwable) {
                "grant refused: ${e::class.simpleName}: ${e.message}"
            }
            val snap = snapshot(c)
            withContext(Dispatchers.Main) { membershipStatus.value = msg; apply(snap) }
        }
    }

    private fun revoke(fingerprint: String) {
        val c = core ?: return
        lifecycleScope.launch(Dispatchers.IO) {
            val msg = try {
                "revoke ok -> version ${c.revokeMember(DEMO_GROUP, fingerprint)}"
            } catch (e: Throwable) {
                "revoke refused: ${e::class.simpleName}: ${e.message}"
            }
            val snap = snapshot(c)
            withContext(Dispatchers.Main) { membershipStatus.value = msg; apply(snap) }
        }
    }

    // ---- Run 40 recovery from the cold key ---------------------------------

    /**
     * Run 40 — recovery from a cold admin seed (primary or recovery; the device
     * legs use the RECOVERY seed — D-40-2). The pasted 64-hex is decoded to 32
     * bytes here; recoverIdentity rebuilds the identity with the admin as active
     * agent, re-delegates a NEW device (Edit on the identity document, Admin on
     * every migrated group) and returns the new device seed once — custodied
     * exactly as the ceremony's. The admin seed lives in this call only. IO thread.
     */
    private fun recoverIdentity(hexText: String) {
        val c = core ?: run { recoveryStatus.value = "core not open"; return }
        val seed = decodeSeedHex(hexText) ?: run {
            recoveryStatus.value = "recovery: paste exactly 64 hex characters (32-byte seed)"
            return
        }
        recoveryStatus.value = "recovery: rebuilding identity from the cold seed..."
        lifecycleScope.launch(Dispatchers.IO) {
            try {
                val report = c.recoverIdentity(seed)
                val stored = SeedCustody.store(this@MainActivity, report.deviceKey.deviceSecret)
                val snap = snapshot(c)
                withContext(Dispatchers.Main) {
                    recoveryStatus.value = "recovery: OK — admin ${report.adminFingerprint.take(16)}…, identity members ${report.identityMembers}, groups migrated ${report.groupsMigrated}, unrecovered ${report.groupsUnrecovered.size}, new device ${report.deviceKey.deviceFingerprint.take(16)}… seed ${if (stored) "custodied" else "CUSTODY WRITE FAILED"}"
                    identityStatus.value = "identity: RECOVERED — new device seed custodied (${report.deviceKey.deviceSecret.size} bytes)"
                    apply(snap)
                }
            } catch (e: Throwable) {
                withContext(Dispatchers.Main) { recoveryStatus.value = "recovery: error: ${e::class.simpleName}: ${e.message}" }
            }
        }
    }

    /**
     * Run 40 — TEST-ONLY: drop the custodied device seed (the wrapped seed is
     * removed from SharedPreferences; the SQLite rows are untouched) to stage the
     * recovery state on a device without an uninstall. Not a product control.
     */
    private fun forgetDeviceSeed() {
        val gone = SeedCustody.forget(this)
        identityStatus.value = if (gone)
            "identity: device seed FORGOTTEN (test-only) — rows kept; relaunch shows the recovery state"
        else
            "identity: no device seed to forget"
        membershipStatus.value = ""
    }

    /** Run 40 — 64 hex -> 32 bytes, or null. Whitespace trimmed; case-insensitive. */
    private fun decodeSeedHex(text: String): ByteArray? {
        val hex = text.trim().lowercase()
        if (hex.length != 64 || !hex.all { it in '0'..'9' || it in 'a'..'f' }) return null
        val out = ByteArray(32) { i -> hex.substring(i * 2, i * 2 + 2).toInt(16).toByte() }
        return out
    }

    private data class Snapshot(val version: ULong, val deviceLevel: String, val members: List<GroupMember>)

    /** Everything the membership section renders, read from the core in one place (IO thread). */
    private fun snapshot(c: Core): Snapshot = Snapshot(
        version = c.membershipVersion(DEMO_GROUP),
        deviceLevel = c.deviceGrantLevel(DEMO_GROUP)?.toString() ?: "-",
        members = c.groupMembers(DEMO_GROUP),
    )

    private fun apply(s: Snapshot) {
        membershipVersion.value = s.version
        deviceLevel.value = s.deviceLevel
        members.value = s.members
    }

    private fun copyToClipboard(label: String, text: String) {
        val cm = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        // Run 40 (D-40 record shell input): the clipboard is the residual custody
        // surface for the cold seeds — mark the clip SENSITIVE (Android 13+ hides
        // it from the clipboard preview; older releases ignore the extra).
        val clip = ClipData.newPlainText(label, text)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            clip.description.extras = PersistableBundle().apply { putBoolean(ClipDescription.EXTRA_IS_SENSITIVE, true) }
        }
        cm.setPrimaryClip(clip)
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

/** Run 39 — the app-owned group the shell demonstrates grant/revoke on; also the SharedPreferences file name. */
const val DEMO_GROUP = "demo"
const val PREFS = "lfs-shell"

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
    // Run 39
    identityStatus: String,
    coldKeysOnce: ColdKeyExport?,
    onDismissColdKeys: () -> Unit,
    onCopy: (String, String) -> Unit,
    onCeremony: () -> Unit,
    onReloadIdentity: () -> Unit,
    membershipVersion: ULong,
    deviceLevel: String,
    members: List<GroupMember>,
    membershipStatus: String,
    onGrant: (GrantLevel) -> Unit,
    onRevoke: (String) -> Unit,
    // Run 40
    recoveryStatus: String,
    onRecover: (String) -> Unit,
    onForgetDeviceSeed: () -> Unit,
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

        // Run 39 — identity ceremony + membership (grant/revoke on device)
        HorizontalDivider()
        Text("Identity & groups (Run 39)", style = MaterialTheme.typography.titleMedium)
        Text(identityStatus, style = MaterialTheme.typography.bodySmall)
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(onClick = onCeremony) { Text("Run ceremony") }
            Button(onClick = onReloadIdentity) { Text("Reload identity") }
        }
        coldKeysOnce?.let { ColdKeysOnceSection(it, onCopy, onDismissColdKeys) }
        Text("group \"$DEMO_GROUP\": version $membershipVersion, device level $deviceLevel", style = MaterialTheme.typography.bodySmall)
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(onClick = { onGrant(GrantLevel.READ) }) { Text("Grant Read") }
            Button(onClick = { onGrant(GrantLevel.EDIT) }) { Text("Grant Edit") }
        }
        Text(membershipStatus, style = MaterialTheme.typography.bodySmall)
        MembersSection(members, onRevoke)

        // Run 40 — recovery from the cold key
        HorizontalDivider()
        Text("Recovery (Run 40)", style = MaterialTheme.typography.titleMedium)
        Text("Paste a cold admin seed (64 hex) — decoded on this device, never stored", style = MaterialTheme.typography.bodySmall)
        var seedHex by remember { mutableStateOf("") }
        OutlinedTextField(
            value = seedHex,
            onValueChange = { seedHex = it },
            label = { Text("cold admin seed, 64 hex") },
            singleLine = true,
        )
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(onClick = { onRecover(seedHex); seedHex = "" }) { Text("Recover identity") }
            Button(onClick = onForgetDeviceSeed) { Text("Forget device seed (test-only)") }
        }
        Text(recoveryStatus, style = MaterialTheme.typography.bodySmall)
    }
}

/**
 * Run 39 — the cold admin material, shown ONCE at ceremony time for off-device
 * custody (copy, then dismiss). Not persisted by the shell; dismissing drops the
 * only copy the app holds.
 */
@Composable
fun ColdKeysOnceSection(keys: ColdKeyExport, onCopy: (String, String) -> Unit, onDismiss: () -> Unit) {
    val small = MaterialTheme.typography.bodySmall
    fun hex(b: ByteArray) = b.joinToString("") { "%02x".format(it) }
    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text("Cold admin keys — shown once, NOT stored on this device", style = MaterialTheme.typography.titleSmall)
        Text("primary ${keys.primaryAdminFingerprint.take(16)}… / recovery ${keys.recoveryFingerprint.take(16)}…", style = small)
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(onClick = { onCopy("primary seed", hex(keys.primaryAdminSecret)) }) { Text("Copy primary") }
            Button(onClick = { onCopy("recovery seed", hex(keys.recoverySecret)) }) { Text("Copy recovery") }
            Button(onClick = onDismiss) { Text("Dismiss") }
        }
    }
}

/**
 * Run 39 — per-member grant state from groupMembers, one row per member
 * (fingerprint prefix, level, "device" marker), with a Revoke control on every
 * peer. The list is exactly what the core returns — no shell-side state.
 */
@Composable
fun MembersSection(members: List<GroupMember>, onRevoke: (String) -> Unit) {
    val small = MaterialTheme.typography.bodySmall
    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text("Members (${members.size})", style = MaterialTheme.typography.titleSmall)
        if (members.isEmpty()) Text("no group yet", style = small)
        members.forEach { m ->
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("${m.fingerprint.take(12)}… ${m.level}${if (m.isDevice) " (device)" else ""}", style = small)
                if (!m.isDevice) Button(onClick = { onRevoke(m.fingerprint) }) { Text("Revoke") }
            }
        }
    }
}

/**
 * Run 39 — custody of the DEVICE seed only (spec §5.2 as ruled, by property):
 * an AES-256-GCM key generated in AndroidKeyStore (never exportable) wraps the
 * 32-byte seed; iv + ciphertext live base64 in app-owned SharedPreferences.
 * setUserAuthenticationRequired is not required in Phase 1. The cold admin seeds
 * are NOT stored here or anywhere on the device.
 */
object SeedCustody {
    private const val ALIAS = "lfs-device-seed-wrap"
    private const val KEY = "deviceSeed"

    private fun wrapKey(): SecretKey {
        val ks = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        (ks.getKey(ALIAS, null) as? SecretKey)?.let { return it }
        val kg = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore")
        kg.init(
            KeyGenParameterSpec.Builder(ALIAS, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setKeySize(256)
                .build(),
        )
        return kg.generateKey()
    }

    fun store(ctx: Context, seed: ByteArray): Boolean = try {
        val c = Cipher.getInstance("AES/GCM/NoPadding").apply { init(Cipher.ENCRYPT_MODE, wrapKey()) }
        val ct = c.doFinal(seed)
        val packed = Base64.encodeToString(c.iv, Base64.NO_WRAP) + ":" + Base64.encodeToString(ct, Base64.NO_WRAP)
        ctx.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit().putString(KEY, packed).apply()
        true
    } catch (e: Throwable) {
        false
    }

    /** Run 40 — TEST-ONLY: remove the wrapped seed (the KeyStore wrap key is kept; it wraps nothing). */
    fun forget(ctx: Context): Boolean {
        val prefs = ctx.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        val had = prefs.contains(KEY)
        prefs.edit().remove(KEY).apply()
        return had
    }

    fun read(ctx: Context): ByteArray? = try {
        val packed = ctx.getSharedPreferences(PREFS, Context.MODE_PRIVATE).getString(KEY, null) ?: return null
        val (iv, ct) = packed.split(":", limit = 2).map { Base64.decode(it, Base64.NO_WRAP) }
        Cipher.getInstance("AES/GCM/NoPadding")
            .apply { init(Cipher.DECRYPT_MODE, wrapKey(), GCMParameterSpec(128, iv)) }
            .doFinal(ct)
    } catch (e: Throwable) {
        null
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
