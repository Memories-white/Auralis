# Auralis · AI 工作交接

> 每次改动后更新此表；入口约定已写入根目录 `AGENTS.md`。布局、交互、设置校验和动画以 Rust 为主，Android 只做必要的平台桥接。

## 当前交付 · 0.5.8

| 项目 | 状态 / 入口 |
| :-- | :-- |
| 日期、版本 | 2026-11-27 · 0.5.8-preview · versionCode 35 |
| 源码基线 | d9caacb240e3fad2a340e83291602338abc941fa，已推送的0.5.7 |
| 当前提交 | 本表随本轮代码提交至main并核对远端；使用git log -1获取完整SHA |
| APK | APKs/Auralis-v0.5.8-preview.apk · 16238683字节 |
| SHA256 | AEE9148FF05AF393EED46B3A997414E055969B97A814B278323E450EC72D4B17 |
| 签名与数据 | apksigner v2通过；沿用debug证书a1dab650d1c39a63e21255150995b50e2cac236337e794c33a96e7db292cbfde和包名，不清账号/播放数据 |
| 安装 / 发布 | 本轮未安装手机；用户会后续通知USB连接。不创建Release；APK保留本地忽略目录，源码推送main |

### 本轮交接表：录屏返回重叠

| 用户反馈 / 边界 | 改动与准确入口 | 验证、限制与下一步 |
| :-- | :-- | :-- |
| 对照录屏 | 查看用户55.8秒视频：前段Melox，后段Auralis；原始材料与抽帧仅保留device-debug/video-1127忽略目录 | 录屏Auralis为三栏导航，不能据此认定安装的确切版本；当前四栏源码仍能重现相同背景遮挡，未将问题简单归为旧包 |
| 返回只见封面/渐变，文字按钮被挡 | ui/src/motion.rs::sheet_blend与lib.rs::apply_sheet：背景在p0–0.2交接，mini前景在p0–0.3恢复，末段把dock临时置于播放层上方；全页控件使用p0.1–0.6渐显 | 新tools/check-player-handoff.cjs先在0.5.7真实WASM失败：surface opacity始终1；修复后同一用例通过。尾段采样验证surface<0.7、mini前景恢复、最终播放器卸载/mini opacity1 |
| 两份封面与圆角打架 | lib.rs::measure_morph读取实际mini圆角/容器边界；apply_sheet通过sharing-artwork仅在共享封面飞行期间隐藏真实mini封面；unmount_morph清理 | 保留已有封面资源复用与共享元素；不是额外截图背景叠加。浏览器检查封面交接、反向中断仍通过；手机GPU材质与合成需后续实测 |
| 歌词页返回后马上点开跳回起点 | lib.rs::begin_sheet/open_player：反向运行只依赖弹簧是否未结束，不再依赖有无morph；保持当前进度/速度与层，不重新从0开始 | focused真实WASM覆盖无共享封面的歌词页，比较反向前后的surface transform不变；连续两次系统Back均消费，不穿透退出App |
| 内容回传后姿态失配 | lib.rs::paint_mini/paint_mini_pose重建后恢复mini高度、expanded类、浮动定位偏移和当前sheet pose；关闭时同步重置定位偏移 | 回归在close中连续5次content回传，验证最后无mounted、无shared-artwork、真实封面可见、dock不残留inert、导航opacity1；标准和悬浮展开mini返回分别恢复68/64px |
| 封面/歌词切页不跟手，内容刷新重播CSS动画 | lib.rs::pager/paint_pager/advance_pager与PointerEvent回调；motion.rs::Feel::PAGER；style.css::.player-strip删除CSS transform transition | Rust弹簧与手指共享同一页进度；按下时缓存宽度，逐帧不读布局。CDP真触摸验证未松手进度在1与2之间、取消后回1；水平手势先区分竖向，保留歌词浏览/自动恢复；仅拖向歌词时准备歌词DOM |
| 缩放/方向变化 | resize取消当前pager手势并回到选定页，复用原窗口与路由所有权；关闭阶段不推进横向页动画 | 未增加Kotlin页面或JS动画控制器；原生桥、登录、协议和播放逻辑不改。减少动态即时落位，无残留层 |
| Miuix与Melox依据 | 保留Miuix 0.9.5-beta01主题/inset、现有标准/悬浮尺寸；研究Melox固定bfe7c5dc467b84d7a9df8e63c93208a3e6f61672的PlayerSheetTransition.kt阶段/输入交接 | 该文件未标Apache，因此仅行为研究，Rust交接函数/手势独立实现，不复制GPL渲染代码；不声称原生材质或字体栅格逐像素相同 |
| 测试 / 性能 | Rust UI31/core44、Java4组、完整WASM/NDK/Gradle构建通过；focused回归和原check-rust-ui.cjs通过；lint0错误0警告 | handoff-058浏览器探针68回调，中位16.7ms/P95约16.8ms，ScriptDuration约9ms、LayoutDuration约3.4ms/整个采样窗；只是Chromium证据，不能当作手机帧率或相对性能改进结论 |

### 下一步：实机核对

1. 用户告知USB连接后，先确认安装versionCode35，再保留数据安装本地APK；不清账号/缓存，不重复要求现在连接。
2. 暂停、播放、mini自动展开时分别测试进入/返回；从歌词页连续返回和半途再打开；比较视频里的遮挡是否消失。
3. 检查封面⇄歌词拖动、取消、连续快滑和歌词数据回传时的位置；再记录FrameTimeline/实际呈现帧。浏览器通过不能替代这些检查。
4. 如仍有可见差异，按具体帧调整对应所有权/交接阶段，继续用Rust；保留用户原始录屏于忽略目录，不公开个人截图。

---

## 0.5.7 历史交付

| 项目 | 状态 / 入口 |
| :-- | :-- |
| 日期、版本 | 2026-11-27 · 0.5.7-preview · versionCode 34 |
| 源码基线 | d2a87feaa02df63e3768c7a5d777ed8457c03754，修改前本地main与origin/main一致 |
| 当前提交 | 本文随本轮代码提交并推送main；使用git log -1读取完整SHA，避免自引用 |
| 本地APK | APKs/Auralis-v0.5.7-preview.apk，17494286字节，目录被Git忽略 |
| SHA256 | 16CC38E9D46448B6D753D56725B8EB2F363ED62D49AC9CA71067BD411FA85C60 |
| 签名 | apksigner v2通过，debug证书SHA256 a1dab650d1c39a63e21255150995b50e2cac236337e794c33a96e7db292cbfde，与上版一致 |
| 安装 / Release | 用户会在USB连接后通知；本轮未安装、未发布GitHub Release，源码推送与APK交付分开 |

