#[macro_use]
mod macros;
mod storage;
mod clipboard;
mod platform;
mod platform_commands;

use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::PhysicalPosition as DpiPhysicalPosition;
use tauri::image::Image;
use tauri::{AppHandle, Emitter, Listener, Manager, Position, State};
use storage::{ClipboardItem, SharedStorage, SimpleStorage};
use platform::{get_platform_adapter, Permission};
use serde_json::json;
use std::collections::HashSet;

// 全局快捷键管理器
#[derive(Clone)]
pub struct ShortcutManager {
    app_handle: AppHandle,
    registered_shortcuts: Arc<Mutex<HashSet<String>>>,
}

impl ShortcutManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            registered_shortcuts: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    // 清理所有可能存在的残留快捷键
    pub fn cleanup_residual_shortcuts(&self) -> Result<(), Box<dyn std::error::Error>> {
        use tauri_plugin_global_shortcut::GlobalShortcutExt;

        // 直接清理所有快捷键，不管是什么
        let _ = self.app_handle.global_shortcut().unregister_all();
        dev_log!("已清理所有残留快捷键");
        Ok(())
    }

    // 注册快捷键
    pub fn register_shortcut(&self, shortcut: &str) -> Result<(), Box<dyn std::error::Error>> {
        use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

        // 检查是否已经注册
        {
            let registered = self.registered_shortcuts.lock().unwrap();
            if registered.contains(shortcut) {
                dev_log!("快捷键已经注册过: {}", shortcut);
                return Ok(());
            }
        }

        // 检查是否已经被系统注册（可能是之前的实例或重启后残留）
        let is_already_registered = self.app_handle.global_shortcut().is_registered(shortcut);

        if is_already_registered {
            dev_log!("快捷键已被系统注册，尝试注销后重新注册: {}", shortcut);
            // 先注销已有的注册
            let _ = self.app_handle.global_shortcut().unregister(shortcut);
        }

        // 注册快捷键事件处理器
        self.app_handle.global_shortcut().on_shortcut(shortcut,
            move |app, shortcut_event, event| {
                // 只处理按键按下事件，忽略释放事件
                if event.state == ShortcutState::Pressed {
                    dev_log!("快捷键被触发: {:?}, 状态: {:?}", shortcut_event, event);
                    handle_app_toggle(app);
                }
            }
        )?;

        // 注册快捷键
        match self.app_handle.global_shortcut().register(shortcut) {
            Ok(_) => {
                let mut registered = self.registered_shortcuts.lock().unwrap();
                registered.insert(shortcut.to_string());
                dev_log!("成功注册快捷键: {}", shortcut);
                Ok(())
            }
            Err(e) => {
                eprintln!("注册快捷键失败: {} - {}", shortcut, e);
                // 检查错误信息，如果是因为已经注册则不视为错误
                let error_msg = e.to_string();
                if error_msg.contains("already registered") || error_msg.contains("HotKey already registered") {
                    dev_log!("快捷键已被占用，但可能是自身实例: {}", shortcut);
                    // 添加到已注册列表，避免重复冲突提示
                    let mut registered = self.registered_shortcuts.lock().unwrap();
                    registered.insert(shortcut.to_string());
                    Ok(())
                } else {
                    Err(format!("快捷键冲突: {}", e).into())
                }
            }
        }
    }

    // 注销快捷键
    pub fn unregister_shortcut(&self, shortcut: &str) -> Result<(), Box<dyn std::error::Error>> {
        use tauri_plugin_global_shortcut::GlobalShortcutExt;

        match self.app_handle.global_shortcut().unregister(shortcut) {
            Ok(_) => {
                let mut registered = self.registered_shortcuts.lock().unwrap();
                registered.remove(shortcut);
                dev_log!("成功注销快捷键: {}", shortcut);
                Ok(())
            }
            Err(e) => {
                eprintln!("注销快捷键失败: {} - {}", shortcut, e);
                Err(format!("注销快捷键失败: {}", e).into())
            }
        }
    }

    // 清理所有快捷键
    pub fn cleanup_all(&self) {
        use tauri_plugin_global_shortcut::GlobalShortcutExt;

        let shortcuts = {
            let registered = self.registered_shortcuts.lock().unwrap();
            registered.clone()
        };

        for shortcut in shortcuts {
            if let Err(e) = self.unregister_shortcut(&shortcut) {
                eprintln!("清理快捷键失败: {}", e);
            }
        }

        // 最后尝试注销所有快捷键
        let _ = self.app_handle.global_shortcut().unregister_all();
        dev_log!("所有快捷键已清理完毕");
    }
}

