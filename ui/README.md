# Auralis · Rust UI

页面布局和交互代码位于本目录，编译为 WebAssembly，随 Android APK 离线打包。

```text
Rust 页面模型 → Rust DOM 渲染与事件处理 → Android WebView
                    ↓ 操作消息                ↓ 本机资源
              原有 MusicState / PlayerService / ArtworkCache
```

| 文件 | 职责 |
| :-- | :-- |
| `src/presentation.rs` | 页面层次、分组、内容顺序、设置依赖与操作描述 |
| `src/render.rs` | 页面、歌曲栏、封面 / 歌词 / 信息页及弹层的 DOM 构建 |
| `src/lib.rs` | 点击、键盘、左右滑动、滚动位置、底栏胶囊 / 歌曲栏 / 播放器共享元素、歌词弹簧跟随及播放进度 |
| `src/motion.rs` | 闭式解弹簧与插值工具；所有自定义轨迹按 `dt` 采样，60/90/120/144Hz 同一曲线 |
| `src/style.css` | 语义配色、间距、圆角、响应布局及过渡样式 |
| `src/labels.rs` | 复用 Android 字符串资源，格式化与转义外部内容 |
| `host/` | 本机文档入口与绑定启动代码；不包含页面事件实现 |

## 构建与检查

```powershell
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.108 --locked
.\build-ui.ps1
cargo test --manifest-path ui/Cargo.toml --locked
```

完整 APK 使用根目录 `build.ps1`。生成的 WASM、绑定和宿主文档位于 `android/app/src/main/assets/ui/`，不提交二进制构建产物。Gradle 会拒绝缺失或过期的界面资源，防止打出空白界面的 APK。CLI 与依赖版本必须匹配 `Cargo.lock`。

交互检查使用真实编译的 WASM 和模拟的平台桥接：安装 Playwright 及 Chromium 后运行 `node tools/check-rust-ui.cjs`。也可通过 `PLAYWRIGHT_MODULE` 指定已有的 Playwright 模块目录。测试截图与报告写入忽略目录 `device-debug/rust-ui/browser/`。它验证界面行为，不代表外部服务授权或实际音频播放。检查还会采样底栏指示器与共享封面动画的中间帧，验证圆角连续性、减少动态与设置子页往返。视觉迭代可运行 `node tools/preview-redesign.cjs`，产物写入忽略目录 `device-debug/rust-ui/preview/`，它不是通过门槛。

## 运行边界

- 页面不依赖远程网页、脚本或 CDN，需要支持 WebAssembly / ES Modules 的 Android System WebView。
- Android 保留生命周期、窗口 inset、系统栏、IME、MIUIX / 莫奈主题、登录及播放服务；Kotlin 不再编排页面布局。
- UI 只接收必要歌曲、歌单和显示状态；账号 Cookie、音乐密钥与授权音源地址不传入 WebView。
- 封面、头像和二维码经本机资源通道读取。图片复用有界磁盘缓存；远程页面导航与 WebView 直接网络请求被阻止。
- 播放位置使用轻量更新，结构性数据变化才重绘页面；大歌单先显示 100 首，接近底部继续展开，定位当前歌曲可直接展开其所在区域。
- 歌词按毫秒时间轴定位，使用弹簧滚动、放大和渐进模糊。手动浏览结束后约 1.8 秒恢复跟随；系统关闭动画时直接定位。
- 自定义动效全部在 Rust 采样：底栏选中胶囊、歌曲栏展开、播放器容器与共享封面使用同一个帧回调；CSS 只负责表现属性与按压反馈，不要用 CSS 高度过渡或 JS 控制器替代。
- 设置按 Melox 的分组列表组织：`settings` 是入口，`ui-customize` / `lyrics-style` 是分组页，`ui-dock`、`ui-player`、`ui-list`、`ui-motion`、`lyric-type`、`lyric-effect` 是具体选项页；新增页面必须同时加入 `RustWebView.kt` 的 `SETTINGS_ROUTES`。

## 视觉参考

参考 [Melox](https://github.com/Inefy-03/Melox) 的大标题、分组卡片、封面货架、独立歌曲栏与导航结构，检查基线为 `97cae45b6acf8a0d1a0a89282d1f7ca02d198ad7`。页面布局独立实现。0.4.1 将单独标注 Apache-2.0 的歌词行运动与焦点 / 模糊规则对应移植为 Rust，来源与修改说明见 `docs/licenses/Melox-lyrics-NOTICE.md`。没有复制品牌图标、截图或素材，原有协议与播放链路继续使用原实现。
