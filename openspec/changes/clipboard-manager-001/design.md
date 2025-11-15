# 剪切板管理器架构设计

## 系统概述

剪切板管理器将实现为后台系统托盘应用程序，监控系统剪切板变化（文本和图片），将它们存储在带可配置限制的本地 SQLite 数据库中，并提供一个最简化的 React 前端界面用于历史记录访问和搜索。

## 组件架构

### 后端 (Rust)
- **剪切板监控器**: 使用策略模式处理内容类型的后台服务
- **内容处理器**: 可插拔的文本和图片处理策略
- **存储层**: 带大小限制和自动清理的 SQLite 数据库
- **Tauri 命令**: 前端通信的 API 端点
- **系统托盘集成**: 后台运行和托盘菜单
- **配置管理器**: 用户偏好设置存储和验证

### 前端 (React)
- **历史组件**: 最小化列表显示，包含图片缩略图和文本预览
- **搜索组件**: 剪切板历史记录的实时搜索
- **设置组件**: 存储限制、启动行为、快捷键配置
- **系统托盘菜单**: 历史记录和设置的快速访问

## 跨平台考虑

## 简化剪切板处理（无策略模式）

### 直接处理（基于 clipboard-rs）
```rust
use clipboard_rs::{ClipboardContext, Clipboard, ContentFormat};

pub struct SimpleClipboardMonitor {
    ctx: ClipboardContext,
    last_content: Option<String>,
}

impl SimpleClipboardMonitor {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            ctx: ClipboardContext::new()?,
            last_content: None,
        })
    }

    pub fn check_for_changes(&mut self) -> Option<String> {
        match self.ctx.get_text() {
            Ok(content) => {
                // 检查是否有变化
                if Some(&content) != self.last_content.as_ref() {
                    // 检查大文本限制
                    if content.len() <= 1024 * 1024 { // 1MB 限制
                        self.last_content = Some(content.clone());
                        return Some(content);
                    } else {
                        // 显示大文本不支持的通知
                        self.show_large_text_notification();
                    }
                }
                None
            }
            Err(_) => None, // 忽略错误，继续监控
        }
    }

    pub fn set_content(&mut self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.ctx.set_text(content.to_string())?;
        self.last_content = Some(content.to_string());
        Ok(())
    }

    pub fn has_text_content(&self) -> bool {
        self.ctx.has(ContentFormat::Text)
    }

    fn show_large_text_notification(&self) {
        // 使用 Tauri API 显示通知
        // TODO: 实现通知功能
        println!("警告：不支持监控大于1MB的文本内容");
    }
}
```

### 优势
- ✅ 无需复杂的策略模式，直接处理
- ✅ 代码量减少 90%
- ✅ 仅支持文本（符合初期需求）
- ✅ 大文本检测简单直接
- ✅ 易于理解和维护
- ✅ 跨平台兼容性良好（clipboard-rs 支持 Windows/macOS/Linux）

## 跨平台打包配置

### Tauri 构建优化
```toml
# src-tauri/Cargo.toml
[profile.release]
panic = "abort"
codegen-units = 1
lto = true
opt-level = "s"
strip = true
```

### 平台特定构建脚本
```bash
# 构建 Windows 版本
npm run tauri build -- --target x86_64-pc-windows-msvc

# 构建 macOS 版本
npm run tauri build -- --target x86_64-apple-darwin

# 构建 macOS Apple Silicon 版本
npm run tauri build -- --target aarch64-apple-darwin
```

### 依赖优化策略
- **clipboard-rs**: 跨平台，无需条件编译
- **tauri-plugin-global-shortcut**: 跨平台插件，Tauri 处理平台差异
- **serde/serde_json**: 纯 Rust，跨平台
- **编译时优化**: 使用 `lto=true` 和 `strip=true` 减少包大小

### 平台特定考虑

#### Windows（基于 Tauri 文档）
- 使用 `arboard` 跨平台剪切板访问（内部使用 Windows API）
- 使用 `tauri-plugin-global-shortcut` 进行全局热键注册
- 使用 Tauri `TrayIconBuilder` 进行系统托盘集成
- 文件路径使用反斜杠，处理 UNC 路径