struct UiState {
    disable_hotkey_toggle: Arc<Mutex<bool>>,
    last_window_move: Arc<Mutex<Option<Instant>>>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            disable_hotkey_toggle: Arc::new(Mutex::new(false)),
            last_window_move: Arc::new(Mutex::new(None)),
        }
    }
}

fn position_window_near_cursor(window: &tauri::WebviewWindow, cursor: DpiPhysicalPosition<f64>) {
    const EDGE_MARGIN: f64 = 8.0;
    const CURSOR_GAP: f64 = 18.0;

    let window_size = match window.outer_size() {
        Ok(size) => size,
        Err(err) => {
            eprintln!("无法获取窗口尺寸: {}", err);
            return;
        }
    };

    let mut min_x = cursor.x - window_size.width as f64;
    let mut min_y = cursor.y - window_size.height as f64;
    let mut max_x = cursor.x;
    let mut max_y = cursor.y;

    if let Ok(Some(monitor)) = window.current_monitor() {
        let origin = monitor.position();
        let size = monitor.size();
        min_x = origin.x as f64 + EDGE_MARGIN;
        min_y = origin.y as f64 + EDGE_MARGIN;
        max_x = origin.x as f64 + size.width as f64 - window_size.width as f64 - EDGE_MARGIN;
        max_y = origin.y as f64 + size.height as f64 - window_size.height as f64 - EDGE_MARGIN;
    }

    if max_x < min_x {
        max_x = min_x;
    }
    if max_y < min_y {
        max_y = min_y;
    }

    let mut target_x = cursor.x - (window_size.width as f64 / 2.0);
    let mut target_y = cursor.y + CURSOR_GAP;

    if target_y > max_y {
        target_y = cursor.y - window_size.height as f64 - CURSOR_GAP;
    }

    target_x = target_x.clamp(min_x, max_x);
    target_y = target_y.clamp(min_y, max_y);

    let position = Position::Physical(DpiPhysicalPosition::new(
        target_x.round() as i32,
        target_y.round() as i32,
    ));

    if let Err(err) = window.set_position(position) {
        eprintln!("设置窗口位置失败: {}", err);
    }
}

fn build_tray_icon_image() -> Image<'static> {
    const SIZE: usize = 32;
    const BYTES_PER_PIXEL: usize = 4;
    const TOTAL: usize = SIZE * SIZE * BYTES_PER_PIXEL;

    let mut pixels = vec![0u8; TOTAL];
    let mut set_pixel = |x: usize, y: usize, rgba: (u8, u8, u8, u8)| {
        if x >= SIZE || y >= SIZE {
            return;
        }
        let idx = (y * SIZE + x) * BYTES_PER_PIXEL;
        pixels[idx] = rgba.0;
        pixels[idx + 1] = rgba.1;
        pixels[idx + 2] = rgba.2;
        pixels[idx + 3] = rgba.3;
    };

    let body_color = (248, 248, 248, 255);
    let border_color = (205, 205, 205, 255);
    let clip_color = (217, 179, 130, 255);
    let clip_highlight = (244, 211, 171, 255);
    let paper_shadow = (230, 230, 230, 255);
    let accent_dark = (139, 167, 255, 255);
    let accent_light = (158, 178, 255, 255);

    for y in 9..28 {
        for x in 7..25 {
            set_pixel(x, y, body_color);
        }
    }

    for x in 7..25 {
        set_pixel(x, 9, border_color);
        set_pixel(x, 27, border_color);
    }
    for y in 9..28 {
        set_pixel(7, y, border_color);
        set_pixel(24, y, border_color);
    }

    for y in 4..9 {
        for x in 9..23 {
            set_pixel(x, y, clip_color);
        }
    }

    for y in 5..7 {
        for x in 11..21 {
            set_pixel(x, y, clip_highlight);
        }
    }

    for x in 10..22 {
        set_pixel(x, 4, border_color);
    }
    for y in 4..9 {
        set_pixel(9, y, border_color);
        set_pixel(22, y, border_color);
    }

    for x in 8..24 {
        set_pixel(x, 28, paper_shadow);
    }

    for x in 10..21 {
        set_pixel(x, 14, accent_dark);
    }
    for x in 10..21 {
        set_pixel(x, 17, accent_light);
    }

    Image::new_owned(pixels, SIZE as u32, SIZE as u32)
}


