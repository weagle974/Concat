#!/usr/bin/env python3
"""Static, read-only audit of an Android APK. Nothing in the APK is executed.

Usage: audit_apk.py <apk> <Cargo.lock> <report.md>

Writes a Markdown report: signature, manifest (permissions, exported
components, cleartext traffic), every file in the package, the native
libraries' undefined symbols by category, URLs and keywords found in the
native code and the dex, and the Rust crates embedded in the native library
compared with the versions in Cargo.lock.
"""
import collections
import glob
import hashlib
import os
import re
import subprocess
import sys
import zipfile

apk_path, lock_path, report_path = sys.argv[1:4]
out = []


def h(title):
    out.append(f"\n## {title}\n")


def code(text, lang=""):
    out.append(f"```{lang}\n{text.rstrip()}\n```")


def run(cmd, timeout=120):
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
        return (r.stdout + (("\n[stderr]\n" + r.stderr) if r.stderr.strip() else "")).strip()
    except Exception as e:  # a missing tool must not stop the audit
        return f"[tool unavailable: {cmd[0]}: {e}]"


def sdk_tool(name):
    root = os.environ.get("ANDROID_HOME") or os.environ.get("ANDROID_SDK_ROOT") or ""
    found = sorted(glob.glob(os.path.join(root, "build-tools", "*", name)))
    return found[-1] if found else name


apk = open(apk_path, "rb").read()
out.append("# APK audit report")
out.append(f"\nFile: `{os.path.basename(apk_path)}`  size: {len(apk)} bytes")
out.append(f"\nSHA-256: `{hashlib.sha256(apk).hexdigest()}`")
out.append("\nStatic analysis only; nothing was executed. Absence of findings is not proof of absence of malware.")

# --- signature -------------------------------------------------------------
h("Signature")
code(run([sdk_tool("apksigner"), "verify", "--print-certs", "--verbose", apk_path]))

# --- manifest --------------------------------------------------------------
h("Permissions and application info (aapt2 badging)")
badging = run([sdk_tool("aapt2"), "dump", "badging", apk_path])
keep = [l for l in badging.splitlines() if re.match(
    r"(package|sdkVersion|targetSdkVersion|uses-permission|permission|application-label:|launchable|uses-feature|native-code|supports-screens)", l)]
code("\n".join(keep) or badging[:3000])

h("Manifest details (components, exported, cleartext, backup)")
tree = run([sdk_tool("aapt2"), "dump", "xmltree", "--file", "AndroidManifest.xml", apk_path])
flag_lines = [l.strip() for l in tree.splitlines() if re.search(
    r"(E: (activity|service|receiver|provider|intent-filter|uses-library|meta-data)|android:name|exported|usesCleartextTraffic|allowBackup|debuggable|networkSecurityConfig|action|category|scheme)", l)]
code("\n".join(flag_lines[:200]) or tree[:4000])

# --- package contents ------------------------------------------------------
z = zipfile.ZipFile(apk_path)
infos = z.infolist()
h("Files in the package")
rows = [f"{i.file_size:>10}  {i.filename}" for i in infos if not i.filename.startswith("res/")]
res_count = sum(1 for i in infos if i.filename.startswith("res/"))
code("\n".join(rows) + f"\n(+ {res_count} files under res/)")

# Anything that looks executable outside lib/
odd = []
for i in infos:
    if i.filename.startswith("lib/") or i.is_dir():
        continue
    head = z.read(i)[:4]
    if head == b"\x7fELF":
        odd.append(f"ELF outside lib/: {i.filename}")
    if head[:2] == b"PK" and not i.filename.endswith((".apk", ".jar")):
        odd.append(f"nested archive: {i.filename}")
    if head[:4] in (b"dex\n", ) and i.filename != "classes.dex" and not re.match(r"classes\d*\.dex", i.filename):
        odd.append(f"extra dex: {i.filename}")
h("Executable code outside the expected places")
code("\n".join(odd) or "none found")

# --- native libraries -------------------------------------------------------
CATS = {
    "network": ["socket", "connect", "getaddrinfo", "send", "sendto", "sendmsg", "recv", "recvfrom", "gethostbyname", "bind", "listen", "accept"],
    "process execution": ["execve", "execv", "execvp", "execl", "fork", "vfork", "system", "popen", "posix_spawn", "posix_spawnp"],
    "debug / injection": ["ptrace", "process_vm_readv", "process_vm_writev", "prctl"],
    "dynamic loading": ["dlopen", "dlsym", "android_dlopen_ext"],
    "files": ["unlink", "unlinkat", "rmdir", "remove", "rename", "chmod", "chown", "mkfifo"],
}
KEYWORDS = [
    "pastebin", "webhook", "telegram", "discord.gg", "ngrok", "stealer", "mimikatz", "keylog",
    "/system/xbin/su", "/system/bin/su", "Superuser", "magisk", "frida", "xposed",
    "api.ipify", "posthog", "sentry", "mixpanel", "amplitude", "firebase", "crashlytics",
    "google-analytics", "datadog", "bugsnag", "appsflyer", "adjust.com", "facebook",
]
DEX_KEYWORDS = [
    "DexClassLoader", "PathClassLoader", "Runtime;->exec", "ProcessBuilder", "getPrimaryClip",
    "AccessibilityService", "SmsManager", "TelephonyManager", "getDeviceId", "getImei", "getSubscriberId",
    "READ_CONTACTS", "READ_SMS", "ACCESS_FINE_LOCATION", "LocationManager", "getInstalledPackages",
    "WebView", "loadUrl", "HttpURLConnection", "okhttp", "java/net/Socket", "MediaProjection",
    "AccountManager", "getAccounts", "KeyStore", "BiometricPrompt",
]

