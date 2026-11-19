# 第三方项目与素材说明

[← 返回首页](../README.md)

## 参考与集成

| 项目 | 用途 | 许可 / 说明 |
| :-- | :-- | :-- |
| [Melodist](https://github.com/Viemean/melodist) | QQ 登录、接口调用研究与首页层次参考 | MIT；保留 [许可原文](licenses/Melodist-LICENSE) 及作者署名 |
| [MIUIX 0.9.5-beta01](https://github.com/compose-miuix-ui/miuix/tree/v0.9.5-beta01) | Compose 组件、主题、图标与设计规范 | Apache-2.0；[许可原文](licenses/MIUIX-LICENSE) |
| [LyriconProvider](https://github.com/limczhh/LyriconProvider) | 系统歌词适配协议与消费链路 | 独立项目，本仓库不包含其模块源码或 APK；以其仓库及适配版本说明为准 |

## 依赖

Android 与 Rust 依赖分别声明在 [`android/app/build.gradle.kts`](../android/app/build.gradle.kts) 和 [`core/Cargo.toml`](../core/Cargo.toml)，Rust 的锁定版本见 [`Cargo.lock`](../core/Cargo.lock)。第三方依赖继续受各自许可证约束，根目录 MIT 许可不会替换依赖许可证。

Android 依赖包括 Jetpack Compose、AndroidX、Kotlin 协程与 Material 等；Rust 依赖包括 JNI、serde、serde_json、base64、http 与 ureq 及其传递依赖。正式发行前应基于最终依赖树更新完整的依赖许可清单。

## 音乐服务与界面素材

QQ、QQ 音乐及相关商标属于相应权利人。本项目是非官方播放器，不代表服务方提供支持，也不授权用户获取账号权益以外的内容。

歌曲、歌词、封面与头像等通过账号授权和服务接口获得，其权利不属于项目代码许可证。文档截图中的歌曲封面仅用于说明播放器界面；截图未包含登录凭据、二维码或账号资料。文档横幅 SVG 为本项目制作的仓库展示素材。
