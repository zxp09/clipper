use tauri::AppHandle;

/// 平台特定权限状态
#[derive(Debug, Clone)]
pub enum PermissionStatus {
    Granted,
    Denied,
    Unknown,
    NotRequired,
}

/// 平台特定权限类型
#[derive(Debug, Clone)]
pub enum Permission {
    Clipboard,
    Accessibility,
    Notification,
    GlobalShortcut,
}

/// 平台适配器trait - 定义不同平台的接口
pub trait PlatformAdapter {
    /// 获取平台默认快捷键
    fn default_shortcut(&self) -> String;

    /// 获取平台快捷键修饰键说明
    fn shortcut_modifier_name(&self) -> &'static str;

    /// 检查平台特定权限
    fn check_permission(&self, permission: Permission) -> PermissionStatus;

    /// 请求平台特定权限
    fn request_permission(&self, app: &AppHandle, permission: Permission) -> Result<(), String>;

    /// 显示原生通知
    fn show_notification(&self, title: &str, body: &str) -> Result<(), String>;

    /// 获取平台名称
    fn platform_name(&self) -> &'static str;

    /// 检查是否支持透明窗口
    fn supports_transparency(&self) -> bool;

    /// 获取推荐窗口样式
    fn get_window_style(&self) -> WindowStyle;
}

/// 窗口样式配置
#[derive(Debug, Clone)]
pub struct WindowStyle {
    pub transparent: bool,
    pub decorations: bool,
    pub skip_taskbar: bool,
    pub always_on_top: bool,
}

/// Windows平台实现
pub struct WindowsPlatform;

impl WindowsPlatform {
    pub fn new() -> Self {
        Self
    }
}

impl PlatformAdapter for WindowsPlatform {
    fn default_shortcut(&self) -> String {
        "Alt+2".to_string()
    }