#### macOS（基于 Tauri 文档）
- 使用 `arboard` 跨平台剪切板访问（内部使用 NSPasteboard）
- 使用 `tauri-plugin-global-shortcut` 进行全局热键注册（使用 NSHotKey）
- 使用 Tauri `TrayIconBuilder` 进行系统托盘集成（使用 NSStatusItem）
- 文件路径使用正斜杠，处理应用沙盒

### 系统托盘实现（基于 Tauri v2 文档）
```rust
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

// 创建系统托盘
let toggle_item = MenuItemBuilder::with_id("toggle", "显示/隐藏剪切板历史").build(app)?;
let settings_item = MenuItemBuilder::with_id("settings", "设置").build(app)?;
let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;

let menu = MenuBuilder::new(app)
    .items(&[&toggle_item, &settings_item, &quit_item])
    .build()?;

let tray = TrayIconBuilder::new()
    .menu(&menu)
    .on_menu_event(move |app, event| match event.id().as_ref() {
        "toggle" => {
            // 显示/隐藏剪切板历史窗口
            show_clipboard_history(app);
        }
        "settings" => {
            // 显示设置窗口
            show_settings(app);
        }
        "quit" => {
            app.exit(0);
        }
        _ => (),
    })
    .on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            // 左键点击显示历史
            let app = tray.app_handle();
            show_clipboard_history(app);
        }
    })
    .build(app)?;
```

### 全局热键实现（基于 Tauri 文档）
```rust
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

// 注册默认快捷键
let app_handle = app.handle();
app_handle.global_shortcut().register(
    "CmdOrCtrl+Shift+V",
    move || {
        // 切换剪切板历史界面
        show_clipboard_history(&app_handle);
    }
)?;
```

## 简化存储方案（JSON 文件存储）

### 存储结构设计
```rust
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: u64,
    pub content: String,              // 初期仅支持文本
    pub timestamp: u64,              // 使用 UNIX 时间戳
    pub is_favorite: bool,           // 收藏标记
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardData {
    pub items: Vec<ClipboardItem>,
    pub next_id: u64,
    pub settings: AppSettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub max_items: usize,            // 默认: 100
    pub max_size_mb: usize,          // 默认: 50
    pub auto_start: bool,            // 默认: false
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            max_items: 100,
            max_size_mb: 50,
            auto_start: false,
        }
    }
}
```

### 简化存储管理
```rust
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct SimpleStorage {
    file_path: PathBuf,
    data: ClipboardData,
}

impl SimpleStorage {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut path = std::env::current_dir()?;
        path.push("clipboard_data.json");

        let data = if path.exists() {
            let content = fs::read_to_string(&path)?;
            serde_json::from_str(&content)?
        } else {
            ClipboardData {
                items: Vec::new(),
                next_id: 1,
                settings: AppSettings::default(),
            }
        };

        Ok(Self {
            file_path: path,
            data,
        })
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }

    pub fn add_item(&mut self, content: String) -> Result<u64, Box<dyn std::error::Error>> {
        // 检查重复内容
        if let Some(last_item) = self.data.items.last() {
            if last_item.content == content {
                return Ok(last_item.id); // 返回已存在项目的ID
            }
        }

        // 检查大文本 (>1MB)
        if content.len() > 1024 * 1024 {
            return Err("Content too large (>1MB)".into());
        }

        let item = ClipboardItem {
            id: self.data.next_id,
            content,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs(),
            is_favorite: false,
        };

        self.data.items.push(item);
        self.data.next_id += 1;

        // 清理旧项目
        self.cleanup_old_items()?;

        self.save()?;
        Ok(self.data.next_id - 1)
    }

    pub fn get_history(&self, limit: usize) -> &[ClipboardItem] {
        let end = self.data.items.len().min(limit);
        &self.data.items[..end]
    }

    pub fn get_item_by_id(&self, id: u64) -> Option<&ClipboardItem> {
        self.data.items.iter().find(|item| item.id == id)
    }

    pub fn remove_item(&mut self, id: u64) -> Result<bool, Box<dyn std::error::Error>> {
        let original_len = self.data.items.len();
        self.data.items.retain(|item| item.id != id);
        let removed = self.data.items.len() < original_len;

        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    fn cleanup_old_items(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let max_items = self.data.settings.max_items;

        if self.data.items.len() > max_items {
            let remove_count = self.data.items.len() - max_items;
            // 保留收藏的项目
            let mut to_remove = Vec::new();

            for (index, item) in self.data.items.iter().enumerate() {
                if !item.is_favorite && to_remove.len() < remove_count {
                    to_remove.push(index);
                }
            }

            // 从后往前删除，避免索引错位
            for &index in to_remove.iter().rev() {
                self.data.items.remove(index);
            }
        }

        Ok(())
    }
}
```