### 本轮交接表

| 请求 / 问题 | 实现与准确入口 | 验证 / 限制 / 下一步 |
| :-- | :-- | :-- |
| 恢复歌曲只在播放后出现歌词 | MusicState.kt::sample不再每180ms清空；ensureLyrics/retryLyrics使用专用worker，与播放启动解耦；RustWebView发布lyricState | Rust core新增无session/无播放的缓存恢复集成检查；WASM验证暂停时loading→ready、error→retry且无play事件依赖。手机进程冷启动与真实歌词请求需之后实测 |
| 本地歌词缓存 | core/src/lyric_cache.rs与lib.rs::cached_lyrics；MusicState和PlayerService两处自动/恢复路径传私有cacheDir | 单份512KiB、8MiB/64份；校验mid/version与损坏文件，原子写入；淘汰最旧修改文件。测试拒绝空歌词、错误mid、损坏JSON，不保留session，保护无关文件；无歌词结果不负缓存 |
| 懒加载错误显示无歌词 | render.rs::lyrics统一处理四种状态；lib.rs::ensure_lyrics与player共用；retry_lyrics原生桥接 | 新回归抓到原懒加载路径漏传状态，现已修复；请求失败可重试，真正成功且为空才显示暂无歌词 |
| 图2展开歌曲栏裁切 | render.rs::mini改为进度+单排收藏/上一首/播放/下一首/队列；style.css::mini-five；lib.rs::advance_mini移动浮动按钮 | WASM断言五个按钮全部处于卡片边界内；截图mini-expanded-light.png复核。保留4.5秒自动收起及共享封面动画 |
| 图4/图5歌曲栏和底栏 | render.rs::mini/navigation、presentation.rs::screen、lib.rs::paint_progress/dock_index/layout_shell、icons.rs、MusicState/RustWebView路由桥 | 首页/歌曲/音乐库/设置四栏，单色实心图标与选中强调；标准mini68/nav64，悬浮mini64/nav64且带文字；详情仅歌曲栏。mini副标题按播放进度更新当前歌词，无歌词时回退作者；不显示折叠mini进度线 |
| 歌曲页数据边界 | collection根页展示已经同步的喜欢与最近播放；入口仍可由音乐库打开完整云端喜欢 | 没有伪造全网歌曲库或本地扫描，根页当前喜欢数据范围与既有同步一致；完整喜欢仍通过原分页歌单入口 |
| 重做歌单 | presentation.rs::detail、render.rs::playlist-header/playlist-tabs/playlist-artists、style.css::playlist-* | 去掉重复大标题/大卡片，96px封面+单一标题、真实歌曲数、分段按钮、平面歌曲行；700首分页/播放全部/随机/定位回归通过；艺术家只取已加载歌曲，无虚构专辑年份或艺人资料 |
| 歌单标签与后退 | lib.rs::detail_artists/route_key分别保存标签滚动、屏蔽艺术家标签的歌曲自动分页；RustWebView搜索路由，MusicState::navigate/back保留返回歌单请求 | 艺术家点击发搜索事件，原生进入搜索；返回保留原歌单分页元数据。真实网络与系统返回待手机验证 |
| 关于页 | render.rs::slot(about_identity)、lib.rs::paint_header_scroll、style.css::.about-* | 112px图标、35px名称、14px版本与弥散背景；滚动20–80/80–140/140–200px依次淡出。唯一关于入口、5份本地许可、外链/复制与滚动恢复回归通过；使用Auralis品牌，不复制Melox社区/赞助链接 |
| Melox逐词上浮 | ui/src/karaoke.rs::effect/spatial_reveal/eligible，lib.rs::cache_lyric_nodes/sync_lyrics，style.css::karaoke-glyph | 重叠窗口0.68或4/(n+3)，长音强度smoothstep、sin²脉冲、余弦上浮；字形宽度仅挂载时读，逐帧使用缓存。修复已显亮字仍在脉冲时被旧缓存跳过；暂停/seek/跨句/减少动态、真实字形矩阵通过 |
| 参考与许可 | Melox bfe7c5dc467b84d7a9df8e63c93208a3e6f61672 的AboutScreen/MiniPlayer/布局作行为研究，LyricsView.kt明确Apache-2.0的函数移植 | 更新docs/licenses/Melox-lyrics-NOTICE.md与THIRD_PARTY；GPL设置/关于代码独立实现。DOM字形/字体栅格、WebView磨砂与Compose材质不保证像素或GPU输出完全相同 |
| 验证与交付 | test.ps1、check-rust-ui.cjs、check-player-motion.cjs melox-057-final、完整build.ps1、样式令牌lint、APK签名 | UI30/core44/Java4组通过；WASM含暂停歌词、加载/重试、4导航、艺术家标签、5按钮边界；lint0错误0警告。浏览器68回调、间隔中位约16.7ms/P95约16.8ms，不等于手机GPU帧率；设备验证仍待连接 |

### 下一位AI先做

1. 用户通知USB已连接后再检查ADB并保留数据安装上面APK；不要求现在连接，不清数据、不重新登录。
2. 对照用户真实歌曲做“强制停止→重开→保持暂停→进入歌词”，确认缓存与联网缺失路径；检查返回歌单与连续切歌的歌词身份。
3. 录制展开/收起与逐词长音；测手机回调、FrameTimeline/卡顿，再决定是否调整光晕/模糊。不要把浏览器16.7ms描述为手机120Hz。
4. 继续遵守Rust布局/交互/动画规则；Kotlin仅保留现有状态、媒体与平台桥；更新此交接表并区分源码、APK、安装、Release。

---

## 0.5.6 历史交付

