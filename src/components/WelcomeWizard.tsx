import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

interface PlatformInfo {
  platform: string;
  defaultShortcut: string;
  shortcutModifier: string;
  supportsTransparency: boolean;
  windowStyle: {
    transparent: boolean;
    decorations: boolean;
    skipTaskbar: boolean;
    alwaysOnTop: boolean;
  };
}

interface WizardProps {
  onComplete: () => void;
  onClose: () => void;
}

const WelcomeWizard: React.FC<WizardProps> = ({ onComplete, onClose }) => {
  const [currentStep, setCurrentStep] = useState(0);
  const [platformInfo, setPlatformInfo] = useState<PlatformInfo | null>(null);
  const [permissionErrors, setPermissionErrors] = useState<string[]>([]);
  const [isCheckingPermissions, setIsCheckingPermissions] = useState(false);

  // 获取平台特定的步骤配置
  const getPlatformSpecificSteps = () => {
    const platform = platformInfo?.platform || '';

    switch (platform) {
      case 'Windows':
        return [
          { title: '欢迎使用 Clipper', description: '强大的剪切板历史管理工具' },
          { title: '快捷键设置', description: '了解如何快速访问剪切板历史' },
          { title: '开始使用', description: 'Clipper 已准备就绪' }
        ];

      case 'macOS':
        return [
          { title: '欢迎使用 Clipper', description: '强大的剪切板历史管理工具' },
          { title: '权限设置', description: '配置必要的系统权限' },
          { title: '快捷键设置', description: '了解如何快速访问剪切板历史' },
          { title: '开始使用', description: 'Clipper 已准备就绪' }
        ];

      case 'Linux':
        return [
          { title: '欢迎使用 Clipper', description: '强大的剪切板历史管理工具' },
          { title: '快捷键设置', description: '了解如何快速访问剪切板历史' },
          { title: '开始使用', description: 'Clipper 已准备就绪' }
        ];

      default:
        return [
          { title: '欢迎使用 Clipper', description: '强大的剪切板历史管理工具' },
          { title: '开始使用', description: 'Clipper 已准备就绪' }
        ];
    }
  };

  const steps = getPlatformSpecificSteps();

  useEffect(() => {
    loadPlatformInfo();
    // 向导开始时调整窗口大小
    adjustWindowSize(true);
  }, []);

  useEffect(() => {
    const platform = platformInfo?.platform || '';
    // Windows平台跳过权限检查，macOS和Linux在权限步骤时检查
    if (currentStep === getPermissionStepIndex(platform) && platform !== 'Windows') {
      checkPermissions();
    }
  }, [currentStep, platformInfo]);

  // 获取权限检查的步骤索引
  const getPermissionStepIndex = (platform: string): number => {
    switch (platform) {
      case 'Windows': return -1; // Windows跳过权限检查
      case 'macOS': return 1;   // 第2步
      case 'Linux': return -1;  // Linux暂时跳过
      default: return -1;
    }
  };

  // 动态调整窗口大小
  const adjustWindowSize = async (isWizardMode: boolean) => {
    try {
      const window = getCurrentWindow();
      if (isWizardMode) {
        await window.setSize({ width: 600, height: 700 });
      } else {
        await window.setSize({ width: 400, height: 500 });
      }
    } catch (error) {
      console.error('调整窗口大小失败:', error);
    }
  };

  const loadPlatformInfo = async () => {
    try {
      const info = await invoke<PlatformInfo>('get_platform_info');
      setPlatformInfo(info);
    } catch (error) {
      console.error('获取平台信息失败:', error);
    }
  };

  const checkPermissions = async () => {
    setIsCheckingPermissions(true);
    try {
      const errors = await invoke<string[]>('check_permissions');
      setPermissionErrors(errors);
    } catch (error) {
      console.error('检查权限失败:', error);
    } finally {
      setIsCheckingPermissions(false);
    }
  };

  const openSystemSettings = async (settingType: string) => {
    try {
      await invoke('open_system_settings', { settingType });
    } catch (error) {
      console.error('打开系统设置失败:', error);
    }
  };

  const handleNext = async () => {
    if (currentStep < steps.length - 1) {
      setCurrentStep(currentStep + 1);
    } else {
      // 向导完成时恢复窗口大小并最小化
      await adjustWindowSize(false);
      try {
        const window = getCurrentWindow();
        await window.minimize();
      } catch (error) {
        console.error('最小化窗口失败:', error);
      }
      onComplete();
    }
  };

  const handlePrevious = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  // 向导完成处理
  const handleWizardComplete = async () => {
    try {
      await adjustWindowSize(false);
      const window = getCurrentWindow();
      await window.minimize();
      onComplete();
    } catch (error) {
      console.error('向导完成处理失败:', error);
      onComplete();
    }
  };

  const renderStepContent = () => {
    const platform = platformInfo?.platform || '';

    // 根据平台和当前步骤渲染不同内容
    if (platform === 'Windows') {
      return renderWindowsStepContent();
    } else if (platform === 'macOS') {
      return renderMacOSSstepContent();
    } else if (platform === 'Linux') {
      return renderLinuxStepContent();
    } else {
      return renderGenericStepContent();
    }
  };

  // Windows平台特定内容
  const renderWindowsStepContent = () => {
    switch (currentStep) {
      case 0: // 欢迎页
        return (
          <div className="welcome-content">
            <div className="welcome-icon">📋</div>
            <h2>欢迎使用 Clipper</h2>
            <p>一个强大的剪切板历史管理工具</p>
            <ul className="feature-list">
              <li>📝 智能剪切板历史记录</li>
              <li>⌨️ 全局快捷键快速访问</li>
              <li>🔍 强大的搜索功能</li>
              <li>💾 安全的本地存储</li>
            </ul>
          </div>
        );

      case 1: // 快捷键设置
        return (
          <div className="shortcut-info">
            <h2>快捷键设置</h2>
            <p>在Windows上，使用以下快捷键访问剪切板历史：</p>
            <div className="current-shortcut">
              <span className="shortcut-key">Alt + 2</span>
            </div>
            <div className="completion-tips">
              <h3>使用提示：</h3>
              <ul>
                <li>按 Alt + 2 随时唤出剪切板历史</li>
                <li>点击项目直接粘贴到当前输入框</li>
                <li>右键点击项目可以删除</li>
                <li>应用会在系统托盘运行</li>
              </ul>
            </div>
          </div>
        );

      case 2: // 完成页
        return (
          <div className="completion-step">
            <div className="success-icon">🎉</div>
            <h2>设置完成!</h2>
            <p>Clipper 已经准备就绪，现在开始使用吧！</p>
            <div className="completion-tips">
              <h3>快速开始：</h3>
              <ul>
                <li>应用已最小化到系统托盘</li>
                <li>使用 Alt + 2 快捷键唤出剪切板历史</li>
                <li>享受便捷的剪切板管理体验</li>
              </ul>
            </div>
          </div>
        );

      default:
        return null;
    }
  };

  // macOS平台特定内容
  const renderMacOSSstepContent = () => {
    switch (currentStep) {
      case 0: // 欢迎页
        return (
          <div className="welcome-content">
            <div className="welcome-icon">📋</div>
            <h2>欢迎使用 Clipper</h2>
            <p>一个强大的剪切板历史管理工具</p>
            <ul className="feature-list">
              <li>📝 智能剪切板历史记录</li>
              <li>⌨️ 全局快捷键快速访问</li>
              <li>🔍 强大的搜索功能</li>
              <li>💾 安全的本地存储</li>
            </ul>
          </div>
        );

      case 1: // 权限设置
        return (
          <div className="permissions-step">
            <h2>权限设置</h2>
            <p>为了在macOS上正常使用，需要配置以下权限：</p>

            {isCheckingPermissions ? (
              <div className="checking-permissions">
                <div className="spinner"></div>
                <p>正在检查权限状态...</p>
              </div>
            ) : (
              <div className="permissions-list">
                {permissionErrors.length === 0 ? (
                  <div className="permission-success">
                    <span className="icon">✅</span>
                    <span>所有权限已正确配置</span>
                  </div>
                ) : (
                  permissionErrors.map((error, index) => (
                    <div key={index} className="permission-error">
                      <span className="icon">⚠️</span>
                      <div className="error-content">
                        <p>{error}</p>
                        <button
                          className="settings-button"
                          onClick={() => {
                            if (error.includes('辅助功能')) {
                              openSystemSettings('accessibility');
                            } else if (error.includes('通知')) {
                              openSystemSettings('notifications');
                            }
                          }}
                        >
                          打开系统设置
                        </button>
                      </div>
                    </div>
                  ))
                )}
              </div>
            )}
          </div>
        );

      case 2: // 快捷键设置
        return (
          <div className="shortcut-info">
            <h2>快捷键设置</h2>
            <p>在macOS上，使用以下快捷键访问剪切板历史：</p>
            <div className="current-shortcut">
              <span className="shortcut-key">⌘ + Shift + V</span>
            </div>
            <div className="completion-tips">
              <h3>使用提示：</h3>
              <ul>
                <li>按 ⌘ + Shift + V 随时唤出剪切板历史</li>
                <li>点击项目直接粘贴到当前输入框</li>
                <li>右键点击项目可以删除</li>
                <li>应用会在菜单栏运行</li>
              </ul>
            </div>
          </div>
        );

      case 3: // 完成页
        return (
          <div className="completion-step">
            <div className="success-icon">🎉</div>
            <h2>设置完成!</h2>
            <p>Clipper 已经准备就绪，现在开始使用吧！</p>
            <div className="completion-tips">
              <h3>快速开始：</h3>
              <ul>
                <li>应用已最小化到菜单栏</li>
                <li>使用 ⌘ + Shift + V 快捷键唤出剪切板历史</li>
                <li>享受便捷的剪切板管理体验</li>
              </ul>
            </div>
          </div>
        );

      default:
        return null;
    }
  };

  // Linux平台特定内容
  const renderLinuxStepContent = () => {
    switch (currentStep) {
      case 0: // 欢迎页
        return (
          <div className="welcome-content">
            <div className="welcome-icon">📋</div>
            <h2>欢迎使用 Clipper</h2>
            <p>一个强大的剪切板历史管理工具</p>
            <ul className="feature-list">
              <li>📝 智能剪切板历史记录</li>
              <li>⌨️ 全局快捷键快速访问</li>
              <li>🔍 强大的搜索功能</li>
              <li>💾 安全的本地存储</li>
            </ul>
          </div>
        );

      case 1: // 快捷键设置
        return (
          <div className="shortcut-info">
            <h2>快捷键设置</h2>
            <p>在Linux上，使用以下快捷键访问剪切板历史：</p>
            <div className="current-shortcut">
              <span className="shortcut-key">Alt + 2</span>
            </div>
            <div className="completion-tips">
              <h3>使用提示：</h3>
              <ul>
                <li>按 Alt + 2 随时唤出剪切板历史</li>
                <li>点击项目直接粘贴到当前输入框</li>
                <li>右键点击项目可以删除</li>
                <li>应用会在系统托盘运行</li>
              </ul>
            </div>
          </div>
        );

      case 2: // 完成页
        return (
          <div className="completion-step">
            <div className="success-icon">🎉</div>
            <h2>设置完成!</h2>
            <p>Clipper 已经准备就绪，现在开始使用吧！</p>
            <div className="completion-tips">
              <h3>快速开始：</h3>
              <ul>
                <li>应用已最小化到系统托盘</li>
                <li>使用 Alt + 2 快捷键唤出剪切板历史</li>
                <li>享受便捷的剪切板管理体验</li>
              </ul>
            </div>
          </div>
        );

      default:
        return null;
    }
  };

  // 通用平台内容
  const renderGenericStepContent = () => {
    switch (currentStep) {
      case 0:
        return (
          <div className="welcome-content">
            <div className="welcome-icon">📋</div>
            <h2>欢迎使用 Clipper</h2>
            <p>一个强大的剪切板历史管理工具</p>
            <ul className="feature-list">
              <li>📝 智能剪切板历史记录</li>
              <li>⌨️ 全局快捷键快速访问</li>
              <li>🔍 强大的搜索功能</li>
              <li>💾 安全的本地存储</li>
            </ul>
          </div>
        );

      case 1:
        return (
          <div className="completion-step">
            <div className="success-icon">🎉</div>
            <h2>设置完成!</h2>
            <p>Clipper 已经准备就绪，现在开始使用吧！</p>
            <div className="completion-tips">
              <h3>快速开始：</h3>
              <ul>
                <li>应用已最小化到系统托盘</li>
                <li>使用快捷键唤出剪切板历史</li>
                <li>享受便捷的剪切板管理体验</li>
              </ul>
            </div>
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <div className="welcome-wizard-overlay">
      <div className="welcome-wizard">
        <div className="wizard-header">
          <div className="step-indicators">
            {steps.map((_, index) => (
              <div
                key={index}
                className={`step-indicator ${index === currentStep ? 'active' : ''} ${index < currentStep ? 'completed' : ''}`}
              />
            ))}
          </div>
          <button className="close-button" onClick={onClose}>
            ✕
          </button>
        </div>

        <div className="wizard-content">
          <h1>{steps[currentStep].title}</h1>
          <p className="step-description">{steps[currentStep].description}</p>
          {renderStepContent()}
        </div>

        <div className="wizard-footer">
          <div className="step-counter">
            {currentStep + 1} / {steps.length}
          </div>
          <div className="wizard-actions">
            {currentStep > 0 && (
              <button className="button secondary" onClick={handlePrevious}>
                上一步
              </button>
            )}
            <button
              className="button primary"
              onClick={handleNext}
              // 只有在权限检查步骤且正在检查时禁用按钮
              disabled={
                (platformInfo?.platform === 'macOS' && currentStep === 1 && isCheckingPermissions) ||
                (platformInfo?.platform === 'Linux' && currentStep === 1 && isCheckingPermissions)
              }
            >
              {currentStep === steps.length - 1 ? '开始使用' : '下一步'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default WelcomeWizard;