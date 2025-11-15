use clipboard_rs::{ClipboardContext, Clipboard, ContentFormat};
use crate::storage::SharedStorage;
use thiserror::Error;
use tauri::Emitter;

#[derive(Error, Debug)]
pub enum ClipboardError {
    #[error("剪切板操作失败: {0}")]
    ClipboardError(String),
    #[error("存储操作失败: {0}")]
    StorageError(String),
    #[error("内容过大")]
    ContentTooLarge,
    #[error("无效操作")]
    InvalidOperation,
}

pub struct SimpleClipboardMonitor {
    ctx: ClipboardContext,
    last_content: Option<String>,
    storage: SharedStorage,
    is_running: bool,
}

type ClipboardResult<T> = Result<T, ClipboardError>;

impl SimpleClipboardMonitor {
    pub fn new(storage: SharedStorage) -> ClipboardResult<Self> {
        Ok(Self {
            ctx: ClipboardContext::new().map_err(|e| ClipboardError::ClipboardError(e.to_string()))?,
            last_content: None,
            storage,
            is_running: false,
        })
    }

    pub fn start_monitoring(&mut self) {
        self.is_running = true;
        dev_log!("剪切板监控已启动");
    }

    pub fn stop_monitoring(&mut self) {
        self.is_running = false;
        dev_log!("剪切板监控已停止");
    }

    pub fn check_for_changes(&mut self) -> Option<String> {
        if !self.is_running {
            return None;
        }

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

    pub fn set_content(&mut self, content: &str) -> ClipboardResult<()> {
        self.ctx.set_text(content.to_string())
            .map_err(|e| ClipboardError::ClipboardError(e.to_string()))?;
        self.last_content = Some(content.to_string());
        Ok(())
    }

    pub fn has_text_content(&self) -> bool {
        self.ctx.has(ContentFormat::Text)
    }

    pub fn process_clipboard_change(&mut self, content: String) -> ClipboardResult<Option<u64>> {
        if let Ok(mut storage) = self.storage.lock() {
            let item_id = storage.add_item(content)
                .map_err(|e| ClipboardError::StorageError(e.to_string()))?;
            dev_log!("剪切板项目已添加: ID {}", item_id);
            Ok(Some(item_id))
        } else {
            Err(ClipboardError::StorageError("无法访问存储".to_string()))
        }
    }

    fn show_large_text_notification(&self) {
        dev_log!("警告：不支持监控大于1MB的文本内容");
        // TODO: 这里可以使用 Tauri API 显示系统通知
    }
}

// 用于后台监控的函数
pub fn start_clipboard_monitoring(storage: SharedStorage) -> ClipboardResult<()> {
    start_clipboard_monitoring_with_events(storage, None)
}

// 用于后台监控的函数，支持事件通知
pub fn start_clipboard_monitoring_with_events(storage: SharedStorage, app_handle: Option<tauri::AppHandle>) -> ClipboardResult<()> {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    static MONITOR_RUNNING: AtomicBool = AtomicBool::new(false);

    // 防止在开发模式下启动多个监控线程
    if MONITOR_RUNNING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        dev_log!("剪切板监控已在运行中，跳过重复启动");
        return Ok(());
    }

    let mut monitor = SimpleClipboardMonitor::new(storage.clone())?;
    monitor.start_monitoring();

    let _storage_clone = storage.clone();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = stop_flag.clone();

    std::thread::spawn(move || {
        // 设置线程清理逻辑
        let thread_id = std::thread::current().id();
        dev_log!("启动剪切板监控线程: {:?}", thread_id);

        loop {
            // 检查是否应该停止
            if stop_flag_clone.load(Ordering::SeqCst) {
                dev_log!("剪切板监控线程收到停止信号，退出");
                MONITOR_RUNNING.store(false, Ordering::SeqCst);
                break;
            }

            if let Some(content) = monitor.check_for_changes() {
                if let Ok(Some(item_id)) = monitor.process_clipboard_change(content.clone()) {
                    // 如果有事件通知，发送到前端
                    if let Some(ref app) = app_handle {
                        use crate::storage::ClipboardItem;

                        // 构建剪切板项目
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();

                        let clipboard_item = ClipboardItem {
                            id: item_id,
                            content: content.clone(),
                            timestamp,
                            is_favorite: false,
                        };

                        // 发送事件到前端
                        let _ = app.emit("clipboard-updated", clipboard_item);
                        dev_log!("已发送剪切板更新事件: {}", content.chars().take(50).collect::<String>());
                    }
                }
            }

            // 使用较短的睡眠时间，但检查停止标志
            for _ in 0..10 {
                std::thread::sleep(std::time::Duration::from_millis(50));
                if stop_flag_clone.load(Ordering::SeqCst) {
                    break;
                }
            }
        }
    });

    dev_log!("剪切板监控已安全启动");
    Ok(())
}