| 项目 | 状态 / 入口 |
| :-- | :-- |
| 日期、版本 | 2026-11-27 · `0.5.6-preview` · versionCode `33` |
| 基线 | `c8985590659a56213e716c7ecede55d94c03695e` · 已推送的0.5.5播放器与音效版 |
| 当前提交 | 本文与本次实现同一个提交；使用 `git log -1` 获取完整 SHA，避免文档自引用哈希 |
| 最新本地包 | `APKs/Auralis-v0.5.6-preview.apk` |
| APK SHA256 | FFAAE5D135535F14C3B1CB962FC954E42DC8A9EF5A92B669A31934F57A1BE531 |
| 包体 | 17497958 字节；`apksigner verify` v2通过；debug证书继续保持一致 |
| 仓库 | [Memories-white/Auralis](https://github.com/Memories-white/Auralis) · public |
| 已发布版本 | GitHub Release 与主分支分开管理；本次不自动创建 Release |
| 当前界面架构 | Rust/WASM + Android WebView；不要恢复 Kotlin 页面或增加 JS 动画控制器 |

## 本次改动交接表

| 问题 / 需求 | 实现与入口 | 验证 / 边界 / 下一步 |
| :-- | :-- | :-- |
| 关于重复、设置层级混乱 | `ui/src/presentation.rs::settings/customization_page`：去掉顶部关于卡，只留“关于 Auralis”一行；设置首页按个性化/播放/应用/账号分组；主题配色与界面合并到“主题与外观”，莫奈选项归入主题颜色 | Rust断言关于入口恰好1个、没有settings-intro；音质/音效/定时/缓存/退出继续可用；退出位于最后；原参数保留 |
| 关于页重新设计 | `render.rs::slot(about_identity/licenses)`：使用现有A+播放符号、名称/版本/一句简介、项目/版本/反馈链接、5份本地许可、复制地址；`shell.rs::project_link`和RustWebView原生外链/剪贴板桥接 | 真实WASM验证唯一身份区、5个许可文档、项目URL和复制事件；原生只允许本项目3个GitHub地址，有浏览器失败提示。实机浏览器/剪贴板尚未验证；许可文本是静态打包，不发起网络获取 |
| 底栏不是Miuix | 依据beta01的NavigationBar.kt：普通64px、图标26、文字12、选中加粗，同色内容按选中/未选/按压透明度区分；删除移动胶囊、描边/填充交叉淡入和拖动选页。悬浮栏52高、28图标、48触控项、12内间距，3项宽192 | WASM断言普通64/26/700字重、aria-selected、按钮无系统默认底色、没有nav-pill；尺寸探针检查标准/悬浮几何。旧glass值经schema读取映射normal，其余个人设置不重置 |
| 规范间距与控件状态 | `style.css`：设置卡片16圆角，外侧12/底部12，小标题28/8，偏好正文17/摘要14，去掉彩色前导底块与默认逐行分隔线；Switch按49×28、20滑块、4→25偏移映射；原生语义禁用色桥接，布尔/单选的语义放在操作按钮上 | 依据Miuix beta01 Card/SmallTitle/NavigationBar/Switch声明与Defaults核对；编译和浏览器图像验证分开。MiSans优先，缺少时回退系统/Noto；没有下载或嵌入字体 |
| 顶栏与适配不统一 | `shell.rs`统一宽屏策略：宽≥840或宽≥600且高宽比<1.2；`layout_shell/paint_header_scroll`以同一个主滚动容器驱动大标题收起，宽屏固定紧凑标题；一次性交接状态栏/底栏空间 | 首次设置进入scrollTop=0；滚动后header=80（safeTop24+56）；返回保存的位置误差<3px；窄/宽屏回归通过。CSS通过data-wide使用同一Rust判断，不保留700px分叉 |
| 切页/返回和弹层不像Miuix | Rust `route_spring/advance_shell`驱动进入淡移；子设置页左边缘右滑复用现有back；`overlay_spring`驱动面板和遮罩，头部下拖未过阈值回弹，过阈值退出，保留DOM直到动画结束；退出期间inert并消费返回 | 浏览器检查实际拖动退出、关闭前节点仍连接、最终卸载；原添加歌单稳定弹窗及触屏切页继续通过。路由/返回仍由现有MusicState所有，不创建第二个栈；拖动区域为头部，不抢占列表/歌词竖向滚动 |
| 次要文字和遮罩颜色失真 | `RustWebView.kt::RustApp`原ARGB→CSS转换丢弃alpha，本轮改为RRGGBBAA；增加card-fg/section-fg/divider/secondary/disabled/error/window-dimming等原生语义色角色 | 编译验证实际Miuix 0.9.5-beta01属性；全局主题仍由原有MiuixTheme提供，未新增Kotlin页面。没有用手工浅/深调色板替换原生取色 |
| 首屏被上一页滚动推走 | `paint_content`先应用壳层空间再恢复滚动；移除主容器全局smooth和自动锚定，显式Auto恢复会取消上一页未结束的平滑滚动；返回顶部/定位仍使用原来的显式Smooth | 先前700首歌单滚到底再首次打开设置，等待450ms仍在顶部；进入子页后返回恢复位置。调试用滚动探针已删除 |
| 交付 | 0.5.6/code33，本地包见上表；完整构建及原证书v2签名；交付推送main，不自动创建Release | 用户要求后续USB连接再通知；本轮未安装或测量手机实际GPU帧率。源码、APK、安装和Release状态分开记录 |

## 0.5.6 依据、偏离与验证

| 类型 | 结果 / 说明 |
| :-- | :-- |
| Miuix依据 | 固定 `v0.9.5-beta01` / `6d2554a850b8960e2616c968263ff64beb5ffb03`；已读NavigationBar.kt、Switch.kt、Card.kt、Scaffold.kt及组件契约。Gradle现有依赖正是0.9.5-beta01，没有升级依赖 |
| Melox依据 | 固定 `bfe7c5dc467b84d7a9df8e63c93208a3e6f61672` 的SettingsScreen.kt / AboutScreen.kt；参考分组、单一关于入口与导航关系，未复制GPL界面源码 |
| 目标边界 | 用户要求界面及交互使用Rust，因此保留Rust/WASM+WebView映射，不调用Compose布局组件；SVG语义图标沿用项目矢量。CSS corner-shape可用时用squircle，否则16px圆角回退；WebView背景模糊不等同于原生材质/液态折射。不声称已经原生组件API/runtime一致 |
| 状态所有权 | 路由和后退仍由既有MusicState持有；Rust控制布局、手势、动画和参数schema；原生层只承接主题/窗口/媒体/外链/剪贴板。没有另建Navigator/账号/播放状态 |
| 单元/策略 | Rust UI30、core42、Java4组通过；新增关于唯一性、许可路由、旧glass回退、壳层尺寸/宽屏阈值和固定外链策略 |
| 真实WASM | tools/check-rust-ui.cjs：首次进入顶部、返回滚动恢复、标题折叠、唯一关于、许可展开、外链/复制事件、面板拖动与延迟卸载；原二维码/700首分页/共享封面/逐字动效/音效/缓存回归通过 |
| 几何/性能探针 | tools/check-player-motion.cjs miuix：标准mini68/nav64，悬浮mini68/nav52/宽192；封面页仍0词行DOM且无全屏clip-path。浏览器单次回调/CPU不能作为手机FPS结论 |
| 视觉证据 | 本地忽略目录device-debug/rust-ui/browser/settings-overview-dark.png、settings-dark.png、about-dark.png与wide-dark.png。全部为测试数据，未上传个人截图 |
| 设计令牌 | lint_tokens.py --style miuix通过；此静态检查不能替代原生控件运行验证或覆盖所有实际动态调色板对比度 |
| 下一步实机 | 保留数据安装后核对新主题透明度、明暗/莫奈、标准/悬浮底栏、设置返回位置、面板快速开关与下拉、系统返回/侧滑冲突、外链和复制；实测呈现帧率后再评价性能 |

## 0.5.5 历史改动交接表

| 问题 / 需求 | 实现与入口 | 验证 / 边界 / 下一步 |
| :-- | :-- | :-- |
| 图3的进度条与底部控件 | `ui/src/render.rs::player` / `style.css`：封面和歌词使用同一seek组件，6px中性圆角实心轨道、无边框、无闲置滑块；时间位于下方；进度→上一首/播放/下一首→顺序/定时/音效/队列/更多。歌词上方增加字号与译文入口 | WASM检查轨道6px/边框0、实际元素纵向顺序；封面、歌词共用seek输入与原媒体跳转桥接；参考图按dp和可用高度适配，并非跨字体/分辨率像素截图保证 |
| 只还原逐字亮度，没有空间动效 | `karaoke.rs::pieces/effect`、`render.rs::lyrics`、`lib.rs::sync_lyrics/cache_lyric_nodes`：Rust依据真实词区间的扫亮分数，驱动字形上浮；长音缩放与柔光，再回到正常比例；已唱字在本句保持抬起，离开焦点复位；原整句上浮和远近模糊保留 | 暂停冻结、倒退seek、跨句、原生采样间逐帧推进、无逐字数据回退均通过；字形矩阵测得已唱字上浮、未唱字静止、长音scale>1.02。仅有词时钟，不伪称云端提供逐字符时间；复杂组合/连写文字保留整词排版 |
| 动效开销与用户排版 | 新增glyph_fills缓存，只有揭示分数变化的字形写样式；仅活动行算效果；首次进入歌词才创建节点；减少动态时不位移/缩放/柔光。默认居中24px/500字重，已有自定义设置仍保留 | Rust UI28项通过，包含空间位移/长音/端点/减少动态/组合文字；没有据此声称手机实际GPU帧率达标。长逐字歌词的初次建字节点成本仍需真机采样 |
| 图1音频面板 | `render.rs::overlay(audio-effects)`：倍速滑轨、系统均衡器、可用时MiSound、播放暂停淡入淡出；白色滑块，分组半透明卡片；`RustWebView.kt`桥接和能力查询；Manifest只增加目标intent/package可见性 | 浏览器验证1.25倍事件、开关事件、明暗与360px；均衡器无可用组件时隐藏，未播放时提示先播放；MiSound检测com.miui.misound.HeadsetSettingsActivity的enabled/exported属性，可用时显示。真实系统面板及听感未实测 |
| 倍速与淡入淡出真实接入 | `core/src/playback_controls.rs::audio`：校验0.25–2.0倍/0.05步进，Rust生成320ms音量曲线；`PlayerService.kt`用PlaybackParams及Handler执行曲线，持久化参数。Snapshot/MediaSession/逐词插值使用实际速度；新播放/反向操作/停止取消旧音量任务 | 核心测试验证边界、曲线端点/单调与时长；Android编译通过。参数不支持时保留旧速度或回退原速并显示原因；暂停淡出过程中关闭开关会结束实际播放，避免静音继续跑；实机听感待验证 |
| 图2歌曲操作菜单 | `render.rs::overlay(song-actions)`：封面/歌名/作者/专辑头部，下一首、加入队列、云端喜欢、加入歌单、专辑/艺术家、音质、音效、分享、歌曲信息；78%高度，内部可滚动。跨子页保留同一个sheet根节点 | 不包含本地元数据编辑、隐藏和删除文件；专辑/艺术家目前打开歌曲信息页，不宣称新增在线专辑/艺人目录。WASM验证稳定弹层、加入歌单原流程和下一首/队尾不同事件 |
| 队列与系统分享 | `core::playback_controls::enqueue`决定插入位置，保留当前索引；`RustWebView.kt`取得当前歌曲和最新队列，`PlayerService.QUEUE_EDIT`再次检查当前歌曲并保存恢复快照；分享仅在用户点按钮时打开系统选择器 | Rust覆盖下一首/尾部/非法索引；操作菜单可显示结果。未在手机真实播放中验证重复歌曲、连续改队列及系统分享目标；切歌并发冲突下需重试 |
| 构建和交付 | code32 / 0.5.5；本地包、源码推送与Release分开 | 完整WASM/NDK/Gradle离线构建及APK签名通过；普通沙箱Gradle缓存缺插件，用已有本机缓存构建成功，无依赖升级；源码推送main，不创建Release |

## 0.5.5 验证表

| 检查 | 结果 / 证据 |
| :-- | :-- |
| 单元/策略 | Rust UI28、core42、Java4组通过；音频曲线和队列插入策略位于Rust，原QRC/登录/歌单/Provider链路未替换 |
| 实际WASM | `tools/check-rust-ui.cjs` 通过，新增逐字空间矩阵、进度条几何/顺序、音效参数与状态回传、明暗音效截图和360px溢出；原二维码、700首分页、共享封面反向、触屏切页、封面缓存和稳定弹层回归通过 |
| 视觉/令牌 | 本地忽略目录 `device-debug/rust-ui/browser/karaoke-spatial.png`、`audio-effects-{light,dark}.png`、`song-actions-dark.png` 已复核；为自写歌词和图片夹具。设计令牌lint 0 error / 0 warn |
| 包与签名 | APK哈希/字节见当前交付；v2签名通过，证书SHA256 `a1dab650d1c39a63e21255150995b50e2cac236337e794c33a96e7db292cbfde` 与0.5.4相同 |
| 实机/下一步 | 用户要求后续连接USB再通知，本轮没有安装、真实播放或听感验证。之后保留数据安装，检查真实长音逐字上浮、快进/暂停/倍速同步、淡出反向、音效能力入口、队列插入和实际呈现帧率 |

## 0.5.4 历史改动交接表

| 问题 / 需求 | 实现与入口 | 验证 / 边界 / 下一步 |
| :-- | :-- | :-- |
| 底栏与Melox仍不一致 | 研究Melox当前提交 `bfe7c5dc467b84d7a9df8e63c93208a3e6f61672`；`ui/src/style.css` 玻璃歌曲栏52、导航54、水平16、间隔8、底边max(24,safe+8)，胶囊每侧越过槽位5，图标25/标签11；莫奈关闭用中性前景，开启用主题色 | WASM实测52/54；普通68/64、悬浮64/64且导航宽308，明暗截图复核；Rust独立实现，不复制GPL布局源码。WebView磨砂仍不具备Melox原生液态折射 |
| 底栏缺少跟手反馈 | `lib.rs` 的 `NavDrag`、pointerdown/move/up/cancel、`sync_dock/paint_dock/advance_dock`：按压弹簧、拖动定位、松手导航、取消归位，缓存DOM与槽位宽度；宽屏侧栏保持点击操作 | 真实WASM指针拖到0.6槽时指示器位于0.6，松手请求mine；弹簧微量过冲有上限；resize重算槽位 |
| 展开歌曲栏卡顿 | `SheetLayers` 缓存节点；`apply_sheet` 独立背景surface、内容与共享封面层，移除全屏clip-path与根节点继承的逐帧变量；overlay尺寸只在创建时设置，逐帧改transform/opacity/圆角；打开期间暂停背后mini高度运动 | 封面圆角连续并正确交接；关闭中重新打开复用同一overlay与位置；`motion.rs::SHEET` 使用19rad/s临界阻尼；未宣称完全无重绘或稳定120FPS |
| 打开封面页生成整首歌词DOM | `render.rs::lyrics/player` 与 `lib.rs::ensure_lyrics/cache_lyric_nodes`：封面页不挂载词行，首次进入歌词页才创建，保留逐词节点缓存与真实时间 | 400行夹具打开封面时词行节点400→0；首次进入歌词页正确创建，原暂停/seek/帧间逐词进度回归通过 |
| 高刷新率提示与生命周期 | `RustWebView.kt::animationRate`：动画/播放中歌词时申请API35 View高刷新类别及同分辨率最高窗口刷新率；省电模式请求60；结束或close恢复窗口旧值 | Kotlin/Android编译通过；只改应用窗口提示，系统/WebView/设备策略可覆盖；实机需观察实际呈现帧与停止后的恢复 |
| 更新交付状态 | versionCode31 / 0.5.4；本地APK与源码分开管理 | 完整离线构建、签名通过；交付推送main，不创建Release；用户要求手机后续连接再通知，本轮不安装、不声称手机已顺滑 |

## 0.5.4 验证表

| 检查 | 结果 / 证据 |
| :-- | :-- |
| 单元与策略 | Rust UI27、core40、Java4组通过；60/120Hz相同弹簧轨迹、反向保留速度、轻微过冲约束继续通过；歌词运动另覆盖多帧率 |
| 实际WASM回归 | `tools/check-rust-ui.cjs` 通过：尺寸、胶囊跟手、共享封面连续圆角/交接、关闭中反向、封面页不生成歌词、首次挂载、五次真实触屏切页、歌词跟随恢复、逐词进度、700首分页、二维码、设置、封面复用、窄/宽屏 |
| 性能方法 | 新增 `tools/check-player-motion.cjs`；411×914 Chromium实际WASM，自写400行歌词、暂停曲目，从点击到约1.1秒后读取CDP CPU指标。浏览器回调与CPU并非Android GPU呈现帧，单次样本不是统计基准 |
| 前后CPU样本 | 基线28e7f5d / 本轮：LayoutDuration 12.840→9.681ms，RecalcStyleDuration 21.141→7.709ms，TaskDuration 68.776→46.161ms；RecalcStyleCount 63→41。桌面回调p95均约16.8ms，没有据此声称手机提高到某个帧率 |
| 性能与截图证据 | 本地忽略目录 `device-debug/rust-ui/motion/{baseline,after}.json`、`after-dock-{light,dark}.png`、`after-player.png`；浏览器完整回归在 `device-debug/rust-ui/browser/`；均为测试夹具，不上传个人内容 |
| 样式检查 | frontend-design-styles 的lint_tokens.py：0 error / 0 warn；尺寸与截图另做检查，不能仅靠令牌lint证明还原完整 |
| 实机下一步 | 用户通知USB连接后保留数据安装；比较展开/收起/快速反向、玻璃拖动、120Hz与省电模式、播放歌词与停止刷新提示恢复。原生液态折射仍未实现 |

## 0.5.3 历史改动交接表

| 问题 / 需求 | 实现与入口 | 验证 / 边界 |
| :-- | :-- | :-- |
| 当前整句一起变亮，参考图需要句内随唱渐亮 | `core/src/lib.rs::lyrics` 同一请求读取行级与逐字歌词；有真实逐字数据时生成一致的LRC与wordLyrics | 不用行长或字数猜时间；无逐字数据保留整句跟随 |
| QRC为加密响应 | `core/src/qrc.rs` / `qrc_tables.rs`：MIT来源的自定义DES变体移植为Rust，zlib限长解压，解析XML里的绝对词时间 | 已验证上游独立算法向量；许可和修改说明见docs/licenses/QRC-*；只支持云端hex，不读取本地QMC文件 |
| 队列自动切歌、恢复等路径缺失逐字数据 | `MusicState.kt`、`PlayerService.kt`、`RustWebView.kt`新增wordLyrics字符串桥接；Snapshot末尾追加默认字段；PLAY/LYRIC/自动下一首/恢复读取/停止均传递或清理 | 384KiB桥接上限；Provider/媒体通知仍收到不含词标记的普通LRC |
| 逐词视觉与实时性 | `ui/src/karaoke.rs`校验文本匹配、计算进度；`lib.rs::sync_lyrics`在行索引未改变时仍按播放时钟更新当前词；`render.rs`输出词片段，CSS做柔边文字渐变 | 已唱白、未唱灰；译文较淡；保留居中、逐行上浮与远近模糊；设置字号/字重/对齐继续有效 |
| 性能 | 缓存词DOM节点和上次填充值，只写当前句中进度改变的词，不重建整段歌词 | 时间使用现有毫秒播放时钟+短程插值；暂停不推进，跳转可前进或倒退 |
| 公共协议诊断 | `core/examples/check_word_lyrics.rs`只打印行/词数量，不打印歌词或凭据 | `cargo run --manifest-path core/Cargo.toml --example check_word_lyrics --offline -- 003rJSwm3TechU`：本轮28行/385个词；服务响应可能变化 |

## 0.5.3 验证表

| 检查 | 结果 |
| :-- | :-- |
| 单元/策略 | Rust UI27、core40、Java4组通过；包括算法向量、QRC绝对时间与空格、无效输入、进度倒退和行级回退 |
| 实际协议 | 公共曲目、不带账号：普通/逐字合并请求均code0；实际Rust dispatch到解密/解析链路成功，28行385词；没有保存或上传歌词正文 |
| 浏览器 | 真实WASM+自写词句夹具：已唱/正在唱/未唱的填充不同；暂停冻结；seek复位；跨句；帧间推进；没有逐字时间时不生成词片段；原回归通过 |
| 视觉 | 深色、多行、居中、未唱灰/已唱白与柔边检查截图在忽略目录device-debug/rust-ui/browser/karaoke-progress.png；不冒充真实歌曲截图 |
| 构建与包 | 完整build.ps1 -Offline成功；code30 / 0.5.3-preview；最终包签名检查通过，哈希见上表 |
| 实机 | 用户要求先继续修改，之后连接手机再通知；本轮未安装或验证截图中的歌曲/音频对齐。连接后保留数据安装，重新播放曲目以获取新逐字数据，检查暂停/seek/自动下一首 |
| 发布 | 交付时推送main；本地APK与GitHub Release分开；未创建新Release |

## 0.5.2 历史改动交接表

| 来源 / 差异 | 合并处理 | 验证与后续 |
| :-- | :-- | :-- |
| 本地main比远程领先2个提交 | 先推送571b865（共享封面/底栏）和a457103（歌单分页），远程main从e84bf3b推进到a457103 | 无未提交文件；没有覆盖或重置DeepSeek已提交的本地工作 |
| fix/qq-login-confirmation · a3b9261 | 335b333合并；二维码使用已验证的Base64图片；授权确认后不重新轮询，DNS错误原步骤重试 | 合并CSP同时允许blob封面与data二维码；保留uiPreferences发布和退出清除目录缓存 |
| fix/player-swipe-navigation · 8736099 | 54f99d0合并；嵌套滚动容器统一pan-y，横向手势继续交给Rust | 保留本地封面圆角/底栏回归，加入CDP真实touchStart/touchMove/touchEnd五次往返切页检查 |
| fix/playlist-song-add · e613f94 | 合并添加歌曲权限、类型解析、回执核对、稳定弹窗和推荐分页修复 | 保留advance_sheet闭式弹簧、后台目录快照、700首按需分页与play_all接口，不以旧分支覆盖整文件 |
| 两套分页的兼容 | 主界面继续playlist_page/playlist_all按需加载；分支playlist模块保留legacy全量读取、音乐库与添加后成员核对 | 全量成员核对与所有真实API仍需账号验证；现有播放全部3000首/40页安全界限不变 |
| 弹窗冲突 | Rust Overlay分块更新header/items/message，保留sheet节点；仅首次挂载执行enter，子页切换去掉enter，进度/状态更新不重播动画 | 浏览器检查添加歌单前后sheet同一节点、只列可写歌单、忙碌时禁用重复提交 |
| 分支归属 | 三个修复分支是a3b9261→8736099→e613f94连续链；按顺序合并保留祖先关系 | 不删除远程分支，不强推，不创建Release |

## 0.5.2 验证表

| 检查 | 结果 |
| :-- | :-- |
| 单元与策略 | Rust UI25、core36、Java4组通过；包含授权重试、分页、写入回执、推荐去重和帧率无关弹簧 |
| 构建 | build.ps1 -Offline完整成功（WASM、NDK、Gradle）；code29；debug APK v2签名通过；哈希见上表 |
| 浏览器 | 实际WASM、模拟服务：二维码296px可解码且可见、五次真实触屏切页、700首分页与play_all、sheet节点保留、防重复提交、封面缓存/圆角交接/底栏动效均通过 |
| 量化夹具 | 26种图片每资源最多请求1次；封面共享浮层12帧后落位28px并完成交接；是浏览器回调/夹具，不能作为实机GPU或CDN测量 |
| 实机 | 本轮未安装或重新扫码/云端添加歌曲；分支历史的实机结果不等于合并版验证 |
| 发布 | 本地先上传，最终合并提交再推送main；保留3个修复分支；没有上传APK到GitHub Release |

## 0.5.1 历史改动交接表

| 需求 / 根因 | 修复行为与源码入口 | 验证 / 后续 |
| :-- | :-- | :-- |
| 歌单只显示 100 首 | 根因是 `core/src/lib.rs::playlist` 固定 `song_begin=0 / song_num=100`（disstid）与 `offset=0 / size=100`（dirid），列表数据源本身只有首页。新增 `playlist_page`（单页：`songs/total/hasMore/nextOffset`，total 依次读取 `total_song_num`、`songnum`、`total` 等键）与 `playlist_all`（显式播放全部时按页取完，带 3000 首 / 40 页安全上限并按 mid 去重，防止服务忽略 offset 时死循环） | core 单测覆盖 total 解析、每页剩余数、去重/无进展/上限停止；浏览器检查以 700 首夹具验证头部显示真实总数 |
| 歌词/歌单页要显示"该是多少首就多少首" | Android 侧 `MusicState` 新增 `detailTotal/detailHasMore/detailLoadingMore`，`loadDetail` 首屏取 100 首并记录服务端总数，滚动到底调用 `loadMoreTracks()` 追加；`RustWebView.kt` 发布这些字段并处理 `load_more_tracks`，`counts.tracks` 用服务端总数而不是已加载数量 | 浏览器检查：头部 "700 首"、首屏 100 行、点击/滚动触发 `load_more_tracks`、追加后渲染更多行、取完后不再请求 |
| 播放全部 / 随机播放不应只播已加载的 100 首 | Rust 改发 `play_all`（含 shuffle 标记），Android `playAllTracks()` 在 `detailHasMore` 时先通过 `playlist_all` 取完整曲目再入队；取完失败保留当前列表并显示错误 | 浏览器检查断言 `play_all` 的 shuffle 标记；core 测试覆盖取全逻辑 |
| 我喜欢的音乐同样只有 100 首 | `library` 的 likes 仍只取首页做预览与数量，但 `open_collection` 打开 likes 改为 `playlist(0,201,…)` 走同一分页接口，`counts.likes` 改用 library 返回的真实 total | 无账号时无法实机验证真实 total，逻辑与歌单共用同一分页路径；实机打开喜欢列表需复核 |
| 分批渲染与自动加载 | `render.rs` 的歌曲节点在"已加载未渲染"或"服务端还有更多"时输出本地加载区（显示"已加载 X / Y 首"、加载中状态或继续加载按钮）；`lib.rs::render_pending/scrolled/local_more` 先扩展已取到的行，再在滚动到底时请求下一页，保留滚动位置 | 浏览器检查先渲染 100 行，追加数据后滚动会渲染更多；多歌单/推荐列表仍按 100 行批次扩展，不一次性创建上千 DOM 节点 |
| 范围与边界 | 没有改动协议鉴权、播放地址与缓存策略；分页大小 100 与既有接口一致。服务不返回 total 时以"满页即可能还有"推进，空页或重复页立即停止 | 真实账号下 700+ 歌单的接口页大小、total 键名与限速仍需实机验证；真机未连接 |

## 0.5.1 验证表

| 检查 | 结果 |
| :-- | :-- |
| 构建与签名 | `build.ps1 -Offline` 完整成功（UI WASM + NDK arm64 + Gradle），APK `apksigner verify` v2 通过，`versionCode=28` / `0.5.1-preview` |
| 单元/策略 | Rust UI 25、core 25（新增分页 total/页尾/去重/上限 3 个测试）、Java 4 组通过 |
| 真实 WASM 浏览器 | `node tools/check-rust-ui.cjs` 通过：700 首歌单头部显示总数、首屏 100 行、继续加载触发 `load_more_tracks`、追加后渲染更多、播放全部/随机发 `play_all`、图集仍每资源最多请求 1 次；截图 `device-debug/rust-ui/browser/playlist-long.png` |
| 实机 | `adb devices` 为空；未验证真实账号下歌单分页的接口行为、滚动加载手感与长列表滚动性能 |

## 0.5.0 历史改动交接表

| 需求 / 根因 | 修复行为与源码入口 | 验证 / 后续 |
| :-- | :-- | :-- |
| 从歌词栏 / 歌曲栏打开封面页时圆角突然变大 | 根因是共享封面浮层固定在 `12px`、播放页封面按形状为 `8/28/50%`，动画结束才切换节点。现在由 Rust 逐帧插值面积与圆角：`lib.rs::measure_morph/apply_sheet/begin_sheet`，浮层的 CSS 半径按缩放反算以保持屏幕像素半径一致，`#shared-artwork` 与 `#cover-art` 在同一帧交接；圆形封面同样连续 | 浏览器检查在真实 WASM 中采样中间帧：几何/半径连续、飞行中真实封面透明、结束时浮层移除且落地半径 28px；减少动态时跳过浮层并正确卸载 |
| 底栏动画要对齐 Melox | 新增 Rust 弹簧模块 `ui/src/motion.rs`（闭式解，60/90/120/144Hz 同轨迹，可中途反向），底栏选中胶囊 `--nav-index` 由 `lib.rs::advance_dock/paint_dock` 每帧采样，图标由描边交叉淡入填充，颜色按 `--nav-p` 连续混合；槽位与胶囊几何一致 | 检查脚本断言指示器出现中间值、单调推进、最终与选中项误差 < 2px；广屏侧栏保留静态选中背景 |
| 歌曲栏展开 / 收起要弹性 | 去掉 CSS 高度过渡，`--mini` 弹簧驱动卡片高度与内容显现（`lib.rs::advance_mini`、`style.css` `.mini`/`.mini-expanded`），暂停/切歌短暂展开、紧凑设置保持收起、播放页打开时不再在背后展开 | 展开帧高度 72→206 连续并轻微过冲后回落；检查断言展开类与折叠 |
| 设置页要与 Melox 一致 | 参考 Melox 的分组列表：前导图标、标题/说明、尾部当前值、分隔线、独立子页。`presentation.rs` 新增 `ui-dock`、`ui-player`、`ui-list`、`ui-motion`、`lyric-type`、`lyric-effect`；`customization.rs::radios` 生成单选行；`RustWebView.kt` 的 `SETTINGS_ROUTES` 同步放行 | 浏览器检查覆盖入口子页、选项值经 `ui_preferences` 原生回传后保持、360px 无横向溢出；窄屏截图复核 |
| 封面加载生硬 | 资源层只在字节可用时给 `img[src]`，CSS 用 `.artwork img[data-art-src][src]` 淡入，占位不闪；共享浮层复用已解码节点 | 真实图片请求仍合并为每资源 1 次；淡入只影响表现，不改变缓存与请求策略 |
| 子页切换重播弹层动画 | `lib.rs::paint_overlay` 只在遮罩从关闭变为打开时添加 `enter` 类，音质/加入歌单等子页切换不再从底部重播 | 检查脚本在歌曲操作面板内切换子页后截图与断言 |
| 减少动态 | 弹簧、浮层、流光、歌词位移统一走 `reduced` 分支：直接落位、不挂共享浮层，结束后移除 `mounted` | 新增断言：减少动态下打开/关闭播放页无浮层、封面不透明、播放器正确卸载 |
| 范围与边界 | 未新增播放器功能，未改协议、鉴权与播放链路；没有移植 Melox 的液态折射、逐字歌词或本地扫描。广屏仍是侧栏导航，不做手机端横向分页手势 | 浏览器回调不等于实机 GPU 帧；真机未连接，圆角连续性与弹簧手感仍需实机复核 |

## 0.5.0 验证表

| 检查 | 结果 |
| :-- | :-- |
| 构建与签名 | `build.ps1 -Offline` 完整成功（UI WASM + NDK arm64 + Gradle），最终 APK `apksigner verify` v2 通过，哈希见上表 |
| 单元/策略 | Rust UI 25、core 20、Java 播放顺序/图片缓存/取色/睡眠定时 4 组通过；新增弹簧轨迹、设置子页结构与单选值测试 |
| 真实 WASM 浏览器 | `node tools/check-rust-ui.cjs` 通过：新增底栏指示器连续性与落点、封面共享元素中间帧/圆角/交接、减少动态卸载、设置子页与原生回传、窄屏无溢出；报告在忽略目录 `device-debug/rust-ui/browser/report.json` |
| 视觉预览 | `node tools/preview-redesign.cjs` 输出 `device-debug/rust-ui/preview/`：浅/深色、三种底栏、播放页三页、歌曲栏展开、设置各子页；均为测试夹具 |
| 实机 | 本轮 `adb devices` 为空；未安装、未验证真实设备上的底栏弹簧手感、共享元素在低端 WebView 的合成表现与真实 CDN 封面淡入 |
| GitHub | 交付前推送 main；没有创建 Release 或上传 APK 到 Release |

## 0.4.5 历史改动交接表

| 需求 / 根因 | 修复行为与源码入口 | 验证 / 后续 |
| :-- | :-- | :-- |
| 切页重建图片导致占位闪烁 | `ui/src/artwork.rs::replace` 先保留已解码图片再更新DOM，同一内容不重建；加载中的图片通过统一资源层完成后接入当前页面 | 浏览器检查刷新时同一个图片节点仍存在，切页后封面已加载 |
| 列表/歌曲栏/封面页不同URL | `core/src/artwork.rs::identity` 统一分辨率身份；`ui/src/render.rs::art` 仅生成惰性data-art-src，实际请求和资源归属由Rust控制 | 同一专辑不同尺寸归一、头像不重写的单测；取消原player参数分裂 |
| 当前封面被发现内容拖延 | 当前歌曲使用持久会话元数据先出图并优先排队；歌曲栏/播放器/共享封面引用的Blob不会被普通浏览内容淘汰 | 延迟响应检查当前歌曲是首个图片请求；实际手机排队/磁盘时延仍待验证 |
| 同图并发请求与页面重复下载 | Rust XMLHttpRequest single-flight + Blob URL LRU；新页面直接使用同一Blob；已解码节点额外保留16MiB/64个，压缩Blob池8MiB/160个 | 延迟图片响应夹具检查整轮切页、播放页、状态变化中每个封面资源最多请求1次；这是本地资源请求计数，不冒充真实CDN次数 |
| 有缓存仍等待高清下载 | `ArtworkCache.kt::load` 先检查各分辨率缓存再下载；`loadEncoded` 直接返回原压缩字节，复用旧artwork-v1文件 | 不重新解码再WebP压缩；新增2MiB压缩字节内存池；沿用24MiB/240文件/单项256KiB磁盘限制 |
| WebView禁用缓存 | `RustWebView.kt` 改为LOAD_DEFAULT；图片Cache-Control保留，失败no-store；UI代码/WASM继续no-store | 减少浏览器重新解码；Blob仅用于公开图片；CSP只给img-src增加blob:，网络来源仍是本机虚拟源 |
| 启动首页没有任何旧内容 | `artwork_catalog` Rust白名单和数量限制；`MusicState` 保存/恢复账号隔离的目录，后台刷新成功后替换，不在刷新开始时清空发现歌曲 | 目录不含凭据和播放URL，256KiB上限；退出清除；恢复库在进入音乐库时仍后台刷新；真实跨进程恢复待手机 |
| 范围与边界 | 页面滚动不再依赖临界可见时才发起图片请求；只加载当前已渲染批次，未无限预取所有歌单 | 首次从未缓存的新封面仍需网络；系统清理缓存或LRU淘汰后需重新读取/下载；没有“所有图片永久常驻”的承诺 |

## 0.4.5 验证表

| 检查 | 结果 |
| :-- | :-- |
| 构建与签名 | 完整build.ps1成功，code26，最终APK签名验证通过 |
| 单元/策略 | Rust UI 18、core 20、Java策略4组通过 |
| 真实WASM浏览器 | 104个不同测试封面，每个资源最多请求1次；报告在忽略目录device-debug/rust-ui/browser/report.json |
| 实机 | 本轮ADB设备列表为空，未安装或验证实际设备行为 |

## 0.4.4 历史改动交接表

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
| 玻璃 / 普通 / iOS-like 底栏 | 对应磨砂 / 经典 / 悬浮胶囊；WebView `backdrop-filter` 是普通模糊，不声称复现原生液态折射；选中胶囊为 Rust 弹簧滑动，不是 CSS 过渡 |
| 紧凑播放器、封面形状、列表封面/时长 | 已实现；歌曲栏展开改为弹簧采样 |
| 歌词字号、字重、居中、模糊、翻译 | 已实现；显示已有翻译，不生成或请求新翻译 |
| 动态流动背景、封面翻转 | 以Rust独立实现；不声称与上游像素和物理模型完全一致 |
| 用户自选背景图片 | 未移植；后续须做系统照片选择、缩放缓存和真实文件恢复，不能仅放不可用入口 |
| 状态栏隐藏、预测返回、触觉反馈 | 未移植；涉及系统窗口/返回生命周期，不把普通CSS过渡冒充系统功能 |
| 本地文件名、扫描本地库、语言切换 | 非本次外观范围；在线曲库暂无本地扫描功能，不添加空壳入口 |
| 逐词动画、对唱、罗马音与翻译语言选择 | 当前歌词只有行级LRC与已有翻译；需先扩展真实数据契约再适配 |
| 底部导航横向滑动切页 | 未实现；当前通过点击导航切页，避免与播放页/歌词页手势冲突 |

## 0.4.4 验证记录

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
5. 视觉迭代可运行 `node tools/preview-redesign.cjs`，帧与截图写入忽略目录 `device-debug/rust-ui/preview/`；它不是通过门槛。
6. 设备 `9ff29dd3`；包 `dev.qqlyric.app`；安装使用 `adb install -r`。禁止 `pm clear` 或卸载来完成界面测试。
7. 先核对手机是否锁屏。真实账号场景不注入虚构歌曲；测试临时设置后恢复。截图留在忽略的 `device-debug/`，不要上传用户头像、曲库或二维码。
8. 自定义动效只写在 Rust（`motion.rs` / `lib.rs`）；CSS 只做表现与按压反馈，不要重新引入 JS 动画控制器或 CSS 高度过渡来替代弹簧。
