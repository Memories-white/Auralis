# Auralis · MIUIX 设计记录

## 0.3.7-preview 内容层次调整

保留单一页面宿主、滚动连接、推荐状态与弹层。首页把每日精选放在快捷入口之前，封面扩大至 112dp；快捷入口收为两列辅助操作，最小 48dp 目标。内容书架使用 156dp 完整封面、两行标题及独立歌手摘要，取消图片内缩与整张厚底板。

动态歌曲集合改用 72dp 左右的轻量行，当前播放保留选中底色与播放图标，避免全页每行都像设置卡片。普通内容标题使用 title3，设置分组继续使用 SmallTitle。颜色来自当前 MIUIX 语义色，未加入其他设计体系。

歌曲栏收起高度由 80dp 减为 72dp，进度中心从 66dp 连续移动到展开时的 108dp；展开总高度仍为 200dp。保持原播放、拖动与 4.5 秒收起计时逻辑。当前仅完成构建验证，缺少连接设备，不能将该预览描述为已经通过实机视觉验收。

## 结构和归属

- MainActivity：单个 Compose Activity 和根 MiuixTheme；系统栏图标随最终显示模式变化。
- MusicState：账号、曲库、网络响应代次、播放命令和设置持久化；不把账号复制到 Compose 页面。异步歌单/搜索结果不会覆盖已经离开的页面。
- MusicShell：页面 Scaffold、滚动 TopAppBar、单个 LazyColumn、底部歌曲栏和 NavigationBar。Scaffold 的有效 Insets 只在内容边界消费一次。
- 根 popup Scaffold：登录二维码、退出确认、播放队列的 Overlay 宿主；控制 show 保留组件退出动画。
- MusicPlayer：同一 Activity 内独立的播放内容边界，源位置来自歌曲栏的真实布局坐标。Animatable spring 驱动展开与回收，避免横向 Activity 转场和共享容器层的漏绘。
- HorizontalPager：封面和歌词共用一个页状态；歌词仅在当前页启动跟随动画，布局测量完成后定位。

## 组件和视觉

直接依赖 MIUIX ui/preference/icons 0.9.5-beta01。卡片、按钮、输入、导航、选择项和弹层使用库组件，色彩来自 MiuixTheme.colorScheme，字体来自 textStyles。页面分组使用 SmallTitle + Card，分组外侧横向和底部留白 12dp。封面使用固定几何占位，网络完成后不改变行高；图片不染色。普通操作使用 MiuixIcons，上一首/下一首等库缺失的媒体符号复用项目自有 MusicIcon。

页面底层使用 semantic surface，卡片使用库默认 surfaceContainer。设置保留跟随系统、浅色、深色，无其他风格入口。字体随 Android 系统字体回退，不捆绑未经授权的 MiSans 文件。

## 动效和回退

库负责按压、滚动大标题、过滚回弹和弹层。播放展开使用阻尼 0.86 / 刚度 420 的 spring；歌词滚动 0.7 / 450，强调缩放 0.65 / 450。歌词强调使用图层缩放，不逐帧改字阶；未激活歌词按距离渐进模糊，上下边缘透明遮罩。手动拖动释放后 1.8 秒恢复跟随。

本版顶栏使用不透明 semantic surface 回退：实机静态截图与连续录屏存在局部绘制差异，原因尚未完全确定；为减少叠加图层，停用跨组件背景录制。歌词自身 RenderEffect 模糊不受影响；Android 12 以下不提供实时模糊。未测量帧率，不能以源码或构建成功代替动画性能结论。当前主要针对紧凑手机窗口，尚未完整实现宽屏导航栏切换。

CSS token lint 不适用于实际 Compose 组件树；旧 HTML 样例和 lint 结果只保留为历史参考。当前 API 依据版本契约/源代码和 Android 编译，视觉结果依赖独立实机检查。


## 0.3.1 页面与动效细化

顶部 Search 紧邻头像；底部只保留 Home / Mine。Search、歌单和设置子页统一使用同一返回栈；设置系列页面不显示播放器和底部目的地导航。设置通过 ArrowPreference 进入显示模式、莫奈配色、播放模式、存储和关于子页。

MonetAppearance 解析唯一 ThemeController：动态色来源、色板、规范和显示模式均是 remember 的输入。壁纸刷新通过系统资源和生命周期观察；封面在 IO 线程从 48×48 样本提取加权主色；颜色值保存在私有偏好中，未加载封面时使用系统色回退。支持库内全部 10 种色板。自定义取色只接受 6 位 RGB，非法草稿不会覆盖持久化颜色。

歌词组件独立为 SynchronizedLyrics。自动切句时，先读取当前句在 lazy 布局中的偏移，再将新布局与 LyricFlight 同时提交；每行以最后呈现位置补偿布局变化，独立运行 spring。后续行每隔 38ms 启动，最多 190ms；新的切句继承已呈现的位移和速度，避免等待上一段结束。当前句在位移开始后柔和放大再收稳；浏览时暂时清晰，松手 1.8 秒后恢复跟随。遮罩仅在歌词自身图层，当前句位于中线略上。系统动画时长为零时直接定位，不保留固定错峰等待。

ThinSeekBar 是媒体用途的自定义细轨，因为库默认 Slider 的轨道高度与本次视觉要求不同。外层保留 48dp 触控、拖动预览、抬手提交、TalkBack setProgress 和方向键 ±5 秒。封面显示细轨与小滑块；歌词的边框进度位于控制区末尾。配色仍使用 MIUIX 语义色。

