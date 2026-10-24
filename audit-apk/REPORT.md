# APK audit report

File: `Concat-0.2.8-android-arm64.apk`  size: 80155688 bytes

SHA-256: `0f308588ec24cd8d9dc7328e683a910cdad6019d50fdd82e39d4511c4222eabe`

Static analysis only; nothing was executed. Absence of findings is not proof of absence of malware.

## Signature

```
Verifies
Verified using v1 scheme (JAR signing): false
Verified using v2 scheme (APK Signature Scheme v2): true
Verified using v3 scheme (APK Signature Scheme v3): true
Verified using v3.1 scheme (APK Signature Scheme v3.1): false
Verified using v3.2 scheme (APK Signature Scheme v3.2): false
Verified using v4 scheme (APK Signature Scheme v4): false
Verified for SourceStamp: false
Number of signers: 1
V3.0 Signer: certificate DN: CN=Concat, O=Concat
V3.0 Signer: certificate SHA-256 digest: 634d87c66c1a30bc89bf58f798afc41082d701dcbb6d136f8c6bdd240efc233e
V3.0 Signer: certificate SHA-1 digest: 96f22b3c7e319e3ac1231fc007a6098c5927ac17
V3.0 Signer: certificate MD5 digest: d57231518d76d4d8fd79d1eca9ceb360
V3.0 Signer: key algorithm: RSA
V3.0 Signer: key size (bits): 2048
V3.0 Signer: public key SHA-256 digest: f5d073320418943631170c25e067d84ee55d9b22399d362e132679e07cbf812e
V3.0 Signer: public key SHA-1 digest: b438e7da3b20308e11e33144db691d6b25bc6562
V3.0 Signer: public key MD5 digest: 752190ad69eb47c5c489a414da3f475e
```

## Permissions and application info (aapt2 badging)

```
package: name='app.concat.editor' versionCode='16777736' versionName='0.2.8' platformBuildVersionName='15' platformBuildVersionCode='35' compileSdkVersion='35' compileSdkVersionCodename='15'
targetSdkVersion:'35'
uses-permission: name='android.permission.INTERNET'
uses-permission: name='android.permission.RECORD_AUDIO'
application-label:'Concat'
launchable-activity: name='android.app.NativeActivity'  label='' icon=''
supports-screens: 'small' 'normal' 'large' 'xlarge'
native-code: 'arm64-v8a'
```

## Manifest details (components, exported, cleartext, backup)

```
A: http://schemas.android.com/apk/res/android:name(0x01010003)="android.permission.INTERNET" (Raw: "android.permission.INTERNET")
A: http://schemas.android.com/apk/res/android:name(0x01010003)="android.permission.RECORD_AUDIO" (Raw: "android.permission.RECORD_AUDIO")
A: http://schemas.android.com/apk/res/android:debuggable(0x0101000f)=false
E: activity (line=1)
A: http://schemas.android.com/apk/res/android:name(0x01010003)="android.app.NativeActivity" (Raw: "android.app.NativeActivity")
A: http://schemas.android.com/apk/res/android:exported(0x01010010)=true
E: meta-data (line=1)
A: http://schemas.android.com/apk/res/android:name(0x01010003)="android.app.lib_name" (Raw: "android.app.lib_name")
E: intent-filter (line=1)
E: action (line=1)
A: http://schemas.android.com/apk/res/android:name(0x01010003)="android.intent.action.MAIN" (Raw: "android.intent.action.MAIN")
E: category (line=1)
A: http://schemas.android.com/apk/res/android:name(0x01010003)="android.intent.category.LAUNCHER" (Raw: "android.intent.category.LAUNCHER")
```

## Files in the package

```
      2728  AndroidManifest.xml
      1196  resources.arsc
 181190112  lib/arm64-v8a/libconcat_android.so
   1794776  lib/arm64-v8a/libc++_shared.so
   4456784  lib/arm64-v8a/libsherpa-onnx-c-api.so
  21684880  lib/arm64-v8a/libonnxruntime.so
      1285  META-INF/CONCAT.SF
      1160  META-INF/CONCAT.RSA
      1158  META-INF/MANIFEST.MF
(+ 5 files under res/)
```

## Executable code outside the expected places

```
none found
```

## Native libraries

