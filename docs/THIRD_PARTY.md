# 第三方项目与素材说明

[← 返回首页](../README.md)

## 参考与集成

| 项目 | 用途 | 许可 / 说明 |
| :-- | :-- | :-- |
| [Melodist](https://github.com/Viemean/melodist) | 登录 / 接口、首页层次、封面候选与压缩缓存参考 | MIT；保留 [许可原文](licenses/Melodist-LICENSE) 及作者署名 |
| [Melox](https://github.com/Inefy-03/Melox) | 布局研究与歌词运动适配 | 根许可 GPL-3.0；歌词两个文件单独标注 Apache-2.0，适配范围见下文；未复制截图、品牌图标或素材。参考 commit `97cae45b6acf8a0d1a0a89282d1f7ca02d198ad7` |
| [MIUIX 0.9.5-beta01](https://github.com/compose-miuix-ui/miuix/tree/v0.9.5-beta01) | 原生主题、莫奈取色及视觉规范研究 | Apache-2.0；[许可原文](licenses/MIUIX-LICENSE)；0.4 页面与图标由 Rust UI 独立实现 |
| [LyriconProvider](https://github.com/limczhh/LyriconProvider) | 系统歌词适配协议与消费链路 | 独立项目，本仓库不包含其模块源码或 APK；以其仓库及适配版本说明为准 |

## 依赖

0.5.8研究同一固定Melox版本的PlayerSheetTransition.kt：迷你栏/全页背景与内容的阶段交接、连续反向与输入所有权。该文件属于根GPL范围，本轮仅据行为独立实现Rust交接、测量与手势；不复制其Compose记录层或渲染代码。使用既有Miuix主题、窗口与单一路由所有权，不新增原生页面或JS动画控制器。


0.5.7 按用户提供的 Melox 截图更新四栏导航、磨砂歌曲栏、歌单信息区与关于页的视觉层次。普通栏仍使用64px导航；悬浮栏改为保留文字的64px圆角栏。此处以用户参考图覆盖0.5.6对Miuix默认52px悬浮栏的映射，不将自绘界面称为原生控件。关于页保留Auralis品牌和真实项目链接，独立实现，不复制GPL界面源码。

逐词动效另核对 `bfe7c5dc467b84d7a9df8e63c93208a3e6f61672` 的Apache-2.0 `LyricsView.kt`：重叠字符窗口、长音脉冲与发光强度移植到Rust；实际DOM字形边界在挂载时缓存，羽化上浮独立映射。许可与修改范围见[动效声明](licenses/Melox-lyrics-NOTICE.md)。

0.5.6 使用 Miuix 固定提交 `6d2554a850b8960e2616c968263ff64beb5ffb03`（v0.9.5-beta01）的 NavigationBar、Switch、Card、SmallTitle 和 Scaffold 契约核对布局、选中/按压状态及颜色角色；以 Rust/CSS 映射到既有WebView目标，原生MiuixTheme继续提供真实语义配色。SVG图标保留项目实现，CSS squircle缺少支持时采用圆角回退，不宣称它们是Compose原生组件。设置和关于的分组/导航关系参考同一Melox固定版本的SettingsScreen/AboutScreen，未复制其GPL界面实现。

0.5.5 结合用户提供的播放器截图，研究同一Melox提交的 `FullPlayerScreen.kt`、`PlaybackSettingsSheets.kt`、`LyricRenderGeometry.kt` 和 `LyricsView.kt` 的可见效果，并参考 `AudioEffectPreferences.kt` 确认MiSound兼容入口。新增控制区、音效面板、队列策略与字形运动由Rust独立实现，未复制这些文件的新代码；原0.4.1中已注明Apache许可的移植范围不变。逐字的字形位移由真实词区间的揭示进度推导，不声称获取了原生逐字符时间，也没有移植原生文字栅格缓存器或液态玻璃渲染器。

0.5.4 进一步研究 Melox 提交 `bfe7c5dc467b84d7a9df8e63c93208a3e6f61672` 的玻璃底栏、悬浮导航、歌曲栏尺寸和共享封面视觉行为，使用 Rust 状态/弹簧与 CSS 合成图层独立实现；未移植这些 GPL 文件源码。WebView 背景模糊不等同于 Melox 原生液态折射，性能测量也不代表其原生实现的帧率。

0.5.3 的云端QRC解码器按 MIT 项目 [jixunmoe-go/qrc](https://github.com/jixunmoe-go/qrc) 固定提交 `866e996416b0cec7bef648400633f6483c4200d5` 移植到Rust，保留[许可](licenses/QRC-LICENSE)与[修改说明](licenses/QRC-NOTICE.md)。限长解压使用flate2及其Rust backend，依赖版本锁定在core/Cargo.lock。逐字UI和进度逻辑独立使用Rust/CSS实现。

0.4.5 参考 Melodist 的 `MobileCoverCacheResolver.kt`、`AlbumArtImage.kt` 中缓存优先与避免重复淡入的设计。Auralis 的请求合并、Blob资源与DOM节点保留独立使用Rust实现；Android继续承担磁盘、字节响应与已有状态持久化桥接。

0.4.4 的设置和底栏参考同一固定 Melox 提交中的主题设置、设置首页及歌词偏好分组，使用 Rust 独立实现功能和视觉；不复制这些 GPL 文件的源码。玻璃效果使用 WebView 普通背景模糊，不声称复现原生液态折射。适配和未适配项目见 [交接表](HANDOFF.md)。

### 0.4.1 歌词动效移植

Melox 根目录声明 GPL-3.0，但 `LyricRowMotion.kt` 与 `LyricsView.kt` 的文件头单独标注 Apache-2.0。本次仅将这两个文件中的逐行运动及焦点 / 模糊规则对应移植到 Rust，保留[来源与修改声明](licenses/Melox-lyrics-NOTICE.md)及[Apache-2.0 许可](licenses/Melox-lyrics-Apache-2.0)。播放页布局继续独立实现；不引入其他 GPL 文件的代码。

Android 与 Rust 依赖分别声明在 [`android/app/build.gradle.kts`](../android/app/build.gradle.kts)、[`core/Cargo.toml`](../core/Cargo.toml) 和 [`ui/Cargo.toml`](../ui/Cargo.toml)，各 Rust crate 都提供 Cargo.lock。UI 使用 wasm-bindgen / web-sys / js-sys 与 serde_json。第三方依赖继续受各自许可证约束，根目录 MIT 许可不会替换依赖许可证。

Android 依赖包括 Jetpack Compose、AndroidX、Kotlin 协程与 Material 等；Rust 依赖包括 JNI、serde、serde_json、base64、http 与 ureq 及其传递依赖。正式发行前应基于最终依赖树更新完整的依赖许可清单。

## 音乐服务与界面素材

第三方服务名称及相关商标属于相应权利人。本项目是非官方播放器，不代表服务方提供支持，也不授权用户获取账号权益以外的内容。

歌曲、歌词、封面与头像等通过账号授权和服务接口获得，其权利不属于项目代码许可证。文档截图中的歌曲封面仅用于说明播放器界面；截图未包含登录凭据、二维码或账号资料。文档横幅 SVG 为本项目制作的仓库展示素材。