// 处理应用切换显示/隐藏
fn handle_app_toggle(app: &tauri::AppHandle) {
    if let Some(ui_state) = app.try_state::<UiState>() {
        if let Ok(flag) = ui_state.disable_hotkey_toggle.lock() {
            if *flag {
                dev_log!("当前处于快捷键录制模式，忽略 toggle 热键");
                return;
            }
        }
    }

    let cursor_position = app
        .cursor_position()
        .ok()
        .map(|pos| (pos.x, pos.y));

    if let Some(window) = app.get_webview_window("main") {
        match window.is_visible() {
            Ok(true) => {
                dev_log!("窗口可见，隐藏窗口");
                let _ = window.hide();
            }
            Ok(false) => {
                dev_log!("窗口不可见，显示窗口");

                let app_handle = app.clone();
                let cursor_position = cursor_position;
                tauri::async_runtime::spawn(async move {
                    let _ = app_handle.emit("show-history", ());
                    dev_log!("已发送show-history事件");

                    tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;

                    if let Some(window) = app_handle.get_webview_window("main") {
                        if let Some((x, y)) = cursor_position {
                            position_window_near_cursor(
                                &window,
                                DpiPhysicalPosition::new(x, y),
                            );
                        }
                        if !window.is_visible().unwrap_or(false) {
                            let _ = window.show();
                        }
                        let _ = window.set_focus();
                        dev_log!("窗口已显示并聚焦（历史列表页面）");
                    }
                });
            }
            Err(_) => {
                dev_log!("无法获取窗口状态，显示窗口");
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    } else {
        dev_log!("找不到主窗口");
    }
}


#[tauri::command]
async fn get_clipboard_history(
    storage: State<'_, SharedStorage>,
    limit: Option<usize>,
) -> Result<Vec<ClipboardItem>, String> {
    let storage = storage.lock().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(100);
    Ok(storage.get_history(limit).to_vec())
}

#[tauri::command]
async fn get_all_clipboard_items(
    storage: State<'_, SharedStorage>,
) -> Result<Vec<ClipboardItem>, String> {
    let storage = storage.lock().map_err(|e| e.to_string())?;
    Ok(storage.get_all_items())
}

#[tauri::command]
async fn search_clipboard_items(
    storage: State<'_, SharedStorage>,
    query: String,
) -> Result<Vec<ClipboardItem>, String> {
    let storage = storage.lock().map_err(|e| e.to_string())?;
    let items = storage.search_items(&query);
    Ok(items)
}

#[tauri::command]
async fn copy_to_clipboard(
    content: String,
    storage: State<'_, SharedStorage>,
) -> Result<(), String> {
    use clipboard::SimpleClipboardMonitor;

    let _monitor = SimpleClipboardMonitor::new(storage.inner().clone())
        .map_err(|e| format!("创建剪切板监控器失败: {}", e))?;

    // 注意：这里我们不能直接使用monitor，因为它不是mut的
    // 我们需要创建一个临时的剪切板上下文
    use clipboard_rs::{ClipboardContext, Clipboard};

    let ctx = ClipboardContext::new()
        .map_err(|e| format!("创建剪切板上下文失败: {}", e))?;

    ctx.set_text(content)
        .map_err(|e| format!("设置剪切板内容失败: {}", e))?;

    dev_log!("内容已复制到剪切板");
    Ok(())
}

#[tauri::command]
async fn delete_history_item(
    id: u64,
    storage: State<'_, SharedStorage>,
) -> Result<bool, String> {
    let mut storage = storage.lock().map_err(|e| e.to_string())?;
    storage.remove_item(id).map_err(|e| format!("删除项目失败: {}", e))
}

#[tauri::command]
async fn set_item_favorite(
    id: u64,
    is_favorite: bool,
    storage: State<'_, SharedStorage>,
) -> Result<bool, String> {
    let mut storage = storage.lock().map_err(|e| e.to_string())?;
    storage
        .set_item_favorite(id, is_favorite)
        .map_err(|e| format!("更新置顶状态失败: {}", e))
}

#[tauri::command]
async fn clear_all_history(
    storage: State<'_, SharedStorage>,
) -> Result<(), String> {
    let mut storage = storage.lock().map_err(|e| e.to_string())?;
    storage.clear_all().map_err(|e| format!("清除历史记录失败: {}", e))?;
    dev_log!("所有历史记录已清除");
    Ok(())
}

#[tauri::command]
async fn get_settings(
    storage: State<'_, SharedStorage>,
) -> Result<storage::AppSettings, String> {
    let storage = storage.lock().map_err(|e| e.to_string())?;
    Ok(storage.data.settings.clone())
}

#[tauri::command]
async fn update_settings(
    settings: storage::AppSettings,
    storage: State<'_, SharedStorage>,
) -> Result<(), String> {
    let mut storage = storage.lock().map_err(|e| e.to_string())?;
    storage.data.settings = settings;
    storage.save().map_err(|e| format!("保存设置失败: {}", e))?;
    dev_log!("设置已更新");
    Ok(())
}

#[tauri::command]
async fn update_shortcut(
    shortcut: String,
    storage: State<'_, SharedStorage>,
) -> Result<(), String> {
    let mut storage = storage.lock().map_err(|e| e.to_string())?;
    let shortcut_display = shortcut.clone();
    storage.data.settings.shortcut = shortcut;
    storage.save().map_err(|e| format!("保存快捷键失败: {}", e))?;
    dev_log!("快捷键已更新为: {}", shortcut_display);
    Ok(())
}

#[tauri::command]
async fn update_max_items(
    max_items: usize,
    storage: State<'_, SharedStorage>,
) -> Result<(), String> {
    if max_items == 0 {
        return Err("最大条数必须大于0".into());
    }

    let mut storage = storage.lock().map_err(|e| e.to_string())?;
    storage.data.settings.max_items = max_items;
    storage
        .enforce_item_limit()
        .map_err(|e| format!("应用条数限制失败: {}", e))?;
    storage
        .save()
        .map_err(|e| format!("保存设置失败: {}", e))?;
    dev_log!("最大记录数已更新为 {}", max_items);
    Ok(())
}

#[tauri::command]
async fn set_hotkey_passthrough(
    disabled: bool,
    ui_state: State<'_, UiState>,
) -> Result<(), String> {
    let mut flag = ui_state
        .disable_hotkey_toggle
        .lock()
        .map_err(|e| e.to_string())?;
    *flag = disabled;
    dev_log!(
        "热键切换{}",
        if disabled { "暂时禁用以便录制" } else { "恢复正常" }
    );
    Ok(())
}

#[tauri::command]
async fn hide_window(
    window: tauri::WebviewWindow,
) -> Result<(), String> {
    window.hide().map_err(|e| format!("隐藏窗口失败: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn show_settings(
    app: tauri::AppHandle,
) -> Result<(), String> {
    use tauri::Emitter;
    use tokio::time::{sleep, Duration};

    // Ensure the front-end switches to the settings page before we bring the window forward
    dev_log!("Tray settings menu clicked");

    let _ = app.emit("show-settings", ());
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit("show-settings", ());
    }

    sleep(Duration::from_millis(50)).await;

    if let Some(window) = app.get_webview_window("main") {
        if !window.is_visible().unwrap_or(false) {
            let _ = window.show();
            let _ = window.center();
        }
        let _ = window.set_focus();
    }

    dev_log!("show-settings event emitted");
    Ok(())
}

#[tauri::command]
async fn show_history(
    app: tauri::AppHandle,
) -> Result<(), String> {
    use tauri::Emitter;

    // 发送事件给前端显示历史列表
    dev_log!("托盘显示列表菜单被点击");
    let _ = app.emit("show-history", ());
    dev_log!("已发送show-history事件");
    Ok(())
}

#[tauri::command]
async fn type_text_to_focused_input(text: String) -> Result<(), String> {
    use enigo::{Enigo, Settings};
    use enigo::Keyboard;

    let settings = Settings::default();
    let mut enigo = Enigo::new(&settings).map_err(|e| format!("初始化键盘输入失败: {}", e))?;

    // 键盘输入文本
    enigo.text(&text).map_err(|e| format!("键盘输入失败: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn restart_app(app: tauri::AppHandle) -> Result<(), String> {
    dev_log!("重启应用程序");
    // 在开发模式下，使用进程退出并重启的方式
    // 在生产模式下，Tauri的restart()应该能正常工作
    #[cfg(debug_assertions)]
    {
        // 开发模式：重新启动进程
        use std::process::Command;
        use std::env;

        // 获取当前可执行文件路径
        let current_exe = env::current_exe().map_err(|e| format!("获取可执行文件路径失败: {}", e))?;

        // 启动新进程
        Command::new(&current_exe)
            .args(&env::args().skip(1).collect::<Vec<_>>())
            .spawn()
            .map_err(|e| format!("启动新进程失败: {}", e))?;

        // 退出当前进程
        std::process::exit(0);
    }

    #[cfg(not(debug_assertions))]
    {
        // 生产模式：使用Tauri的重启API
        app.restart();
        Ok(())
    }
}

// 按需检查剪切板变化的命令（开发模式友好）
#[tauri::command]
async fn check_clipboard_changes(storage: State<'_, SharedStorage>) -> Result<Option<ClipboardItem>, String> {
    use clipboard_rs::{ClipboardContext, Clipboard};

    let ctx = ClipboardContext::new()
        .map_err(|e| format!("创建剪切板上下文失败: {}", e))?;

    if let Ok(content) = ctx.get_text() {
        if !content.trim().is_empty() {
            // 检查内容是否已经存在
            if let Ok(mut storage) = storage.lock() {
                let existing_items = storage.get_all_items();

                // 检查是否与最新项目重复
                if let Some(latest) = existing_items.first() {
                    if latest.content == content {
                        return Ok(None); // 内容未变化
                    }
                }

                // 添加新项目，克隆内容避免所有权移动
                let content_clone = content.clone();
                if let Ok(item_id) = storage.add_item(content) {
                    return Ok(Some(ClipboardItem {
                        id: item_id,
                        content: content_clone,
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                        is_favorite: false,
                    }));
                }
            }
        }
    }

    Ok(None)
}

// 启动/停止剪切板监控（仅在开发模式下使用）
#[tauri::command]
async fn toggle_clipboard_monitoring(enable: bool) -> Result<bool, String> {
    use std::sync::atomic::{AtomicBool, Ordering};

    static MONITOR_ENABLED: AtomicBool = AtomicBool::new(false);

    if enable && !MONITOR_ENABLED.load(Ordering::SeqCst) {
        MONITOR_ENABLED.store(true, Ordering::SeqCst);
        dev_log!("剪切板监控已启用（开发模式）");
        return Ok(true);
    } else if !enable && MONITOR_ENABLED.load(Ordering::SeqCst) {
        MONITOR_ENABLED.store(false, Ordering::SeqCst);
        dev_log!("剪切板监控已禁用");
        return Ok(false);
    }

    Ok(MONITOR_ENABLED.load(Ordering::SeqCst))
}

// 获取剪切板数据最后更新时间
#[tauri::command]
async fn get_last_updated(storage: State<'_, SharedStorage>) -> Result<u64, String> {
    let storage = storage.lock().unwrap();
    Ok(storage.get_last_updated())
}

// 检查是否首次启动
#[tauri::command]
async fn check_first_launch(storage: State<'_, SharedStorage>) -> Result<bool, String> {
    let mut storage = storage.lock().map_err(|e| e.to_string())?;
    let is_first = storage.data.is_first_launch;
    if is_first {
        storage.data.is_first_launch = false;
        storage
            .save()
            .map_err(|e| format!("更新首次启动状态失败: {}", e))?;
    }
    Ok(is_first)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 创建共享存储
    let storage = match SimpleStorage::new() {
        Ok(storage) => storage,
        Err(e) => {
            eprintln!("初始化存储失败: {}", e);
            std::process::exit(1);
        }
    };

    let shared_storage = Arc::new(Mutex::new(storage));

    // 使用事件驱动的剪切板监控，避免后台线程与热重载冲突
    dev_log!("剪切板监控切换为事件驱动模式");
    // 暂时不启动后台监控，等应用完全启动后再开启

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .manage(shared_storage)
        .manage(UiState::default())
        .invoke_handler(tauri::generate_handler![
            get_clipboard_history,
            get_all_clipboard_items,
            search_clipboard_items,
            copy_to_clipboard,
            type_text_to_focused_input,
            delete_history_item,
            set_item_favorite,
            clear_all_history,
            get_settings,
            update_settings,
            update_shortcut,
            update_max_items,
            set_hotkey_passthrough,
            hide_window,
            show_settings,
            show_history,
            restart_app,
            check_clipboard_changes,
            toggle_clipboard_monitoring,
            get_last_updated,
            check_first_launch,
            platform_commands::get_platform_info,
            platform_commands::check_permissions,
            platform_commands::request_permission,
            platform_commands::open_system_settings
        ])
        .setup(|app| {
            // 在生产模式下启动后台剪切板监控
            #[cfg(not(debug_assertions))]
            {
                let storage = app.state::<SharedStorage>();
                let app_handle = app.handle().clone();
                if let Err(e) = clipboard::start_clipboard_monitoring_with_events(storage.inner().clone(), Some(app_handle)) {
                    eprintln!("启动剪切板监控失败: {}", e);
                }
            }

            // 注册全局快捷键
            #[cfg(desktop)]
            {
                let app_handle = app.handle();

                // 创建快捷键管理器
                let shortcut_manager = ShortcutManager::new(app_handle.clone());

                // 先清理可能存在的残留快捷键
                if let Err(e) = shortcut_manager.cleanup_residual_shortcuts() {
                    eprintln!("清理残留快捷键失败: {}", e);
                }

                // 从存储中读取用户设置的快捷键
                let user_shortcut = {
                    let storage = app.state::<SharedStorage>();
                    let storage = storage.lock().unwrap();
                    storage.data.settings.shortcut.clone()
                };
                let shortcut_to_register = user_shortcut;

                // 尝试注册快捷键
                match shortcut_manager.register_shortcut(&shortcut_to_register) {
                    Ok(_) => {
                        dev_log!("全局快捷键已注册: {}", shortcut_to_register);
                    }
                    Err(e) => {
                        eprintln!("注册全局快捷键失败: {}, 但应用继续启动", e);

                        // 延迟发送快捷键冲突事件，确保前端已加载完成
                        let app_handle_clone = app_handle.clone();
                        let shortcut_conflict = shortcut_to_register.clone();
                        tauri::async_runtime::spawn(async move {
                            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                            // 发送快捷键冲突事件到前端（不显示窗口，只通过系统托盘通知）
                            let _ = app_handle_clone.emit("shortcut-conflict", json!({
                                "message": format!("快捷键 {} 已被其他程序占用", shortcut_conflict),
                                "suggestion": "请通过系统托盘右键菜单打开设置，修改为其他快捷键组合"
                            }));
                        });
                    }
                }

                // 窗口关闭时不要退出应用（因为需要后台剪切板监控）
                let icon_image = build_tray_icon_image();
                let window = app.get_webview_window("main").unwrap();
                let _ = window.set_icon(icon_image.clone());
                let window_clone = window.clone();
                let move_state = app.state::<UiState>().last_window_move.clone();

                window.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::CloseRequested { .. } => {
                            dev_log!("窗口关闭，但应用继续在后台运行");
                            // 隐藏窗口而不是关闭应用
                            let _ = window_clone.hide();
                        }
                        tauri::WindowEvent::Moved(_) | tauri::WindowEvent::Resized(_) => {
                            if let Ok(mut last_move) = move_state.lock() {
                                *last_move = Some(Instant::now());
                            }
                        }
                        tauri::WindowEvent::Focused(focused) => {
                            if !focused && window_clone.is_visible().unwrap_or(false) {
                                let suppress_hide = move_state
                                    .lock()
                                    .map(|state| {
                                        state
                                            .map(|inst| inst.elapsed() < std::time::Duration::from_millis(350))
                                            .unwrap_or(false)
                                    })
                                    .unwrap_or(false);

                                if suppress_hide {
                                    dev_log!("窗口拖动中，跳过自动隐藏");
                                } else {
                                    dev_log!("窗口失去焦点，自动隐藏");
                                    let _ = window_clone.hide();
                                }
                            }
                        }
                        _ => {}
                    }
                });

                // 重新实现系统托盘功能 - 使用Tauri v2 API
                use tauri::menu::{Menu, MenuItem};
                use tauri::tray::TrayIconBuilder;

                // 创建菜单项
                let show_item = MenuItem::with_id(app, "show", "显示/隐藏", true, None::<&str>)
                    .unwrap();
                let settings_item = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)
                    .unwrap();
                let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
                    .unwrap();

                // 创建托盘菜单
                let tray_menu = Menu::with_items(app, &[
                    &show_item,
                    &tauri::menu::PredefinedMenuItem::separator(app).unwrap(),
                    &settings_item,
                    &tauri::menu::PredefinedMenuItem::separator(app).unwrap(),
                    &quit_item
                ]).unwrap();
                let tray_icon_image = icon_image.clone();



                // 创建托盘图标
                let _tray_icon = TrayIconBuilder::with_id("main-tray")
                    .icon(tray_icon_image)
                    .menu(&tray_menu)
                    .tooltip("剪切板管理器")
                    .on_menu_event(move |app, event| {
                        match event.id().as_ref() {
                            "show" => {
                                // 显示/隐藏主窗口（只控制历史列表）
                                if let Some(window) = app.get_webview_window("main") {
                                    if window.is_visible().unwrap_or(false) {
                                        let _ = window.hide();
                                    } else {
                                        if let Ok(pos) = app.cursor_position() {
                                            position_window_near_cursor(
                                                &window,
                                                DpiPhysicalPosition::new(pos.x, pos.y),
                                            );
                                        } else {
                                            let _ = window.center();
                                        }
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                    }
                                }
                            }
                            "settings" => {
                                let app_handle = app.clone();
                                tauri::async_runtime::spawn(async move {
                                    if let Err(err) = show_settings(app_handle).await {
                                        eprintln!("无法显示设置页面: {}", err);
                                    }
                                });
                            }
                            "quit" => {
                                std::process::exit(0);
                            }
                            _ => {}
                        }
                    })
                    .build(app)
                    .unwrap();

                dev_log!("系统托盘已初始化");

  
                // 监听应用退出事件，确保快捷键被��确清理
                let shortcut_manager_for_cleanup = shortcut_manager.clone();
                app.listen("tauri://close-requested", move |_| {
                    dev_log!("应用即将退出，清理快捷键资源");
                    shortcut_manager_for_cleanup.cleanup_all();
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
