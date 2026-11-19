# 构建指南

[← 返回首页](../README.md) · [架构说明](ARCHITECTURE.md) · [实机验证](../design/DEVICE_VALIDATION.md)

## 工具链

以下是当前工作区实际使用的组合，不代表已验证其他版本组合。

| 工具 | 当前版本 / 要求 |
| :-- | :-- |
| 系统与脚本 | Windows + PowerShell，当前完整构建入口为 `build.ps1` |
| JDK | 21；Android Java / Kotlin 字节码目标为 17 |
| Rust | 稳定版工具链，安装 `aarch64-linux-android` target |
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
.\build.ps1
```

私有仓库需要先配置自己的 GitHub 访问权限。示例路径是占位路径，不包含项目维护者的本机配置。

脚本依次执行：

1. 配置 NDK arm64 clang、ar 和 Rust linker。
2. 在 `core/` 中构建 `aarch64-linux-android` release 动态库。
3. 复制动态库到 `android/app/src/main/jniLibs/arm64-v8a/`。
4. 通过 Gradle Wrapper 执行 `:app:assembleDebug`。

最终产物：

```text
android/app/build/outputs/apk/debug/app-debug.apk
```

仓库不包含预编译 `.so`。新克隆必须先完成 Rust 构建；直接执行 Android `assembleDebug` 不能替代完整构建。

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

运行 Rust 单元测试，以及独立的 Java 播放顺序、图片磁盘缓存和封面取色检查。这些测试不需要 QQ 登录，也不会读取真实账号凭据。仅做协议层测试时可运行：

```powershell
cargo test --manifest-path core/Cargo.toml --locked --offline
```

这些检查不能替代真实设备上的媒体通知、音质切换、动画或 LyriconProvider 联调。当前验证范围见 [设备验证记录](../design/DEVICE_VALIDATION.md)。

## 安装与签名

```powershell
adb install -r android/app/build/outputs/apk/debug/app-debug.apk
Get-FileHash android/app/build/outputs/apk/debug/app-debug.apk -Algorithm SHA256
```

`-r` 仅在包名和签名兼容时保留数据覆盖安装。仓库不分发开发签名私钥；其他电脑的默认 debug keystore 会生成不同签名，通常不能覆盖维护者的 APK。不要为解决签名冲突而直接清除账号和应用数据。

当前没有自动发布流水线或正式发行签名配置，Release 中的安装包为已验证的开发构建。
