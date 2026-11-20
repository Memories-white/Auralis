# Auralis / LyriconProvider 联调

日期：2026-11-19。对应关联聊天“适配音乐软件”，目标 `dev.qqlyric.app`，使用独立 Auralis Provider。

## 已确认

- 手机已安装的适配版 Provider 经校验一致，SHA256 `BF94281FBA2185624040F6A83CD795F5980ACA94116A718EED93894649651244`。
- 现有工作区路由、数组及 scope.list 均包含 Auralis；未覆盖其他聊天保留的未提交改动。
- 4 项 AuralisLyricsTest 全部通过：独立翻译轨、缺少翻译、占位/超时翻译、增强逐字标签保留。
- 播放器使用 Framework MediaSession，MEDIA_ID 为真实歌曲 mid；`lyricInfo` 内使用 `lyric` 与 `transLyric`。原文和翻译沿独立字段发布。
- 暂停、seek 完成、切歌与异步歌词到达均重新发布时间戳状态。原子快照和播放请求 token 防止旧歌词覆盖新请求。
- LSPosed 模块原先已启用，System UI 已在作用域；Auralis 未勾选。此次只补选 Auralis，保留其他作用域。
- 重启播放器后日志未能证明注入；经用户明确允许重启手机并解锁后，完成了系统端实测。判定依据为消费者来源设置和实际显示，未把没有日志输出误记为注册成功日志。
- HyperLyric 的“歌词源”保持 Lyricon，未切换到 LyricInfo 或 SuperLyric。超级岛显示 The Race 的歌名、歌手、封面、原文与中文翻译。
- 暂停后 MediaSession 为 PAUSED（93.122 秒），系统歌词按当前消费者策略停止显示。暂停状态拖到 44.813 秒，再恢复播放，超级岛显示对应的 Enjoy the feeling 与中文翻译，确认时间轴重同步。
- 下一首切至 MONTAGEM QUEBRADA / DJ FKU，岛内封面、标题与歌词一起改变，不再保留上一首的歌词。上一首返回 The Race 后暂停，PAUSED 位置为 2.340 秒。
- 结束时保留账户与 LSPosed 作用域，音乐暂停；无需重新安装 Provider，也未修改其他应用的作用域或消费者配色设置。

当前 Rust 请求逐行 LRC（qrc: 0）。本次应用内的 Apple Music 参考动效不代表添加了逐字时间轴；System UI 的逐字能力由实际歌词时间标签决定。

## 消费端

手机安装 HyperLyric，支持通过 Lyricon Provider / Central 接收数据；当前 Provider 内嵌 Central Bridge，需在 System UI 生效。参考官方 [HyperLyric 来源说明](https://github.com/Xposed-Modules-Repo/com.lidesheng.hyperlyric) 与 [Lyricon Provider 文档](https://tomakino.github.io/lyricon/zh-cn/developer/provider/)。


## 本次结论

Auralis 0.3.1 + 当前适配版 LyriconProvider + 手机 HyperLyric 的 Lyricon 来源，已验证原文、翻译、暂停、seek 与切歌。测试覆盖本机当前系统，不能据此承诺其他 ROM、版本或没有翻译的歌曲具备相同显示能力。原代码工作区有其他聊天的未提交改动，此次仅运行对应测试，没有改写 Provider 实现。
