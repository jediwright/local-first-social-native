package social.localfirst.shell

import android.content.Context
import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyInfo
import android.security.keystore.KeyProperties
import android.util.Base64
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.Button
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.unit.dp
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.lfs_core.Core
import uniffi.lfs_core.F1Pending
import uniffi.lfs_core.F1Report
import uniffi.lfs_core.F1RestoreMode
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.SecretKeyFactory
import javax.crypto.spec.GCMParameterSpec

/**
 * Frontier F-1, tried step 7 — DEBUG SOURCE SET ONLY (charter rule 7). The
 * release source set has an empty stub, so this screen and its controls never
 * compile into a release build.
 *
 * The F-1 hive is separate from the identity ceremony (plan D1(a)). Its device
 * seed and exported prekey secrets are wrapped under their own AndroidKeyStore
 * AES-GCM key and kept in SharedPreferences, written with commit() so "saved"
 * means on disk. After every change the secrets are saved before the rows are
 * committed (plan D2(a)). On launch the stored document is restored with the
 * secrets imported first. Status goes to the screen and to logcat tag "F1":
 * prefixes, counts and read results only, never key bytes.
 */
object F1Debug {
    private const val TAG = "F1"
    private val status: MutableState<String> = mutableStateOf("F-1: not run")
    private val mode: MutableState<F1RestoreMode> = mutableStateOf(F1RestoreMode.IMPORT_FIRST)

    private enum class Change { SETUP, WRITE, ROTATE }

    /** IO thread, after initCore: restore the stored F-1 document, secrets first. */
    suspend fun onLaunch(ctx: Context, c: Core) {
        val line = if (!c.f1Present()) {
            "F-1 launch: no stored document (tap Set up)"
        } else {
            val seed = F1Custody.seed(ctx)
            if (seed == null) {
                "F-1 launch: rows present, no custodied F-1 seed"
            } else try {
                "F-1 launch: " + text(c.f1Restore(seed, F1Custody.secrets(ctx), F1RestoreMode.IMPORT_FIRST)) +
                    "\nwrap key: ${F1Custody.securityLevel()}"
            } catch (e: Throwable) {
                "F-1 launch: restore error: ${e::class.simpleName}: ${e.message}"
            }
        }
        publish(line)
    }

    @Composable
    fun Section(activity: ComponentActivity, core: () -> Core?) {
        val small = MaterialTheme.typography.bodySmall
        fun act(label: String, block: (Core) -> String) {
            val c = core() ?: run { status.value = "F-1: core not open"; return }
            status.value = "F-1: $label..."
            activity.lifecycleScope.launch(Dispatchers.IO) {
                val out = try { block(c) } catch (e: Throwable) { "F-1 $label error: ${e::class.simpleName}: ${e.message}" }
                publish(out)
            }
        }
        HorizontalDivider()
        Text("F-1 on-device secrets (frontier, debug build only)", style = MaterialTheme.typography.titleMedium)
        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Button(onClick = { act("set up") { change(activity, it, Change.SETUP) } }) { Text("Set up") }
                Button(onClick = { act("write") { change(activity, it, Change.WRITE) } }) { Text("Write") }
                Button(onClick = { act("rotate") { change(activity, it, Change.ROTATE) } }) { Text("Rotate") }
            }
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Button(onClick = { mode.value = next(mode.value) }) { Text("Mode: ${mode.value}") }
                Button(onClick = {
                    act("restore") { c ->
                        val seed = F1Custody.seed(activity) ?: return@act "F-1 restore: no custodied F-1 seed"
                        text(c.f1Restore(seed, F1Custody.secrets(activity), mode.value))
                    }
                }) { Text("Restore") }
            }
            Text("Failure staging (kill the app right after each):", style = small)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Button(onClick = { act("7e") { change(activity, it, Change.WRITE, save = true, commit = false) } }) {
                    Text("Write: save secrets, don't commit")
                }
            }
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Button(onClick = { act("7f write") { change(activity, it, Change.WRITE, save = false, commit = true) } }) {
                    Text("Write: commit, don't save")
                }
                Button(onClick = { act("7f rotate") { change(activity, it, Change.ROTATE, save = false, commit = true) } }) {
                    Text("Rotate: commit, don't save")
                }
            }
            Text(status.value, style = small)
        }
    }

    /** One change, in the plan's order: core call → save secrets (commit()) → commit rows. */
    private fun change(ctx: Context, c: Core, what: Change, save: Boolean = true, commit: Boolean = true): String {
        val p: F1Pending = when (what) {
            Change.SETUP -> c.f1Setup()
            Change.WRITE -> c.f1Write()
            Change.ROTATE -> c.f1Rotate()
        }
        val sb = StringBuilder(text(p.report))
        if (save) {
            if (!F1Custody.save(ctx, p.deviceSeed, p.prekeySecrets)) {
                return sb.append("\nSECRETS NOT SAVED — rows not committed").toString()
            }
            sb.append("\nsecrets saved (wrap key: ${F1Custody.securityLevel()})")
        } else {
            sb.append("\nsecrets NOT re-saved (staged failure)")
        }
        if (commit) {
            c.f1Commit()
            sb.append("; rows committed")
        } else {
            sb.append("; rows NOT committed (staged failure) — kill now")
        }
        return sb.toString()
    }

    private fun next(m: F1RestoreMode): F1RestoreMode {
        val all = F1RestoreMode.entries
        return all[(all.indexOf(m) + 1) % all.size]
    }

    private fun text(r: F1Report): String {
        val sb = StringBuilder(
            "${r.phase} · doc ${r.docPrefix} · key update ${r.keyUpdate ?: "-"} · " +
                "pending ${r.pending.size} · cgka ${r.cgkaOps ?: "-"} · secret pairs ${r.secretPairs}",
        )
        r.pending.forEach { sb.append("\n  pending ").append(it) }
        r.reads.forEach { sb.append("\n  read ${it.label}: ${if (it.ok) "OK" else "FAILED"} (${it.detail})") }
        return sb.toString()
    }

    private suspend fun publish(line: String) {
        line.lines().forEach { Log.i(TAG, it) }
        withContext(Dispatchers.Main) { status.value = line }
    }
}

