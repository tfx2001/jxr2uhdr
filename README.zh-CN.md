# jxr2uhdr

将 JPEG XR（`.jxr`）HDR 图像转换为 [Ultra HDR](https://developer.android.com/media/platform/hdr-image-format) JPEG 文件。

🌐 **在线体验：** [https://tfx2001.github.io/jxr2uhdr/](https://tfx2001.github.io/jxr2uhdr/) — 基于 WebAssembly 的浏览器端转换器，无需安装任何软件。

Ultra HDR 是 Google 推出的一种向下兼容的 JPEG 格式，在标准 SDR 基础图像之上嵌入 HDR 增益图。生成的文件在不支持 HDR 的设备上仍可作为普通 JPEG 显示，而支持 HDR 的显示器则能还原完整的高动态范围。

## 使用场景

NVIDIA 游戏内截图工具会将 HDR 画面保存为 JPEG XR 格式（128bpp RGBA float）。本工具可将这些截图直接转换为 Ultra HDR JPEG，原生支持 Android 14+ 及现代 HDR 显示器。

当前转换 pipeline：

```mermaid
flowchart LR
    input["input.jxr<br/>JPEG XR HDR<br/>128bpp RGBA f32 或 64bpp RGBA f16"]
    decode["解码 JPEG XR<br/>jpegxr::ImageDecode"]
    image["线性 RGBA 图像缓冲区"]
    sdr["SDR 基础图分支"]
    tonemap["Hable filmic tone mapping<br/>linear RGB 转 sRGB RGBA8888"]
    sdr_raw["SDR RawImage<br/>BT.709 / sRGB / full range"]
    hdr["HDR intent 分支"]
    scale["将 scRGB 80-nit 参考白<br/>缩放到 Ultra HDR 203-nit 线性白"]
    hdr_raw["HDR RawImage<br/>RGBA f16 / linear / full range"]
    peak["估算目标显示峰值亮度<br/>基于 HDR RGB 峰值百分位"]
    encoder["libultrahdr Encoder"]
    output["output.jpg<br/>Ultra HDR JPEG<br/>SDR base + HDR gain map"]

    input --> decode --> image
    image --> sdr --> tonemap --> sdr_raw --> encoder
    image --> hdr --> scale --> hdr_raw --> encoder
    image --> peak --> encoder
    encoder --> output
```

## 安装

```bash
cargo install --path jxr2uhdr-cli
```

或仅构建，不安装：

```bash
cargo build --release -p jxr2uhdr
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
cargo build -p jxr2uhdr

# Release 构建（推荐正式使用）
cargo build --release -p jxr2uhdr
```

## 测试

```bash
cargo test --workspace
```

## 许可证

MIT
