# Auralis 应用图标

[← 返回首页](../README.md)

<img src="assets/app-icon.png" alt="Auralis 蓝色 A 形播放图标" width="192" />

## 设计

白色 A 形曲线呼应 **Auralis**，中央独立的播放符号表达音乐播放。背景从 `#4DA1FF` 过渡到 `#2068E8`，与应用的蓝色视觉语言协调。图形不含小字和复杂细节，便于在桌面、应用列表和系统设置中辨认。

图标为项目原创矢量几何，采用项目 MIT 许可；不包含 QQ 音乐、Apple Music 等其他产品标识。

## Android 资源

| 类型 | 资源 |
| :-- | :-- |
| 自适应图标 | `mipmap-anydpi-v26/ic_launcher*.xml`，背景与前景分层，由桌面决定遮罩 |
| Android 13+ 主题图标 | `mipmap-anydpi-v33/ic_launcher*.xml` 中的 `monochrome` 层 |
| 兼容 PNG | mdpi / hdpi / xhdpi / xxhdpi / xxxhdpi，48 / 72 / 96 / 144 / 192 px，含普通与圆形版本 |
| 媒体通知小图标 | `drawable/ic_stat_auralis.xml`，透明底单色矢量 |
| 文档与预览 | [SVG 源图](assets/app-icon.svg)、[512px PNG](assets/app-icon.png) |

前景按 108×108 的自适应视口设计，主体留在中央安全区域；普通桌面不在前景里预画圆角，避免双重裁切。通知图标使用独立 72×72 视口放大单色主体。

## 重新生成

```powershell
python -m pip install Pillow
python tools/generate_app_icon.py
```

生成器同时输出 Android 矢量、自适应 XML、各密度兼容 PNG 和文档预览。图形路径在脚本顶部定义，PNG 使用同一曲线采样并超采样缩小。PNG 是由矢量设计渲染的兼容资源，不是 AI 位图素材。

资源已提交，普通 Android 构建无需 Python 或 Pillow。改变图形后还应同步文档横幅中的标志，并在实际桌面或系统应用信息页确认遮罩效果。