/**
 * F-1 custody: the F-1 device seed and prekey secrets, each wrapped under an
 * AES-256-GCM key generated in AndroidKeyStore (alias lfs-f1-wrap, not
 * exportable), stored base64 as iv:ciphertext in the app's own preferences.
 * Separate from SeedCustody (the identity seed), which is unchanged.
 */
object F1Custody {
    private const val ALIAS = "lfs-f1-wrap"
    private const val SEED = "f1DeviceSeed"
    private const val SECRETS = "f1PrekeySecrets"

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

    private fun wrap(bytes: ByteArray): String {
        val c = Cipher.getInstance("AES/GCM/NoPadding").apply { init(Cipher.ENCRYPT_MODE, wrapKey()) }
        val ct = c.doFinal(bytes)
        return Base64.encodeToString(c.iv, Base64.NO_WRAP) + ":" + Base64.encodeToString(ct, Base64.NO_WRAP)
    }

    private fun unwrap(packed: String): ByteArray {
        val (iv, ct) = packed.split(":", limit = 2).map { Base64.decode(it, Base64.NO_WRAP) }
        return Cipher.getInstance("AES/GCM/NoPadding")
            .apply { init(Cipher.DECRYPT_MODE, wrapKey(), GCMParameterSpec(128, iv)) }
            .doFinal(ct)
    }

    /** Synchronous: returns true only once the values are on disk. */
    fun save(ctx: Context, seed: ByteArray?, secrets: ByteArray): Boolean = try {
        val e = ctx.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit()
        seed?.let { e.putString(SEED, wrap(it)) }
        e.putString(SECRETS, wrap(secrets))
        e.commit()
    } catch (e: Throwable) {
        false
    }

    private fun read(ctx: Context, key: String): ByteArray? = try {
        ctx.getSharedPreferences(PREFS, Context.MODE_PRIVATE).getString(key, null)?.let { unwrap(it) }
    } catch (e: Throwable) {
        null
    }

    fun seed(ctx: Context): ByteArray? = read(ctx, SEED)
    fun secrets(ctx: Context): ByteArray? = read(ctx, SECRETS)

    /** Where the wrap key lives. Expected to be software on the emulator. */
    fun securityLevel(): String = try {
        val key = wrapKey()
        val info = SecretKeyFactory.getInstance(key.algorithm, "AndroidKeyStore").getKeySpec(key, KeyInfo::class.java) as KeyInfo
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            when (info.securityLevel) {
                KeyProperties.SECURITY_LEVEL_SOFTWARE -> "software"
                KeyProperties.SECURITY_LEVEL_TRUSTED_ENVIRONMENT -> "TEE"
                KeyProperties.SECURITY_LEVEL_STRONGBOX -> "StrongBox"
                else -> "level ${info.securityLevel}"
            }
        } else {
            @Suppress("DEPRECATION")
            if (info.isInsideSecureHardware) "secure hardware" else "software"
        }
    } catch (e: Throwable) {
        "unknown (${e::class.simpleName})"
    }
}
