# Auralis · AI 工作交接

> 每次改动后更新此表；入口约定已写入根目录 `AGENTS.md`。布局、交互、设置校验和动画以 Rust 为主，Android 只做必要的平台桥接。

## 当前交付

| 项目 | 状态 / 入口 |
| :-- | :-- |
| 日期、版本 | 2026-11-27 · `0.4.4-preview` · versionCode `25` |
| 基线 | `d65ad65` · 0.4.3-preview，逐行上浮、播放器共享封面与刷新率偏好 |
| 当前提交 | 本文与本次实现同一个提交；使用 `git log -1` 获取完整 SHA，避免文档自引用哈希 |
| 最新本地包 | `APKs/Auralis-v0.4.4-preview.apk`（构建与安装证据见下方验证表） |
| APK SHA256 | `E7824195A91FEDE1B51ECE6DFE1E6EFC47239F2C01E7A844EAD0E92B3782E80C` |
| 仓库 | [Memories-white/Auralis](https://github.com/Memories-white/Auralis) · public |
| 已发布版本 | GitHub Release 与主分支分开管理；本次不自动创建 Release |
| 当前界面架构 | Rust/WASM + Android WebView；不要恢复 Kotlin 页面或增加 JS 动画控制器 |

## 本次改动交接表

| 需求 | 已实现行为 | 源码入口 | 验证与后续 |
| :-- | :-- | :-- | :-- |
| 设置页参考 Melox | 分组卡片、独立“主题与界面 / 歌词外观”、音质/缓存/账号保留原功能；退出位于设置底部 | `ui/src/presentation.rs`、`render.rs`、`style.css` | 浏览器深浅色、360px窄屏与900px宽屏检查；不要删除已有莫奈子页 |
| 底栏 | 磨砂玻璃、经典分栏、悬浮胶囊；图标文字可隐藏；紧凑开关控制是否自动展开歌曲栏 | `customization.rs` 的 bar/compact/blur/navLabels，`lib.rs::frame` | 紧凑模式使用原进度、队列与播放控制；无重复播放器实例 |
| 播放页外观 | 封面弥散 / 主题色渐变 / 纯色 / 动态流光；三种封面形状；标题对齐；切歌翻转 | customization background/shape/titleAlign/coverFlip、`lib.rs::frame`、`style.css` | 流光最多30Hz，暂停/减少动态时冻结，无变化不写样式；歌词仍使用原vsync；未量化实机GPU开销 |
| 歌曲列表 | 封面显隐、时长显隐；保持标题伸缩且时长不参与挤压 | `render.rs::song_row`、customization artwork/duration | 时长单位为服务的秒数，转换到原 format_time 毫秒参数 |
| 歌词外观 | 对齐、24/28/32/36字号、500/600/700字重、渐变模糊、翻译显隐、当前句放大、播放控件显隐 | customization lyric*、translation；`lib.rs::paint_player` | 隐藏控件保留底部进度；重建后重新测量歌词；逐词动画需要逐词时间数据，未伪造 |
| 动效降级 | 舒缓弹性 / 减少动态；系统关闭动画优先 | customization motion，`lib.rs::update` | 轨迹算法未修改，原60/90/120/144Hz单测继续通过；不宣称实际GPU帧率 |
| 保存配置 | Rust白名单和值校验；立即应用；Android只存储JSON并回传 | `customization.rs::get/set`，`RustWebView.kt` ui_preferences | SharedPreferences `rust-appearance` / `value`；非法旧值回退默认；不清除原账号和歌曲数据 |
| 后续AI接手 | 每次修改更新本表，区分测试、包、推送、Release | 根目录 `AGENTS.md`、本文 | 不在个人记忆目录写入此交接记录 |

## Melox 外观适配边界

参考版本：`97cae45b6acf8a0d1a0a89282d1f7ca02d198ad7`。
设置源文件为 `ThemeSettingsScreen.kt`、`SettingsScreen.kt`、`LyricsInterfacePreferences.kt`。本次依据可见设计和功能独立编写Rust实现，不复制这些GPL文件的代码；原Apache歌词适配声明继续保留。

| 上游功能 | Auralis 对应 / 后续处理 |
| :-- | :-- |
| 主题模式、动态颜色、来源 | 原有系统/浅/深、壁纸/封面/自选色及完整调色板保留 |
| 玻璃 / 普通 / iOS-like 底栏 | 对应磨砂 / 经典 / 悬浮胶囊；WebView `backdrop-filter` 是普通模糊，不声称复现原生液态折射 |
| 紧凑播放器、封面形状、列表封面/时长 | 本次实现 |
| 歌词字号、字重、居中、模糊、翻译 | 本次实现；显示已有翻译，不生成或请求新翻译 |
| 动态流动背景、封面翻转 | 本次以Rust独立实现；不声称与上游像素和物理模型完全一致 |
| 用户自选背景图片 | 未移植；后续须做系统照片选择、缩放缓存和真实文件恢复，不能仅放不可用入口 |
| 状态栏隐藏、预测返回、触觉反馈 | 未移植；涉及系统窗口/返回生命周期，不把普通CSS过渡冒充系统功能 |
| 本地文件名、扫描本地库、语言切换 | 非本次外观范围；在线曲库暂无本地扫描功能，不添加空壳入口 |
| 逐词动画、对唱、罗马音与翻译语言选择 | 当前歌词只有行级LRC与已有翻译；需先扩展真实数据契约再适配 |

## 验证记录

| 检查 | 结果 / 证据 |
| :-- | :-- |
| Rust UI单元测试 | 14通过，含非法设置值防护与原歌词轨迹检查 |
| Rust core单元测试 | 18通过 |
| Java策略测试 | 播放顺序、图片缓存、取色、睡眠计时4组通过 |
| Android构建 | 最终完整 build.ps1 成功；code25；交付APK通过v2签名验证 |
| 真实WASM浏览器 | 通过；设置选择、原生消息回传模拟、紧凑选项、形状、歌词字号/模糊及窄屏无横向溢出；`device-debug/rust-ui/browser/`为忽略的本地产物 |
| 样式规范检查 | frontend-design-styles lint_tokens：0 error / 0 warn；使用现有原生配色映射语义令牌，静态检查不能证明所有莫奈组合的实际对比度 |
| 实机 | 原USB手机9ff29dd3未连接，仅检测到rubens设备/模拟器；没有向其他设备安装；仍需原手机检查外观偏好跨进程恢复、底栏触控与真实歌词跟随 |
| 推送 | 本地完成后推送main；最终交付消息报告提交SHA。未创建GitHub Release，未上传APK到Release |

## 继续工作的命令与约束

1. `git status --short`，读本文和 `AGENTS.md`。
2. `./test.ps1 -Offline`（设置 `JAVA_HOME=C:/Program Files/Java/jdk-21`）。
3. `./build.ps1 -Offline -Aapt2Path <现有缓存中的aapt2.exe>`，SDK在工作区的 `BetterHeybox-NewAPK/android-sdk`，具体见 `docs/BUILD.md`。
4. `PLAYWRIGHT_MODULE=.../career-ops/node_modules/playwright` 后运行 `node tools/check-rust-ui.cjs`。这是实际编译WASM，服务数据为测试夹具。
5. 设备 `9ff29dd3`；包 `dev.qqlyric.app`；安装使用 `adb install -r`。禁止 `pm clear` 或卸载来完成界面测试。
6. 先核对手机是否锁屏。真实账号场景不注入虚构歌曲；测试临时设置后恢复。截图留在忽略的 `device-debug/`，不要上传用户头像、曲库或二维码。