```
cc3150996a646172   181190112  lib/arm64-v8a/libconcat_android.so
d523468d62d9b603     1794776  lib/arm64-v8a/libc++_shared.so
73dba26ddf63e47e     4456784  lib/arm64-v8a/libsherpa-onnx-c-api.so
dc5e4c172b1be9e5    21684880  lib/arm64-v8a/libonnxruntime.so
```

## Undefined (imported) symbols by category: lib/arm64-v8a/libconcat_android.so

```
0x0000000000000001 (NEEDED)             Shared library: [libsherpa-onnx-c-api.so]
0x0000000000000001 (NEEDED)             Shared library: [libc++_shared.so]
0x0000000000000001 (NEEDED)             Shared library: [libz.so]
0x0000000000000001 (NEEDED)             Shared library: [libm.so]
0x0000000000000001 (NEEDED)             Shared library: [libandroid.so]
0x0000000000000001 (NEEDED)             Shared library: [libmediandk.so]
0x0000000000000001 (NEEDED)             Shared library: [liblog.so]
0x0000000000000001 (NEEDED)             Shared library: [libEGL.so]
0x0000000000000001 (NEEDED)             Shared library: [libGLESv2.so]
0x0000000000000001 (NEEDED)             Shared library: [libaaudio.so]
0x0000000000000001 (NEEDED)             Shared library: [libdl.so]
0x0000000000000001 (NEEDED)             Shared library: [libc.so]
```
```
network: accept, bind, connect, getaddrinfo, listen, recv, recvfrom, send, sendto, socket
process execution: execvp, fork
debug / injection: prctl
dynamic loading: dlopen, dlsym
files: chmod, remove, rename, rmdir, unlink, unlinkat
(total undefined symbols: 768)
```

## Undefined (imported) symbols by category: lib/arm64-v8a/libc++_shared.so

```
0x0000000000000001 (NEEDED)             Shared library: [libc.so]
0x0000000000000001 (NEEDED)             Shared library: [libm.so]
0x0000000000000001 (NEEDED)             Shared library: [libdl.so]
```
```
network: -
process execution: -
debug / injection: -
dynamic loading: -
files: remove, rename, unlinkat
(total undefined symbols: 161)
```

## Undefined (imported) symbols by category: lib/arm64-v8a/libsherpa-onnx-c-api.so

```
0x0000000000000001 (NEEDED)             Shared library: [libandroid.so]
0x0000000000000001 (NEEDED)             Shared library: [liblog.so]
0x0000000000000001 (NEEDED)             Shared library: [libonnxruntime.so]
0x0000000000000001 (NEEDED)             Shared library: [libm.so]
0x0000000000000001 (NEEDED)             Shared library: [libdl.so]
0x0000000000000001 (NEEDED)             Shared library: [libc.so]
```
```
network: -
process execution: popen
debug / injection: -
dynamic loading: dlopen
files: remove
(total undefined symbols: 170)
```

## Undefined (imported) symbols by category: lib/arm64-v8a/libonnxruntime.so

```
0x0000000000000001 (NEEDED)             Shared library: [libdl.so]
0x0000000000000001 (NEEDED)             Shared library: [liblog.so]
0x0000000000000001 (NEEDED)             Shared library: [libm.so]
0x0000000000000001 (NEEDED)             Shared library: [libc.so]
```
```
network: -
process execution: -
debug / injection: prctl
dynamic loading: dlopen, dlsym
files: remove
(total undefined symbols: 214)
```

## URLs and hosts in libconcat_android.so

