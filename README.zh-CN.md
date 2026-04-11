# jxr2uhdr

> **⚠️ 开发中** — 只是个人的 side project，不会持续维护

将 JPEG XR（`.jxr`）HDR 图像转换为 [Ultra HDR](https://developer.android.com/media/platform/hdr-image-format) JPEG 文件。

Ultra HDR 是 Google 推出的一种向下兼容的 JPEG 格式，在标准 SDR 基础图像之上嵌入 HDR 增益图。生成的文件在不支持 HDR 的设备上仍可作为普通 JPEG 显示，而支持 HDR 的显示器则能还原完整的高动态范围。

## 使用场景

NVIDIA 游戏内截图工具会将 HDR 画面保存为 JPEG XR 格式（128bpp RGBA float）。本工具可将这些截图直接转换为 Ultra HDR JPEG，原生支持 Android 14+ 及现代 HDR 显示器。

```
input.jxr  （128bpp RGBA f32，HDR）
    ↓  jxr2uhdr
output.jpg （Ultra HDR JPEG，向下兼容 SDR + HDR gain map）
```

## 安装

```bash
cargo install --path .
```

或仅构建，不安装：

```bash
cargo build --release
# 产物路径：target/release/jxr2uhdr
```

## 用法

```
jxr2uhdr --input <INPUT> --output <OUTPUT> [--quality <QUALITY>]

参数：
  -i, --input <INPUT>      输入 JXR 文件路径
  -o, --output <OUTPUT>    输出 Ultra HDR JPG 文件路径
  -q, --quality <QUALITY>  输出 JPEG 质量（0-100），默认 90
  -h, --help               显示帮助
  -V, --version            显示版本
```

**示例：**

```bash
jxr2uhdr -i screenshot.jxr -o output.jpg
jxr2uhdr -i screenshot.jxr -o output.jpg --quality 95
```

可通过 `RUST_LOG` 环境变量控制日志详细程度：

```bash
RUST_LOG=debug jxr2uhdr -i screenshot.jxr -o output.jpg
```

## 构建

```bash
# 调试构建
cargo build

# Release 构建（推荐正式使用）
cargo build --release
```

## 测试

```bash
cargo test
```

## 许可证

MIT
