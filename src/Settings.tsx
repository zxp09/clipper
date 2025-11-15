import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./Settings.css";

interface SettingsProps {
  onRequestRestart?: () => void;
  onClose?: () => void;
}

interface BackendSettings {
  max_items: number;
  max_size_mb: number;
  auto_start: boolean;
  shortcut: string;
}

function Settings({ onRequestRestart, onClose }: SettingsProps) {
  const [currentShortcut, setCurrentShortcut] = useState("Ctrl+F11");
  const [shortcutSaving, setShortcutSaving] = useState(false);
  const [shortcutError, setShortcutError] = useState<string | null>(null);
  const [isRecordingShortcut, setIsRecordingShortcut] = useState(false);
  const [shortcutStatus, setShortcutStatus] = useState("快捷键用于快速显示/隐藏应用");
  const [maxItemsInput, setMaxItemsInput] = useState("100");
  const [maxItemsStatus, setMaxItemsStatus] = useState("");
  const [maxItemsError, setMaxItemsError] = useState<string | null>(null);
  const [showClearConfirm, setShowClearConfirm] = useState(false);
  const [clearLoading, setClearLoading] = useState(false);
  const setHotkeyPassthrough = (disabled: boolean) => {
    invoke("set_hotkey_passthrough", { disabled }).catch((error) => {
      console.error("更新热键穿透状态失败:", error);
    });
  };

  // 加载设置
  const loadSettings = async () => {
    try {
      const settings = await invoke<BackendSettings>("get_settings");
      if (settings) {
        if (settings.shortcut) {
          setCurrentShortcut(settings.shortcut);
        }
        if (typeof settings.max_items === 'number') {
          setMaxItemsInput(String(settings.max_items));
        }
      }
    } catch (error) {
      console.error("加载设置失败:", error);
    }
  };

  // 格式化按键组合为可读字符串
  const formatShortcut = (e: KeyboardEvent): string => {
    const parts: string[] = [];

    if (e.ctrlKey) parts.push('Ctrl');
    if (e.altKey) parts.push('Alt');
    if (e.shiftKey) parts.push('Shift');
    if (e.metaKey) parts.push('Cmd');

    // 添加主键
    if (e.key && !['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) {
      parts.push(e.key.toUpperCase());
    }

    return parts.join('+');
  };

  // 处理快捷键输入
  const handleShortcutKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    e.preventDefault();
    e.stopPropagation();

    console.log('=== 键盘事件触发 ===');
    console.log('事件类型:', e.type);
    console.log('目标元素:', e.target);

    // 添加更详细的调试信息
    console.log('按键详情:', {
      key: e.key,
      code: e.code,
      keyCode: e.keyCode,
      which: e.which
    });
    console.log('修饰键状态:', {
      ctrlKey: e.ctrlKey,
      altKey: e.altKey,
      shiftKey: e.shiftKey,
      metaKey: e.metaKey
    });

    const shortcut = formatShortcut(e.nativeEvent);
    console.log('格式化快捷键:', shortcut);

    const hasModifier = e.ctrlKey || e.altKey || e.shiftKey || e.metaKey;
    const hasMainKey = e.key && !['Control', 'Alt', 'Shift', 'Meta', 'Escape', 'Tab'].includes(e.key);

    console.log('快捷键验证:', { hasModifier, hasMainKey });

    if (hasModifier && hasMainKey) {
      console.log('✓ 快捷键验证通过，设置值');
      setCurrentShortcut(shortcut);
      setShortcutError(null);
      setShortcutStatus('快捷键设置成功');

      setTimeout(() => {
        e.currentTarget.blur();
        console.log('输入框已失去焦点');
      }, 100);
    } else if (e.key === 'Escape') {
      e.currentTarget.blur();
      console.log('ESC键按下，失去焦点');
    } else {
      console.log('✗ 快捷键验证失败，不处理');
    }
  };

  // 开始录制快捷键
  const startRecordingShortcut = () => {
    setIsRecordingShortcut(true);
    setShortcutStatus('请按下快捷键...');
    setShortcutError(null);
    setHotkeyPassthrough(true);
  };

  // 停止录制快捷键
  const stopRecordingShortcut = () => {
    setIsRecordingShortcut(false);
    setShortcutStatus('快捷键用于快速显示/隐藏应用');
    setHotkeyPassthrough(false);
  };

  // 清除所有历史
  const clearAllHistory = async () => {
    setClearLoading(true);
    try {
      await invoke("clear_all_history");
      alert("所有剪切板历史已清空");
      setShowClearConfirm(false);
    } catch (error) {
      console.error("清除失败:", error);
      alert("清除失败: " + error);
    } finally {
      setClearLoading(false);
    }
  };

  const saveMaxItems = async () => {
    const parsed = parseInt(maxItemsInput, 10);
    if (Number.isNaN(parsed)) {
      setMaxItemsError("请输入有效的数字");
      setMaxItemsStatus("");
      return;
    }

    if (parsed < 10 || parsed > 500) {
      setMaxItemsError("条数范围 10~500");
      setMaxItemsStatus("");
      return;
    }

    setMaxItemsError(null);
    try {
      await invoke("update_max_items", { max_items: parsed });
      setMaxItemsStatus(`已保存，最多保留 ${parsed} 条记录`);
    } catch (error) {
      console.error("保存最大条数失败:", error);
      setMaxItemsError("保存失败: " + error);
    }
  };


  // 保存快捷键
  const saveShortcut = async () => {
    console.log('=== 开始保存快捷键 ===');

    setShortcutSaving(true);

    if (!currentShortcut.trim()) {
      setShortcutError('请先输入快捷键');
      setShortcutSaving(false);
      return;
    }

    try {
      console.log('调用invoke update_shortcut...');
      const result = await invoke('update_shortcut', { shortcut: currentShortcut });
      console.log('invoke返回结果:', result);

      setShortcutStatus('快捷键保存成功！');
      console.log('快捷键保存成�?', currentShortcut);
      if (onRequestRestart) {
        onRequestRestart();
      }
    } catch (error) {
      console.error('保存快捷键失败:', error);
      console.error('错误详情:', error);
      setShortcutError('保存快捷键失败: ' + error);
    } finally {
      setShortcutSaving(false);
    }
  };

  // 组件加载时获取设置
  useEffect(() => {
    loadSettings();
  }, []);

  useEffect(() => {
    return () => {
      setHotkeyPassthrough(false);
    };
  }, []);

  return (
    <div className="settings-container">
      <div className="settings-content">
        {/* 页面头部 */}
        <div className="settings-header">
          <h2>应用设置</h2>
          {onClose && (
            <button
              className="btn btn-small btn-secondary"
              onClick={onClose}
            >
              关闭
            </button>
          )}
        </div>
        <div className="setting-item">
          <div style={{ display: 'flex', flexDirection: 'column', flex: 1, minWidth: 0 }}>
            <label>全局快捷键</label>
            <div className={`shortcut-status ${shortcutStatus.includes('成功') ? 'success' : shortcutStatus.includes('失败') ? 'error' : ''}`}>
              {shortcutStatus}
            </div>
          </div>
          <input
            type="text"
            value={currentShortcut}
            onChange={(e) => {
              setCurrentShortcut(e.target.value);
              // 如果是手动输入的格式，也更新状态
              if (e.target.value && e.target.value.includes('+')) {
                setShortcutError(null);
              }
            }}
            onKeyDown={handleShortcutKeyDown}
            onFocus={startRecordingShortcut}
            onBlur={stopRecordingShortcut}
            placeholder="点击输入框，然后按下快捷键"
            className={`shortcut-input ${isRecordingShortcut ? 'recording' : ''} ${shortcutError ? 'error' : ''}`}
          />
        </div>

        {shortcutError && (
          <div className="setting-error">
            {shortcutError}
          </div>
        )}

        <div className="setting-item">
          <div></div>
          <button
            className="btn btn-primary"
            onClick={saveShortcut}
            disabled={shortcutSaving}
          >
            {shortcutSaving ? '保存中...' : '保存快捷键'}
          </button>
        </div>

                <div className="setting-section">
          <h3>数据管理</h3>
          <div className="setting-item">
            <div style={{ display: 'flex', flexDirection: 'column', flex: 1, minWidth: 0 }}>
              <label>最多保留条数</label>
              <div className="shortcut-status">超出后将自动删除最早的非收藏记录</div>
            </div>
            <div className="max-items-control">
              <input
                type="number"
                min={10}
                max={500}
                value={maxItemsInput}
                onChange={(e) => {
                  setMaxItemsInput(e.target.value);
                  setMaxItemsStatus("");
                  setMaxItemsError(null);
                }}
              />
              <button
                className="btn btn-primary"
                onClick={saveMaxItems}
              >
                保存条数
              </button>
            </div>
          </div>

          {maxItemsError && (
            <div className="setting-error">
              {maxItemsError}
            </div>
          )}

          {maxItemsStatus && (
            <div className="setting-success">
              {maxItemsStatus}
            </div>
          )}

          <div className="setting-item">
            <div style={{ display: 'flex', flexDirection: 'column', flex: 1, minWidth: 0 }}>
              <label>清除剪切板历史</label>
              <div className="shortcut-status">删除所有剪切板历史记录，此操作不可恢复</div>
            </div>
            <button
              className="btn btn-danger"
              onClick={() => setShowClearConfirm(true)}
            >
              清除所有
            </button>
          </div>
        </div>

<div className="setting-hint">
          <strong>提示：</strong><br />
          • 建议使用不常用的快捷键组合，如：Ctrl+Alt+F7、Ctrl+Shift+F12<br />
          • 如果快捷键冲突，请尝试其他组合<br />
          • 修改快捷键后需要重启应用才能生效


        {showClearConfirm && (
          <div className="shortcut-conflict-modal">
            <div className="conflict-content">
              <div className="conflict-header">
                <h3>⚠️ 清空历史</h3>
                <button
                  onClick={() => setShowClearConfirm(false)}
                  className="btn btn-small btn-secondary"
                  disabled={clearLoading}
                >
                  ×
                </button>
              </div>
              <div className="conflict-body">
                <div className="conflict-message">
                  <p>清空所有剪切板记录？此操作不可恢复。</p>
                </div>
                <div className="conflict-actions">
                  <button
                    className="btn btn-danger"
                    onClick={clearAllHistory}
                    disabled={clearLoading}
                  >
                    {clearLoading ? '清除中...' : '确认清除'}
                  </button>
                  <button
                    className="btn btn-secondary"
                    onClick={() => setShowClearConfirm(false)}
                    disabled={clearLoading}
                  >
                    取消
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}
        </div>
      </div>
    </div>
  );
}

export default Settings;
