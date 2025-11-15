## ADDED Requirements

### Requirement: Clipboard History Storage
The application SHALL maintain a persistent history of clipboard items with configurable limits.

#### 场景：存储带元数据的项目
当检测到新剪切板内容时，系统应将其与元数据一起存储，包括时间戳、内容类型和大小。

#### 场景：历史大小管理
当历史记录达到默认限制（100个项目或总计50MB）时，系统应自动删除最旧的项目。

#### 场景：用户可配置限制
用户应能够在设置中配置存储限制，新限制立即生效。

### Requirement: History Search and Display
The application SHALL provide a minimal interface for browsing and searching clipboard history.

#### 场景：时间顺序列表
系统应按时间顺序显示项目，最新的项目在前。

#### 场景：实时搜索
当用户在搜索框中输入时，系统应根据内容和元数据实时过滤项目。

#### 场景：图片缩略图
对于图片项目，系统应显示缩略图预览和元数据（尺寸、文件大小）。

#### 场景：简洁界面
系统应使用简洁的最小化设计，具有清晰的文本预览和小型缩略图。

### Requirement: Item Selection and Pasting
Users SHALL be able to select items from history and paste them into applications.

#### 场景：点击复制
当用户点击历史项目时，系统应将该内容复制到系统剪切板。

#### 场景：键盘导航
用户应能够使用箭头键导航项目，并用回车键选择。

#### 场景：数字键快速粘贴
当历史记录可见时，按数字键（1-9）应立即复制相应项目。

### Requirement: Configuration Management
The application SHALL provide user-configurable settings for storage and behavior.

#### 场景：存储限制配置
用户应能够配置最大项目数量和总存储大小。

#### 场景：启动行为配置
用户应能够切换应用程序是否随系统自动启动。

#### 场景：设置持久性
所有配置更改应在应用程序重启后保持存在。