libs = [i for i in infos if i.filename.startswith("lib/") and i.filename.endswith(".so")]
h("Native libraries")
lib_rows = []
for i in libs:
    data = z.read(i)
    lib_rows.append(f"{hashlib.sha256(data).hexdigest()[:16]}  {i.file_size:>10}  {i.filename}")
code("\n".join(lib_rows))

tmpdir = "/tmp/apk_audit_libs"
os.makedirs(tmpdir, exist_ok=True)
main_lib_data = None
for i in libs:
    name = os.path.basename(i.filename)
    path = os.path.join(tmpdir, name)
    data = z.read(i)
    open(path, "wb").write(data)
    if name == "libconcat_android.so":
        main_lib_data = data
    h(f"Undefined (imported) symbols by category: {i.filename}")
    needed = run(["readelf", "-d", path])
    code("\n".join(l.strip() for l in needed.splitlines() if "NEEDED" in l) or "(no NEEDED entries)")
    syms = run(["readelf", "--dyn-syms", "-W", path], timeout=180)
    undefined = set()
    for line in syms.splitlines():
        cols = line.split()
        if len(cols) >= 8 and cols[6] == "UND":
            undefined.add(cols[7].split("@")[0])
    lines = []
    for cat, names in CATS.items():
        hit = sorted(n for n in undefined if n in names)
        lines.append(f"{cat}: {', '.join(hit) if hit else '-'}")
    lines.append(f"(total undefined symbols: {len(undefined)})")
    code("\n".join(lines))

# --- strings in the main native library ----------------------------------
if main_lib_data:
    h("URLs and hosts in libconcat_android.so")
    urls = collections.Counter(m.group().decode("latin1") for m in re.finditer(
        rb"https?://[A-Za-z0-9._~:/?#\[\]@!$&'()*+,;=%-]{4,200}", main_lib_data))
    hosts = collections.Counter()
    for u in urls:
        m = re.match(r"https?://([^/:?#]+)", u)
        if m:
            hosts[m.group(1).lower()] += 1
    code("hosts:\n" + "\n".join(f"  {h_}" for h_ in sorted(hosts)))
    code("urls (first 120 chars):\n" + "\n".join(f"  {u[:120]}" for u in sorted(urls)))

    h("Keywords in libconcat_android.so (case-insensitive counts)")
    lines = []
    for k in KEYWORDS:
        hits = list(re.finditer(re.escape(k.encode()), main_lib_data, flags=re.I))
        if hits:
            lines.append(f"{k}: {len(hits)}")
            # Where each word sits, so an ordinary word ("exposed", a C++
            # iostream "sentry", work-"stealer") can be told from a real hit.
            seen = set()
            for m in hits:
                a, b = max(0, m.start() - 50), min(len(main_lib_data), m.end() + 50)
                seg = re.sub(rb"[^\x20-\x7e]", b".", main_lib_data[a:b]).decode()
                if seg not in seen:
                    seen.add(seg)
                    lines.append(f"    ...{seg}...")
                if len(seen) >= 3:
                    break
    code("\n".join(lines) or "none of the listed keywords")

    h("Rust crates embedded in libconcat_android.so vs Cargo.lock")
    crates = set()
    for m in re.finditer(rb"registry[/\\]src[/\\][^/\\]+[/\\]([A-Za-z0-9_.+-]+?)-(\d+\.\d+\.\d+[A-Za-z0-9.+-]*)[/\\]", main_lib_data):
        crates.add((m.group(1).decode(), m.group(2).decode()))
    locked = collections.defaultdict(set)
    try:
        txt = open(lock_path, encoding="utf-8").read()
        for blk in txt.split("[[package]]")[1:]:
            n = re.search(r'^name = "([^"]+)"', blk, re.M)
            v = re.search(r'^version = "([^"]+)"', blk, re.M)
            if n and v:
                locked[n.group(1)].add(v.group(1))
    except Exception as e:
        out.append(f"(Cargo.lock unreadable: {e})")
    unknown = sorted((n, v) for n, v in crates if n not in locked)
    different = sorted((n, v, sorted(locked[n])) for n, v in crates if n in locked and v not in locked[n])
    code(f"crates found: {len(crates)}\nnot in Cargo.lock: {len(unknown)}\n{unknown}\n"
         f"version differs from Cargo.lock: {len(different)}\n{different}")

# --- dex ------------------------------------------------------------------
dex_names = [i.filename for i in infos if re.match(r"classes\d*\.dex$", i.filename)]
h("Java code (dex)")
dex_blob = b"".join(z.read(n) for n in dex_names)
code(f"dex files: {dex_names}\ntotal size: {len(dex_blob)} bytes")
lines = []
for k in DEX_KEYWORDS:
    n = len(re.findall(re.escape(k.encode()), dex_blob))
    if n:
        lines.append(f"{k}: {n}")
code("\n".join(lines) or "none of the listed Java APIs")
classes = sorted(set(m.group().decode() for m in re.finditer(rb"L[a-z][A-Za-z0-9_/$]{3,80};", dex_blob)))
app_classes = [c for c in classes if "concat" in c.lower() or c.startswith("Lapp/") or "slint" in c.lower()]
code("application classes referenced:\n" + "\n".join(app_classes[:80]))
dex_urls = sorted(set(m.group().decode("latin1") for m in re.finditer(rb"https?://[A-Za-z0-9._~:/?#@!$&'()*+,;=%-]{4,160}", dex_blob)))
code("urls in dex:\n" + ("\n".join(dex_urls) or "none"))

open(report_path, "w", encoding="utf-8").write("\n".join(out) + "\n")
print(f"report written: {report_path} ({sum(len(x) for x in out)} chars)")
