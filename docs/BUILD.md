# 构建指南

[← 返回首页](../README.md) · [架构说明](ARCHITECTURE.md) · [实机验证](../design/DEVICE_VALIDATION.md)

## 工具链

以下是当前工作区实际使用的组合，不代表已验证其他版本组合。

| 工具 | 当前版本 / 要求 |
| :-- | :-- |
| 系统与脚本 | Windows + PowerShell，当前完整构建入口为 `build.ps1` |
| JDK | 21；Android Java / Kotlin 字节码目标为 17 |
| Rust | 稳定版工具链，安装 `aarch64-linux-android` 与 `wasm32-unknown-unknown` target |
| WASM 绑定 | `wasm-bindgen-cli` 0.2.108，与 UI Cargo.lock 保持一致 |
| Android SDK | platform `android-37.2`，compileSdk 37 / minor 2 |
| Android NDK | 已验证 `27.2.12479018` |
| Gradle | Wrapper 9.7.1，无需另装全局 Gradle |
| Android Gradle Plugin | 9.4.1 |
| Kotlin / Compose 插件 | 2.4.21 |
| MIUIX | 0.9.5-beta01 |
| 应用目标 | minSdk 26 / targetSdk 35 / arm64-v8a |

不要为了绕过依赖版本检查而降低 compileSdk。首次构建需要能访问 Google Maven、Maven Central、Gradle 和 crates.io。

## 完整构建

```powershell
git clone https://github.com/Memories-white/Auralis.git
cd Auralis

# 请改为本机实际安装目录。
$env:JAVA_HOME = 'C:\Program Files\Java\jdk-21'
$env:ANDROID_HOME = 'C:\Android\Sdk'
$env:ANDROID_NDK_HOME = Join-Path $env:ANDROID_HOME 'ndk\27.2.12479018'
$env:PATH = "$env:JAVA_HOME\bin;$env:PATH"

rustup target add aarch64-linux-android
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.108 --locked
.\build.ps1
```

私有仓库需要先配置自己的 GitHub 访问权限。示例路径是占位路径，不包含项目维护者的本机配置。

脚本依次执行：

1. 编译 `ui/` 的 Rust/WASM 界面，生成本机 WebView 资源。
2. 配置 NDK arm64 clang、ar 和 Rust linker，构建 `core/` release 动态库。
3. 复制动态库到 `android/app/src/main/jniLibs/arm64-v8a/`。
4. 通过 Gradle Wrapper 执行 `:app:assembleDebug`；缺失或过期 UI 资源会使构建失败。

最终产物：

```text
android/app/build/outputs/apk/debug/app-debug.apk
```

仓库不包含预编译 `.so`、WASM 或自动生成的绑定。新克隆必须先完成 Rust 构建；直接执行 Android `assembleDebug` 不能替代完整构建。只重建界面可运行 `build-ui.ps1`。运行时需要较新的 Android System WebView，不会从远程网站加载界面。

### 离线与 AAPT2

依赖已缓存时：

```powershell
.\build.ps1 -Offline
```

如本机确实需要使用已经安装的兼容 AAPT2，可传入其完整路径：

```powershell
.\build.ps1 -Offline -Aapt2Path 'C:\Android\tools\aapt2.exe'
```

该参数不是必需项，也不保证任意旧 AAPT2 都能处理当前资源。SDK 路径、Gradle 缓存、签名文件与认证令牌均不得提交。

## 本地检查

```powershell
.\test.ps1 -Offline
```

运行 Rust UI 与协议单元测试，以及独立的 Java 播放顺序、图片磁盘缓存、封面取色和睡眠定时检查。这些测试不需要账号登录，也不会读取真实账号凭据。仅做协议层测试时可运行：

```powershell
cargo test --manifest-path core/Cargo.toml --locked --offline
```

这些检查不能替代真实设备上的媒体通知、音质切换、动画或 LyriconProvider 联调。当前验证范围见 [设备验证记录](../design/DEVICE_VALIDATION.md)。

### Rust 界面交互检查

已完成 `build-ui.ps1` 后，在临时工具目录安装 Playwright 与 Chromium，设置 `PLAYWRIGHT_MODULE` 为该工具目录的 `node_modules/playwright`，然后运行：

```powershell
node tools/check-rust-ui.cjs
```

检查真正的 Rust/WASM 页面，包括左右滑动、播放器从歌曲栏展开、歌词焦点 / 浏览恢复、搜索焦点、歌单分组、加载 / 重试、设置与半屏面板。平台服务使用测试桥接，截图明确使用测试数据。实机 DevTools 检查另见 `tools/check-device-rust-ui.cjs`，仅适用于已授权 USB 调试的 debug 安装包。

### 本地音频实机集成检查

暂停现有播放并关闭现有睡眠定时后，在已安装同签名主 APK 的调试设备上运行：

```powershell
.\android\gradlew.bat -p android :app:assembleDebugAndroidTest
adb install -r -t android/app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk
adb shell am instrument -w dev.qqlyric.app.test/dev.qqlyric.app.PlaybackSmokeTest
adb shell am instrument -w -e mode restore dev.qqlyric.app.test/dev.qqlyric.app.PlaybackSmokeTest
adb shell am instrument -w -e mode playlist dev.qqlyric.app.test/dev.qqlyric.app.PlaybackSmokeTest
```

测试 APK 生成两分钟的本地静音 WAV，通过正常播放服务验证成功播放记录、暂停恢复去重、歌曲结束定时优先于循环，以及真实一分钟倒计时暂停，耗时约 70 秒。它临时修改播放状态，完成后恢复原最近播放记录与顺序偏好并停止测试音频；请勿在收听音乐时运行。测试类仅存在于 `androidTest`，不打入主 APK，也不开放额外外部控制入口。

`mode restore` 检查元数据白名单、暂停恢复、队列身份、进度保存、停止服务后回显、有界队列与损坏 / 未知版本记录；不播放音频，结束后恢复原播放记录。默认播放检查也会恢复原恢复快照。测试组件可通过 `adb uninstall dev.qqlyric.app.test` 移除。

`mode playlist` 使用受控响应检查歌单立即进入、加载、重复点击、失败、重试、返回及旧响应保护，不访问真实推荐接口。该检查与真实账号下的页面点击 / 内容验证分别记录。

## 安装与签名

```powershell
adb install -r android/app/build/outputs/apk/debug/app-debug.apk
Get-FileHash android/app/build/outputs/apk/debug/app-debug.apk -Algorithm SHA256
```

`-r` 仅在包名和签名兼容时保留数据覆盖安装。仓库不分发开发签名私钥；其他电脑的默认 debug keystore 会生成不同签名，通常不能覆盖维护者的 APK。不要为解决签名冲突而直接清除账号和应用数据。

当前没有自动发布流水线或正式发行签名配置，Release 中的安装包为已验证的开发构建。