    fn shortcut_modifier_name(&self) -> &'static str {
        "Alt"
    }

    fn check_permission(&self, permission: Permission) -> PermissionStatus {
        match permission {
            Permission::Clipboard => PermissionStatus::NotRequired,
            Permission::GlobalShortcut => PermissionStatus::NotRequired,
            Permission::Notification => PermissionStatus::Unknown, // 需要运行时检查
            Permission::Accessibility => PermissionStatus::NotRequired,
        }
    }

    fn request_permission(&self, _app: &AppHandle, permission: Permission) -> Result<(), String> {
        match permission {
            Permission::Notification => {
                // Windows通常不需要显式权限请求
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn show_notification(&self, _title: &str, _body: &str) -> Result<(), String> {
        // 使用Windows通知API
        #[cfg(target_os = "windows")]
        {
            // 这里会在后续集成通知插件
            Ok(())
        }
        #[cfg(not(target_os = "windows"))]
        Ok(())
    }

    fn platform_name(&self) -> &'static str {
        "Windows"
    }

    fn supports_transparency(&self) -> bool {
        true
    }

    fn get_window_style(&self) -> WindowStyle {
        WindowStyle {
            transparent: true,
            decorations: false,
            skip_taskbar: true,
            always_on_top: true,
        }
    }
}

/// macOS平台实现
pub struct MacOSPlatform;

impl MacOSPlatform {
    pub fn new() -> Self {
        Self
    }
}

impl PlatformAdapter for MacOSPlatform {
    fn default_shortcut(&self) -> String {
        "Cmd+Shift+V".to_string()
    }

    fn shortcut_modifier_name(&self) -> &'static str {
        "Cmd⌘"
    }

    fn check_permission(&self, permission: Permission) -> PermissionStatus {
        match permission {
            Permission::Clipboard => PermissionStatus::NotRequired,
            Permission::GlobalShortcut => PermissionStatus::Unknown, // 需要检查辅助功能权限
            Permission::Notification => {
                // macOS 10.14+ 需要通知权限
                #[cfg(target_os = "macos")]
                {
                    // 这里可以添加实际的权限检查逻辑
                    // 目前返回Unknown，表示需要运行时检查
                    PermissionStatus::Unknown
                }
                #[cfg(not(target_os = "macos"))]
                PermissionStatus::NotRequired
            }
            Permission::Accessibility => {
                // macOS需要辅助功能权限用于全局快捷键
                #[cfg(target_os = "macos")]
                {
                    // 这里可以添加实际的辅助功能权限检查
                    PermissionStatus::Unknown
                }
                #[cfg(not(target_os = "macos"))]
                PermissionStatus::NotRequired
            }
        }
    }

    fn request_permission(&self, _app: &AppHandle, permission: Permission) -> Result<(), String> {
        match permission {
            Permission::Accessibility => {
                // macOS需要辅助功能权限用于全局快捷键
                #[cfg(target_os = "macos")]
                {
                    let message = format!(
                        "需要启用辅助功能权限：\n\
                        1. 打开系统偏好设置\n\
                        2. 进入「安全性与隐私」\n\
                        3. 选择「隐私」标签\n\
                        4. 找到「辅助功能」并勾选 {}\n\
                        5. 重启应用以使权限生效",
                        std::env::current_exe()
                            .unwrap_or_else(|_| "应用程序".into())
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                    );
                    Err(message)
                }
                #[cfg(not(target_os = "macos"))]
                Ok(())
            }
            Permission::Notification => {
                // macOS 10.14+ 需要通知权限
                #[cfg(target_os = "macos")]
                {
                    let message = format!(
                        "需要启用通知权限：\n\
                        1. 打开系统偏好设置\n\
                        2. 进入「通知」\n\
                        3. 在左侧找到 {}\n\
                        4. 允许发送通知",
                        std::env::current_exe()
                            .unwrap_or_else(|_| "应用程序".into())
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                    );
                    Err(message)
                }
                #[cfg(not(target_os = "macos"))]
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn show_notification(&self, _title: &str, _body: &str) -> Result<(), String> {
        // 使用macOS原生通知
        #[cfg(target_os = "macos")]
        {
            // 这里会集成macOS特定通知实现
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        Ok(())
    }

    fn platform_name(&self) -> &'static str {
        "macOS"
    }

    fn supports_transparency(&self) -> bool {
        true
    }

    fn get_window_style(&self) -> WindowStyle {
        WindowStyle {
            transparent: true,
            decorations: false,
            skip_taskbar: false, // macOS没有skip taskbar概念
            always_on_top: true,
        }
    }
}

/// Linux平台实现
pub struct LinuxPlatform;

impl LinuxPlatform {
    pub fn new() -> Self {
        Self
    }
}

impl PlatformAdapter for LinuxPlatform {
    fn default_shortcut(&self) -> String {
        "Alt+2".to_string()
    }

    fn shortcut_modifier_name(&self) -> &'static str {
        "Alt"
    }

    fn check_permission(&self, permission: Permission) -> PermissionStatus {
        match permission {
            Permission::Clipboard => PermissionStatus::NotRequired,
            Permission::GlobalShortcut => PermissionStatus::Unknown, // 依赖桌面环境
            Permission::Notification => PermissionStatus::Unknown,
            Permission::Accessibility => PermissionStatus::NotRequired,
        }
    }

    fn request_permission(&self, _app: &AppHandle, _permission: Permission) -> Result<(), String> {
        // Linux通常不需要特殊权限，但可能依赖特定服务
        Ok(())
    }

    fn show_notification(&self, _title: &str, _body: &str) -> Result<(), String> {
        // 使用libnotify或其他Linux通知系统
        #[cfg(target_os = "linux")]
        {
            // 这里会集成Linux特定通知实现
            Ok(())
        }
        #[cfg(not(target_os = "linux"))]
        Ok(())
    }

    fn platform_name(&self) -> &'static str {
        "Linux"
    }

    fn supports_transparency(&self) -> bool {
        false // 透明窗口在某些Linux桌面环境支持不佳
    }

    fn get_window_style(&self) -> WindowStyle {
        WindowStyle {
            transparent: false,
            decorations: true, // Linux通常保留装饰条
            skip_taskbar: false,
            always_on_top: true,
        }
    }
}

/// 获取当前平台的适配器
pub fn get_platform_adapter() -> Box<dyn PlatformAdapter> {
    #[cfg(target_os = "windows")]
    {
        Box::new(WindowsPlatform::new())
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(MacOSPlatform::new())
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(LinuxPlatform::new())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        // 默认使用Linux实现作为后备
        Box::new(LinuxPlatform::new())
    }
}

/// 获取当前平台的快捷键显示文本
pub fn get_shortcut_display_text(shortcut: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        shortcut.replace("Cmd", "⌘").replace("Alt", "⌥").replace("Shift", "⇧")
    }
    #[cfg(not(target_os = "macos"))]
    {
        shortcut.to_string()
    }
}

/// 检查权限并返回用户友好的错误信息
pub fn check_permissions_with_user_friendly_errors() -> Vec<String> {
    let adapter = get_platform_adapter();
    let mut errors = Vec::new();

    // Windows平台通常不需要特殊权限，直接返回空错误列表
    #[cfg(target_os = "windows")]
    {
        return Vec::new();
    }

    // macOS和Linux平台检查权限
    #[cfg(not(target_os = "windows"))]
    {
        let accessibility_status = adapter.check_permission(Permission::Accessibility);
        if matches!(accessibility_status, PermissionStatus::Denied) {
            errors.push(format!(
                "{} 需要辅助功能权限来监听全局快捷键。请在系统设置中启用。",
                adapter.platform_name()
            ));
        }

        let notification_status = adapter.check_permission(Permission::Notification);
        if matches!(notification_status, PermissionStatus::Denied) {
            errors.push(format!(
                "{} 需要通知权限来显示剪切板操作提示。",
                adapter.platform_name()
            ));
        }
    }

    errors
}