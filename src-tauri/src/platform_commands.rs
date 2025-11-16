use tauri::{AppHandle, Manager};
use crate::platform::{get_platform_adapter, Permission};

/// 获取平台信息
#[tauri::command]
pub fn get_platform_info() -> serde_json::Value {
    let adapter = get_platform_adapter();
    serde_json::json!({
        "platform": adapter.platform_name(),
        "defaultShortcut": adapter.default_shortcut(),
        "shortcutModifier": adapter.shortcut_modifier_name(),
        "supportsTransparency": adapter.supports_transparency(),
        "windowStyle": {
            "transparent": adapter.get_window_style().transparent,
            "decorations": adapter.get_window_style().decorations,
            "skipTaskbar": adapter.get_window_style().skip_taskbar,
            "alwaysOnTop": adapter.get_window_style().always_on_top
        }
    })
}

/// 检查权限状态
#[tauri::command]
pub fn check_permissions() -> Result<Vec<String>, String> {
    let errors = crate::platform::check_permissions_with_user_friendly_errors();
    Ok(errors)
}

/// 请求特定权限
#[tauri::command]
pub fn request_permission(app: AppHandle, permission_type: String) -> Result<String, String> {
    let permission = match permission_type.as_str() {
        "accessibility" => Permission::Accessibility,
        "notification" => Permission::Notification,
        "clipboard" => Permission::Clipboard,
        "global_shortcut" => Permission::GlobalShortcut,
        _ => return Err("未知的权限类型".to_string()),
    };

    let adapter = get_platform_adapter();
    adapter.request_permission(&app, permission)
        .map_err(|e| format!("请求权限失败: {}", e))?;

    Ok("权限请求已处理".to_string())
}

/// 打开系统设置
#[tauri::command]
pub fn open_system_settings(setting_type: String) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let command = match setting_type.as_str() {
            "accessibility" => {
                Command::new("open")
                    .args(&["x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"])
                    .spawn()
            }
            "notifications" => {
                Command::new("open")
                    .args(&["x-apple.systempreferences:com.apple.preference.notifications"])
                    .spawn()
            }
            "security" => {
                Command::new("open")
                    .args(&["x-apple.systempreferences:com.apple.preference.security"])
                    .spawn()
            }
            _ => return Err("未知的设置类型".to_string()),
        };

        match command {
            Ok(_) => Ok("系统设置已打开".to_string()),
            Err(e) => Err(format!("打开系统设置失败: {}", e)),
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows平台通常不需要打开系统设置，直接返回成功
        // 因为我们已经简化了权限检查流程
        match setting_type.as_str() {
            "notifications" => {
                use std::process::Command;
                match Command::new("ms-settings:notifications").spawn() {
                    Ok(_) => Ok("通知设置已打开".to_string()),
                    Err(e) => Err(format!("打开通知设置失败: {}", e)),
                }
            }
            _ => Ok("Windows平台无需特殊设置".to_string()),
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Linux需要根据具体的桌面环境处理
        use std::process::Command;

        // 尝试常见的控制中心
        let commands = vec![
            ("gnome-control-center", &["privacy"]),
            ("unity-control-center", &["privacy"]),
            ("systemsettings5", &["privacy"]),
        ];

        for (cmd, args) in commands {
            if let Ok(_) = Command::new(cmd).args(args).spawn() {
                return Ok("系统设置已打开".to_string());
            }
        }

        Err("无法打开系统设置，请手动打开隐私设置".to_string())
    }
}