### 优势
- ✅ 零额外依赖，仅使用标准库 + serde
- ✅ 代码量减少 80%
- ✅ 易于调试（直接查看 JSON 文件）
- ✅ 天然支持 JSON 配置存储
- ✅ 性能足够（100条记录的读写操作毫秒级）

## 简化数据流

1. **剪切板检测**: 后台线程使用 `clipboard` 库定期检查剪切板变化
2. **内容处理**: 直接处理文本内容，过滤 >1MB 的大文本
3. **文件存储**: 通过 JSON 文件存储历史记录和配置
4. **UI 更新**: 通过 Tauri 命令通知前端有新的剪切板项目
5. **用户交互**: 用户从简洁列表中选择项目，支持基本搜索
6. **粘贴操作**: 使用 `clipboard` 库将选定内容复制到系统剪切板

### 核心循环（精简版）
```rust
use std::thread;
use std::time::Duration;

pub fn start_clipboard_monitoring(storage: Arc<Mutex<SimpleStorage>>) {
    thread::spawn(move || {
        let mut monitor = SimpleClipboardMonitor::new().unwrap();

        loop {
            if let Some(content) = monitor.check_for_changes() {
                if let Ok(mut storage) = storage.lock() {
                    if let Ok(_id) = storage.add_item(content) {
                        // 通知前端更新
                        // TODO: 实现 Tauri 事件通知
                    }
                }
            }
            thread::sleep(Duration::from_millis(500)); // 每500ms检查一次
        }
    });
}
```

## 性能考虑

- 剪切板监控防抖，避免过度更新
- 限制历史记录大小（例如：最近 100 条项目，总计 50MB）
- 为历史 UI 实现高效分页
- 使用异步操作进行存储，防止 UI 阻塞

## 安全考虑

- 不加密存储（根据用户要求）
- 清除历史记录功能
- 剪切板访问的应用程序权限
- 敏感内容的内存清理

## 技术依赖（精简版 - 基于精巧应用原则）

### Rust Crates（最小依赖）
- `clipboard`: 轻量级剪切板库（Trust Score: 8.6，比 arboard 更轻量）
- `tauri-plugin-global-shortcut`: Tauri 官方全局热键插件（必要）
- `tauri`: Tauri v2 框架，启用 `tray-icon` 特性
- `serde`: JSON 序列化（用于配置文件存储）

### 避免的依赖
- ~~`arboard`~~ → 使用更轻量的 `clipboard`
- ~~`rusqlite`~~ → 初期使用 JSON 文件存储，更简单
- ~~`chrono`~~ → 使用 Rust 标准库 `std::time::SystemTime`
- ~~复杂图片处理~~ → 初期仅支持文本，后续可扩展

### 跨平台精简依赖配置
```toml
[dependencies]
# Tauri 核心 - 跨平台
tauri = { version = "2.0", features = ["tray-icon"] }
tauri-plugin-global-shortcut = "2.0"

# 剪切板库 - 跨平台，支持 Windows/macOS/Linux
clipboard-rs = "0.5"

# JSON 序列化 - 跨平台
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 平台特定依赖
[target.'cfg(windows)'.dependencies]
# Windows 特定依赖（如果有的话）

[target.'cfg(target_os = "macos")'.dependencies]
# macOS 特定依赖（如果有的话）

[target.'cfg(unix)'.dependencies]
# Linux 特定依赖（如果有的话）
```

### React 库
- `@tauri-apps/api`: Tauri 前端 API
- React hooks 用于状态管理
- CSS 用于样式（无额外 UI 库）