PlaybackOrder 与网络解码分离，覆盖顺序结束、列表/单曲循环、随机遍历及历史。PlayerService 统一处理手动和自动下一首，播放/歌词异步结果带代次检查。PlaybackSnapshot 原子发布，主界面的待播放请求以唯一 token 对齐，不能被上一首的静态字段覆盖。

动效参考 [Apple HIG Motion](https://developer.apple.com/design/human-interface-guidelines/motion)，逐行方案和参数为本应用实现，并非 Apple Music 的公开实现或私有动画参数。


## 0.3.2 音质

QualityChoices 复用于设置子页与根 OverlayDialog；播放页歌曲信息区的 TextButton 显示实际音质，避免把偏好标签误当作已获取的音源。当前未播放时只显示选择入口。偏好与允许降级开关由 MusicState 持久化，自动与标准模式禁用不适用的开关但保留选择值。

Rust quality_candidates 控制所请求的 QQ VKey 文件类型及回退顺序，select_playback 为真实接口与离线测试共用的响应选择器。明确指定 HQ 或标准时不会反向升级；关闭回退时只接受所选档位。返回 quality / requestedQuality / downgraded。

PlayerService 在现有曲目上请求新地址，成功后按回调时的实际位置重建媒体播放器；待 prepare 和 seek 完成后才恢复此前播放状态。失败不释放当前音源；切歌、停止及更晚的音质请求会使旧异步结果失效。切换仅更新同一 MediaSession 的状态，保留歌词与翻译，兼容已有 LyriconProvider。

## 0.3.3 歌词焦点与进度

播放顺序只保留播放器图标，使用语义名称供无障碍读取，非顺序模式使用 primary 强调。设置的播放组保留音质入口。

歌词每行增加水平安全区，右侧保留当前句最大 1.105 倍缩放的空间。BlurredEdgeTreatment.Unbounded 允许模糊向字形边缘外扩，避免默认矩形采样裁掉边缘；视口仍约束整体范围。焦点散焦用 320/480ms 缓动连续改变，不在 selected 变化时直接切换模糊值。保留 FLIP 弹簧和无动画系统偏好支持。

CompactPlaybackProgress 在展开控制的 AnimatedVisibility 之外，歌曲栏收起不再丢失进度。4dp 轨道提供进度语义；歌词 footer 的 8dp 边框轨道保留独立 48dp 交互区域。

## 0.3.4 发现页和连续歌曲栏

参考本地 melodist-reference/docs/screenshots/home.jpg 的内容层次：快捷入口、横向精选入口、推荐书架和歌曲列表。保留已有 MusicShell 的 TopAppBar、滚动连接、导航、状态与弹层宿主；DiscoveryContent 仅贡献 LazyListScope 内容，不再嵌套页面 Scaffold。延续 MIUIX 0.9.5-beta01 组件和明暗语义色，不复制参考项目主题。

搜索输入的 leadingIcon 自带 16dp 起始与 8dp 尾部留白。歌手及历史入口使用单行截断的独立可点击 Card，保持最小 48dp；没有真实推荐数据时提供登录或刷新入口。MusicState 分别拥有歌单 tracks 和 searchResults，查询历史持久化最多 8 条。推荐歌手来自每日曲目元数据，不把本地词条伪装成实时热搜。

MiniPlayer 只有一个 ThinSeekBar 节点。单一 spring 进度协调卡片高度、封面尺寸、紧凑控件宽度/透明度、展开控件透明度和轨道位置；进度位置在紧凑底边与展开信息下方之间移动，轨道从边框样式渐变为带滑块的细轨。展开时使用 48dp 交互区域，拖动暂停收起计时，结束后重置计时。收起时不拦截歌曲信息区点击，进度仍可由无障碍读取。系统动画时长设置沿用 Compose 的 MotionDurationScale。

底栏统一绘制语义 surface 背景。首页和我的继续由 NavigationBar 管理底部 inset，搜索和歌单等二级页改由歌曲栏容器接收 navigationBarsPadding，避免重复应用或遗漏。

技能 CSS 校验器已运行：0 error / 1 warning，警告为缺少 CSS 核心变量。该工具不能解析原生 Compose 的 MiuixTheme 角色，未添加无效 CSS 令牌来消除警告；实际语义色、字阶与组件通过代码检查和实机明暗界面核查。

## 0.3.5 推荐、定位与歌曲面板

每日精选卡与个性化歌曲流分开。Rust 只从具有听歌关联语义的 QQ 歌曲货架提取 ID，再批量请求 TrackInfo，不将歌单卡片 ID 当歌曲 ID。每批排除 daily 的 mid / ID 与已有 mid，加载页向前推进，连续空页停止而不是用每日曲目兜底。推荐使用独立执行器，不占用播放请求的串行工作线程。

MusicShell 仍是唯一列表和滚动宿主。接近尾部触发加载，不在失败时重复自动请求；返回顶部按可见索引显示。当前曲目定位按歌单索引与实际头部项数量计算，详情版本变化时重置该列表。

SongActions 复用根 Overlay 宿主，单一半屏 OverlayBottomSheet 中切换菜单、音质和歌单，保留库的入场退出与嵌套滚动。记录操作时歌曲快照，切歌关闭旧面板，避免向错误歌曲写入。只列出自建且有合法 dirid 的歌单。云端写入只有明确成功或成员读取确认后才报告确认，单纯受理单独显示。

HorizontalPager 默认中间封面页，左侧为曲目信息、右侧为歌词；保留原有播放器展开与歌词动画归属。封面点击区域带可访问操作标签。
