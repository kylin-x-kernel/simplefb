# SimpleFB

一个简单的帧缓冲控制台库，用于嵌入式系统。

## 简介

SimpleFB 是一个 `no_std` 的 Rust 库，为嵌入式系统提供帧缓冲控制台功能。它支持可配置的字体大小、ANSI 颜色代码、日志缓冲和图片绘制。

## 依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
simplefb = "0.1"
```

## 使用方法

### 1. 创建帧缓冲配置

首先，创建一个 `FramebufferConfig` 来描述你的帧缓冲设备：

```rust
use simplefb::FramebufferConfig;

let config = FramebufferConfig {
    base_addr: 0x4000_0000,  // 帧缓冲的物理地址
    width: 1920,              // 屏幕宽度（像素）
    height: 1080,             // 屏幕高度（像素）
    font_height: 16,          // 字体高度（像素），设为 0 使用默认 8x8
};
```

### 2. 创建日志缓冲区

创建一个静态缓冲区用于存储日志历史：

```rust
use simplefb::LogBuffer;

static mut LOG_STORAGE: [u8; 4096] = [0; 4096];

let log_buffer = LogBuffer::new(&mut LOG_STORAGE as *mut _);
```

### 3. 创建控制台实例

使用配置和日志缓冲区创建控制台：

```rust
use simplefb::SimpleFbConsole;

let mut console = SimpleFbConsole::new(config, log_buffer);
```

### 4. 写入文本

使用 `write_bytes` 方法向控制台写入文本：

```rust
// 写入普通文本
console.write_bytes(b"Hello, World!\n");

// 使用 ANSI 颜色代码
console.write_bytes(b"\x1b[31mRed text\x1b[0m\n");
console.write_bytes(b"\x1b[32mGreen text\x1b[0m\n");
console.write_bytes(b"\x1b[34mBlue text\x1b[0m\n");
```

### 5. 支持的 ANSI 颜色代码

SimpleFB 支持以下 ANSI SGR 代码：

- `\x1b[0m` - 重置为默认颜色
- `\x1b[30m` - `\x1b[37m` - 前景色（黑、红、绿、黄、蓝、品红、青、白）
- `\x1b[40m` - `\x1b[47m` - 背景色
- `\x1b[90m` - `\x1b[97m` - 亮前景色
- `\x1b[100m` - `\x1b[107m` - 亮背景色

### 6. 设置字体大小和颜色

```rust
// 设置字体高度（像素）
console.set_font_height(20);

// 设置前景色（0x00RRGGBB 格式）
console.set_fg_color(0x00FF0000);  // 红色

// 设置背景色
console.set_bg_color(0x00000000);  // 黑色

// 恢复默认颜色
console.reset_colors();
```

### 7. 清屏

```rust
// 清除整个屏幕
console.clear();
```

### 8. 绘制图片

使用 `picture` 模块可以在屏幕上绘制图片：

```rust
use simplefb::picture::draw_picture;

// 准备图片数据（u32 颜色数组，格式为 0x00RRGGBB）
let picture_data: &[u32] = &[
    0x00FF0000, 0x00FF0000,  // 2x2 红色方块
    0x00FF0000, 0x00FF0000,
];

// 在 (100, 100) 位置绘制 2x2 的图片
draw_picture(&config, 100, 100, 2, 2, picture_data);
```

### 9. 完整示例

```rust
#![no_std]

use simplefb::{FramebufferConfig, LogBuffer, SimpleFbConsole};

static mut LOG_STORAGE: [u8; 4096] = [0; 4096];

fn init_console() -> SimpleFbConsole {
    let config = FramebufferConfig {
        base_addr: 0x4000_0000,
        width: 1920,
        height: 1080,
        font_height: 16,
    };
    
    let log_buffer = LogBuffer::new(&mut LOG_STORAGE as *mut _);
    
    let mut console = SimpleFbConsole::new(config, log_buffer);
    
    // 清屏并显示欢迎信息
    console.clear();
    console.write_bytes(b"SimpleFB Console Initialized\n");
    console.write_bytes(b"\x1b[32mSystem Ready\x1b[0m\n");
    
    console
}
```

## 特性

- **no_std 支持**：适用于嵌入式环境
- **可配置字体大小**：支持任意字体缩放（基于 8x8 基础字体）
- **ANSI 颜色支持**：支持标准和亮色 ANSI 颜色代码
- **日志缓冲**：循环缓冲区保存日志历史
- **图片绘制**：支持绘制 RGB 格式图片
- **自动滚动**：当内容超出屏幕时自动滚动
- **Tab 支持**：Tab 字符自动转换为 4 个空格

## API 文档

### `FramebufferConfig`

```rust
pub struct FramebufferConfig {
    pub base_addr: usize,    // 帧缓冲内存地址
    pub width: usize,        // 屏幕宽度（像素）
    pub height: usize,       // 屏幕高度（像素）
    pub font_height: usize,  // 字体高度（像素）
}
```

### `SimpleFbConsole` 主要方法

- `new(config, log_buffer)` - 创建新的控制台实例
- `write_bytes(&mut self, s: &[u8])` - 写入字节数据
- `clear(&mut self)` - 清屏
- `set_font_height(&mut self, height: usize)` - 设置字体高度
- `set_fg_color(&mut self, color: u32)` - 设置前景色
- `set_bg_color(&mut self, color: u32)` - 设置背景色
- `reset_colors(&mut self)` - 重置为默认颜色

### `LogBuffer` 主要方法

- `new<const N: usize>(storage: *mut [u8; N])` - 创建新的日志缓冲区
- `push(&mut self, byte: u8)` - 添加单个字节
- `push_bytes(&mut self, bytes: &[u8])` - 添加多个字节
- `len(&self)` - 获取缓冲区中的字节数
- `iter(&self)` - 迭代所有字节

## 许可证

Apache-2.0

## 作者

- Debin Luo <luodeb@outlook.com>
- KylinSoft Co., Ltd.
