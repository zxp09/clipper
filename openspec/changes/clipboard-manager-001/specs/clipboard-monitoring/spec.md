## ADDED Requirements

### Requirement: Monitor System Clipboard Changes
The application SHALL continuously monitor the system clipboard for new content changes and automatically record text and image clipboard items.

#### 场景：用户从任何应用程序复制文本
当用户在任何应用程序（网页浏览器、文本编辑器等）中复制文本内容时，剪切板管理器应检测此变化并将内容存储到历史数据库中，无需用户干预。

#### 场景：图片内容监控
当用户在任何应用程序中复制图片内容（PNG、JPEG、BMP）时，剪切板管理器应检测此变化并存储图片数据及生成的缩略图。

#### 场景：重复内容检测
当连续多次复制相同内容时，系统应只更新现有条目的时间戳以避免重复条目。

### Requirement: Content Type Processing
The system SHALL process only text content under 1MB and image formats, excluding other content types.

#### 场景：文本内容处理
当剪切板内容包含1MB以下的纯文本时，系统应以适当的UTF-8编码存储它。

#### 场景：大文本排除
当剪切板内容超过1MB大小时，系统应忽略内容并显示不支持大文本监控的通知。

#### 场景：图片内容处理
当剪切板内容包含支持的图片格式时，系统应存储数据并生成预览缩略图。

#### 场景：不支持内容过滤
当剪切板内容包含文件或其他不支持格式时，系统应在不通知的情况下忽略它们。

### Requirement: Application Lifecycle
The application SHALL operate as a system tray service with configurable startup behavior.

#### 场景：后台监控
当应用程序运行时，剪切板监控应在后台继续，仅显示系统托盘图标。

#### 场景：应用关闭行为
当用户完全退出应用程序时，剪切板监控应完全停止，无后台进程保留。

#### 场景：启动配置
应用程序应尊重用户的"随系统启动"设置，仅在启用时自动启动。