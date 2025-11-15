# 项目上下文

## 项目目的
Clipper 是一个跨平台剪切板管理应用程序，基于 Tauri 构建，允许用户在 Windows 和 macOS 上跟踪、管理和快速访问他们的剪切板历史记录。

## 技术栈
- **前端**: React + TypeScript
- **构建工具**: Vite
- **后端**: Rust + Tauri v2
- **支持平台**: Windows, macOS
- **状态管理**: React useState hooks
- **通信方式**: Tauri invoke 命令

## 项目约定

### 代码风格
- Rust: 标准 rustfmt 格式化
- TypeScript: 启用严格模式，ESLint 配置
- React: 函数式组件 + Hooks
- 文件命名: 文件使用 kebab-case，组件使用 PascalCase

### 架构模式
- **前后端通信**: 使用 Tauri 命令和 `#[tauri::command]` 宏
- **组件结构**: 单文件 React 组件
- **配置管理**: 基于 JSON 的配置（tauri.conf.json, package.json）
- **构建流程**: Vite 开发服务器运行在 1420 端口以兼容 Tauri

### 测试策略
- Rust: 使用 cargo test 测试后端逻辑
- 前端: 使用 React Testing Library 测试组件
- 集成测试: 通过 `npm run tauri dev` 进行手动测试

### Git 工作流
- 新功能开发使用功能分支
- 使用约定式提交信息
- 主分支保护，仅接受经过审查的合并

## ���域背景
这是一个桌面剪切板管理器，需要具备以下能力：
- 监控系统剪切板变化
- 安全存储剪切板历史
- 通过键盘快捷键提供快速访问
- 支持不同的内容类型（文本、图片等）
- 保持跨平台兼容性

## 重要约束
- **安全性**: 剪切板数据可能包含敏感信息
- **性能**: 对系统性能的影响要最小化
- **平台兼容性**: 必须在 Windows 和 macOS 上工作
- **存储限制**: 剪切板历史记录的实际大小限制
- **内存管理**: 大型剪切板项目的高效内存使用

## 外部依赖
- Tauri v2 桌面应用开发框架
- Tauri plugin shell 用于系统集成
- serde 用于 JSON 序列化
- 平台特定的剪切板 API（待确定）