```
hosts:
  android.googlesource.com
  aomedia.org
  api.github.com
  arxiv.org
  concat.standalones.orghttps
  dashif.org
  dharmatype.comhttp
  docs.nvidia.com
  docs.rs
  en.wikipedia.org
  fonts.floriankarsten.com
  github.com
  hf-mirror.com
  huggingface.co
  ieeexplore.ieee.org
  kimberlygeswein.comthis
  models.example.com
  ns.adobe.com
  numpy.org
  omnibus-type.com
  raw.githubusercontent.com
  registry.khronos.org
  scripts.sil.org
  standards.iso.org
  streams.videolan.org
  tinyurl.com
  www.adobe.com
  www.apache.org
  www.fontbros.com
  www.fontdiner.comlicensed
  www.iec.ch
  www.impallari.comhttp
  www.sansoxygen.comhttp
  www.tipospereira.comhttp
  www.w3.org
```
```
urls (first 120 chars):
  http://dashif.org/guidelines/trickmode
  http://dharmatype.comhttp://dharmatype.comThis
  http://kimberlygeswein.comThis
  http://ns.adobe.com/f4m/1.0
  http://omnibus-type.com/This
  http://scripts.sil.org/OFL
  http://scripts.sil.org/OFLHammersmith
  http://scripts.sil.org/OFLhttp://scripts.sil.org/OFL
  http://scripts.sil.org/http://scripts.sil.org/OFL
  http://standards.iso.org/ittf/PubliclyAvailableStandards/MPEG-DASH_schema_files/DASH-MPD.xsd
  http://www.adobe.com/).
  http://www.apache.org/licenses/LICENSE-2.0
  http://www.fontbros.com/foundries/neapolitanhttp://www.tartworkshop.comThis
  http://www.fontbros.com/sideshow.phphttp://www.squidart.comThis
  http://www.fontdiner.comLicensed
  http://www.iec.ch
  http://www.impallari.comhttp://www.impallari.comThis
  http://www.sansoxygen.comhttp://www.sansoxygen.comThis
  http://www.tipospereira.comhttp://www.marcelomagalhaes.netThis
  http://www.w3.org/1999/xlink
  http://www.w3.org/2000/svg
  http://www.w3.org/2000/svgltrinlinevisibleblack041&usvg::parser::svgtree::parseusvg::parser::svgtree
  http://www.w3.org/2000/xmlns/
  http://www.w3.org/2001/SMIL20/Language
  http://www.w3.org/2001/XMLSchema-instance
  http://www.w3.org/TR/SVG11/feature#Cliphttp://www.w3.org/TR/SVG11/feature#BasicCliphttp://www.w3.org/TR/SVG11/feature#Ma
  http://www.w3.org/TR/SVG11/feature#ContainerAttributehttp://www.w3.org/TR/SVG11/feature#ConditionalProcessinghttp://www.
  http://www.w3.org/TR/SVG11/feature#GraphicsAttributehttp://www.w3.org/TR/SVG11/feature#BasicGraphicsAttributehttp://www.
  http://www.w3.org/TR/SVG11/feature#SVGDOM-statichttp://www.w3.org/TR/SVG11/feature#SVG-statichttp://www.w3.org/TR/SVG11/
  http://www.w3.org/TR/SVG11/feature#Texthttp://www.w3.org/TR/SVG11/feature#BasicTexthttp://www.w3.org/TR/SVG11/feature#Pa
  http://www.w3.org/TR/SVG11/feature#XlinkAttributehttp://www.w3.org/1999/xlink
  http://www.w3.org/XML/1998/namespace
  http://www.w3.org/XML/1998/namespacexmlinternal
  http://www.w3.org/ns/ttml
  http://www.w3.org/ns/ttml#metadata
  http://www.w3.org/ns/ttml#parameter
  http://www.w3.org/ns/ttml#styling
  https://android.googlesource.com/toolchain/llvm-project
  https://aomedia.org/emsg/ID3
  https://api.github.com/repos/jub0t/Concat/releases?per_page=100messagethe
  https://arxiv.org/abs/1502.03167.
  https://concat.standalones.orghttps://github.com/jub0t/Concathttps://discord.gg/DVuPfpXfqPstart.dropNothingToOpenstart.n
  https://docs.nvidia.com/cuda/cublas/index.html#cublasLtOrder_t
  https://docs.rs/rustls/latest/rustls/manual/_03_howto/index.html#unexpected-eof
  https://en.wikipedia.org/wiki/Rounding
  https://fonts.floriankarsten.com
  https://github.com/
  https://github.com/Omnibus-Type/ArchivoBlack)Archivo
  https://github.com/PeterL1n/RobustVideoMatting/releases/download/v1.0.0/rvm_mobilenetv3_fp32.onnx88d4531297118f595bf2fd6
  https://github.com/dharmatype/Bebas-Neue)Bebas
  https://github.com/floriankarsten/space-grotesk)
  https://github.com/gfx-rs/wgpu/issues/
  https://github.com/gfx-rs/wgpu/issues/5276
  https://github.com/gfx-rs/wgpu/issues/5276RayQueryVertexPositionsCooperativeMatrixloop_boundMatrix
  https://github.com/gfx-rs/wgpu/issues/5320
  https://github.com/gfx-rs/wgpu/issues/8333)Feature
  https://github.com/gfx-rs/wgpuDevice::create_render_pipelineDevice::create_shader_modulefound
  https://github.com/google/re2/wiki/Syntax).
  https://github.com/googlefonts/Pacifico)PacificoRegular3.001;newt;Pacifico-RegularPacifico
  https://github.com/impallari/The-Lobster-Font),
  https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/
  https://github.com/marcelommp/Londrina-Typeface),
  https://hf-mirror.com/the
  https://huggingface.co/CoderViking/realesr-general-x4v3-onnx/resolve/c6a971706797c7502945a2b4c4274fce4900d4ab/realesr-ge
  https://huggingface.co/ResembleAI/chatterbox-turbo-ONNX/resolve/d21799bd0354adb85e348b8a0442a8405110a2cf/onnx/conditiona
  https://huggingface.co/ResembleAI/chatterbox-turbo-ONNX/resolve/d21799bd0354adb85e348b8a0442a8405110a2cf/onnx/conditiona
  https://huggingface.co/ResembleAI/chatterbox-turbo-ONNX/resolve/d21799bd0354adb85e348b8a0442a8405110a2cf/onnx/embed_toke
  https://huggingface.co/ResembleAI/chatterbox-turbo-ONNX/resolve/d21799bd0354adb85e348b8a0442a8405110a2cf/onnx/embed_toke
  https://huggingface.co/ResembleAI/chatterbox-turbo-ONNX/resolve/d21799bd0354adb85e348b8a0442a8405110a2cf/onnx/language_m
  https://huggingface.co/ResembleAI/chatterbox-turbo-ONNX/resolve/d21799bd0354adb85e348b8a0442a8405110a2cf/onnx/language_m
  https://huggingface.co/ResembleAI/chatterbox-turbo-ONNX/resolve/d21799bd0354adb85e348b8a0442a8405110a2cf/onnx/speech_enc
  https://huggingface.co/ResembleAI/chatterbox-turbo-ONNX/resolve/d21799bd0354adb85e348b8a0442a8405110a2cf/onnx/speech_enc
  https://huggingface.co/ResembleAI/chatterbox-turbo-ONNX/resolve/d21799bd0354adb85e348b8a0442a8405110a2cf/tokenizer.json3
  https://huggingface.co/Xenova/slimsam-77-uniform/resolve/5850ab45f587c112167512ffef949107115e26a0/onnx/prompt_encoder_ma
  https://huggingface.co/Xenova/slimsam-77-uniform/resolve/5850ab45f587c112167512ffef949107115e26a0/onnx/vision_encoder.on
  https://huggingface.co/ggerganov/whisper.cpp/resolve/
  https://ieeexplore.ieee.org/document/1163711
  https://models.example.com/concat127.0.0.1project.hdrMonitorNotecommon.settings
  https://numpy.org/doc/stable/reference/generated/numpy.matmul.html).
  https://raw.githubusercontent.com/
  https://raw.githubusercontent.com/concatapp/assets/main/audio/manifest.jsonmanifest.jsonSoundidaddedbyteslicensesourceau
  https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#devsandqueues-lost-device
  https://scripts.sil.org/OFL
  https://streams.videolan.org/upload/
  https://tinyurl.com/sudb9s96),
  https://www.w3.org/TR/WGSL/#diagnostic-severity
  https://www.w3.org/TR/WGSL/#enable-extension
  https://www.w3.org/TR/WGSL/#filterable-triggering-rules
  https://www.w3.org/TR/WGSL/#language-extensions-sec
```

## Keywords in libconcat_android.so (case-insensitive counts)

```
discord.gg: 1
stealer: 4
keylog: 7
frida: 1
xposed: 42
sentry: 36
amplitude: 14
facebook: 2
```

## Rust crates embedded in libconcat_android.so vs Cargo.lock

```
crates found: 141
not in Cargo.lock: 0
[]
version differs from Cargo.lock: 0
[]
```

## Java code (dex)

```
dex files: []
total size: 0 bytes
```
```
none of the listed Java APIs
```
```
application classes referenced:
```
```
urls in dex:
none
```
