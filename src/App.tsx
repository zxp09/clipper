import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Settings from "./Settings";
import "./App.css";

interface ClipboardItem {
  id: number;
  content: string;
  timestamp: number;
  is_favorite: boolean;
}

function App() {
  const [clipboardHistory, setClipboardHistory] = useState<ClipboardItem[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [shortcutConflict, setShortcutConflict] = useState<{
    message: string;
    suggestion: string;
  } | null>(null);
  const [showRestartConfirm, setShowRestartConfirm] = useState(false);
  const [currentPage, setCurrentPage] = useState<'history' | 'settings'>('history');
  const [clipboardMonitoringEnabled, setClipboardMonitoringEnabled] = useState(false);
  const [lastUpdateTime, setLastUpdateTime] = useState<number>(0);

  // 按需剪切板监控定时器
  const startClipboardMonitoring = () => {
    if (clipboardMonitoringEnabled) return; // 避免重复启动

    // 启用后端监控
    invoke('toggle_clipboard_monitoring', { enable: true })
      .then(() => {
        setClipboardMonitoringEnabled(true);
        console.log('剪切板监控已启动');
      })
      .catch(err => console.error('启动剪切板监控失败:', err));

    // 设置前端定时检查
    const checkInterval = setInterval(async () => {
      try {
        const newItem = await invoke<ClipboardItem | null>('check_clipboard_changes');
        if (newItem) {
          setClipboardHistory(prev => [newItem, ...prev]);
          console.log('检测到新的剪切板内容:', newItem.content.substring(0, 50) + '...');
        }
      } catch (error) {
        console.error('检查剪切板变化失败:', error);
      }
    }, 2000); // 每2秒检查一次

    // 清理函数
    return () => {
      clearInterval(checkInterval);
      setClipboardMonitoringEnabled(false);
      invoke('toggle_clipboard_monitoring', { enable: false })
        .catch(err => console.error('停止剪切板监控失败:', err));
    };
  };

  // 停止剪切板监控
  const stopClipboardMonitoring = () => {
    if (clipboardMonitoringEnabled) {
      invoke('toggle_clipboard_monitoring', { enable: false })
        .then(() => {
          setClipboardMonitoringEnabled(false);
          console.log('剪切板监控已停止');
        })
        .catch(err => console.error('停止剪切板监控失败:', err));
    }
  };

  const closeSettingsPage = async () => {
    try {
      await invoke('hide_window');
    } catch (error) {
      console.error('关闭窗口失败:', error);
    } finally {
      setCurrentPage('history');
    }
  };


  // 检查剪切板数据是否更新（仅在生产模式下使用）
  const checkForUpdates = async () => {
    // 在开发模式下不需要这个功能，因为使用轮询
    if (process.env.NODE_ENV === 'development') return;

    try {
      const currentLastUpdated = await invoke<number>('get_last_updated');
      if (currentLastUpdated > lastUpdateTime) {
        console.log('检测到剪切板数据更新，重新加载...');
        setLastUpdateTime(currentLastUpdated);
        loadClipboardHistory();
      }
    } catch (error) {
      console.error('检查更新失败:', error);
    }
  };

  // 加载剪切板历史
  const loadClipboardHistory = async () => {
    setIsLoading(true);
    try {
      const [history, lastUpdated] = await Promise.all([
        invoke<ClipboardItem[]>("get_clipboard_history", { limit: 100 }),
        invoke<number>("get_last_updated")
      ]);
      setClipboardHistory(history);
      setLastUpdateTime(lastUpdated);
    } catch (error) {
      console.error("加载剪切板历史失败:", error);
    } finally {
      setIsLoading(false);
    }
  };

  // 加载设置
  const loadSettings = async () => {
    try {
      await invoke<any>("get_settings");
      // 设置在组件内部不再需要存储，仅用于加载
    } catch (error) {
      console.error("加载设置失败:", error);
    }
  };

  // 搜索剪切板项目
  const searchClipboard = async (query: string) => {
    if (!query.trim()) {
      loadClipboardHistory();
      return;
    }

    try {
      const results = await invoke<ClipboardItem[]>("search_clipboard_items", { query });
      setClipboardHistory(results);
    } catch (error) {
      console.error("搜索失败:", error);
    }
  };

  // 复制到剪切板
  const copyToClipboard = async (content: string) => {
    try {
      await invoke("copy_to_clipboard", { content });
      console.log("内容已复制到剪切板");
    } catch (error) {
      console.error("复制失败:", error);
    }
  };

  // 将文本输入到焦点输入框
  const typeToFocusedInput = async (content: string) => {
    try {
      await invoke("type_text_to_focused_input", { text: content });
      console.log("内容已输入到焦点输入框");
    } catch (error) {
      console.error("输入失败:", error);
      // 如果输入失败，回退到复制到剪切板
      copyToClipboard(content);
    }
  };

  // 删除项目
  const deleteItem = async (id: number) => {
    try {
      const success = await invoke<boolean>("delete_history_item", { id });
      if (success) {
        setClipboardHistory(prev => prev.filter(item => item.id !== id));
      }
    } catch (error) {
      console.error("删除失败:", error);
    }
  };

  // 格式化时间戳
  const formatTimestamp = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleString("zh-CN");
  };

  // 截取文本预览
  const getTextPreview = (content: string, maxLength: number = 100) => {
    if (content.length <= maxLength) return content;
    return content.substring(0, maxLength) + "...";
  };

  
  // 组件加载时获取数据
  useEffect(() => {
    loadClipboardHistory();
    loadSettings();

    let clipboardMonitorCleanup: (() => void) | undefined;
    let removeEventListeners: (() => void) | undefined;
    let disposed = false;

    if (process.env.NODE_ENV === 'development') {
      console.log('开发模式：启动按需剪切板监控');
      clipboardMonitorCleanup = startClipboardMonitoring();
    }

    const setupEventListeners = async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');

        const unlistenShortcutConflict = await listen('shortcut-conflict', (event: any) => {
          setShortcutConflict({
            message: event.payload.message,
            suggestion: event.payload.suggestion
          });
        });

        const unlistenShowSettings = await listen('show-settings', () => {
          setCurrentPage('settings');
        });

        const unlistenShowHistory = await listen('show-history', () => {
          setCurrentPage('history');
        });

        const unlistenClipboardUpdated = await listen('clipboard-updated', (event: any) => {
          const newItem = event.payload;
          setClipboardHistory(prev => [newItem, ...prev]);
          console.log('剪切板自动更新', newItem.content.substring(0, 50) + '...');
        });

        const cleanup = () => {
          unlistenShortcutConflict();
          unlistenShowSettings();
          unlistenShowHistory();
          unlistenClipboardUpdated();
        };

        if (disposed) {
          cleanup();
        } else {
          removeEventListeners = cleanup;
        }
      } catch (error) {
        console.error('注册事件监听失败:', error);
      }
    };

    setupEventListeners();

    return () => {
      disposed = true;
      if (clipboardMonitorCleanup) {
        clipboardMonitorCleanup();
      }
      stopClipboardMonitoring();
      if (removeEventListeners) {
        removeEventListeners();
      }
    };
  }, []);





  // 生产模式下的定时检查机制
  useEffect(() => {
    // 仅在生产模式下启用
    if (process.env.NODE_ENV === 'development') return;

    const checkInterval = setInterval(async () => {
      await checkForUpdates();
    }, 3000); // 每3秒检查一次

    return () => clearInterval(checkInterval);
  }, [lastUpdateTime]);

  // 处理搜索
  useEffect(() => {
    const timeoutId = setTimeout(() => {
      searchClipboard(searchQuery);
    }, 300);

    return () => clearTimeout(timeoutId);
  }, [searchQuery]);

  return (
    <div className="clipboard-manager">
      {/* 根据当前页面渲染不同内容 */}
      {currentPage === 'history' ? (
        <>
          {/* 历史列表页面 */}
          <header className="header">
            <div className="header-top">
              <h1 className="header-title">剪切板历史</h1>
            </div>
            <div className="header-bottom">
              <div className="search-bar">
                <input
                  type="text"
                  placeholder="搜索剪切板内容..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="search-input"
                />
              </div>
            </div>
          </header>

          <div className="main-content">
            <div className="history-panel">
          {isLoading ? (
            <div className="loading">加载中...</div>
          ) : (
            <div className="history-list">
              {clipboardHistory.length === 0 ? (
                <div className="empty-state">
                  <p>没有剪切板历史记录</p>
                </div>
              ) : (
                clipboardHistory.map((item) => (
                  <div
                    key={item.id}
                    className="history-item"
                    onClick={() => {
                      // 先隐藏窗口，让焦点回到原来的应用程序
                      invoke('hide_window').then(() => {
                        // 短暂延迟确保焦点回到原应用
                        setTimeout(() => {
                          typeToFocusedInput(item.content);
                        }, 100);
                      }).catch(console.error);
                    }}
                    onContextMenu={(e) => {
                      e.preventDefault();
                      deleteItem(item.id);
                    }}
                    title="点击输入到当前焦点输入框，右键删除"
                  >
                    <div className="item-content">
                      <div className="text-preview">
                        {getTextPreview(item.content)}
                      </div>
                      <div className="item-meta">
                        <span className="timestamp">{formatTimestamp(item.timestamp)}</span>
                        {item.is_favorite && <span className="favorite">⭐</span>}
                      </div>
                    </div>
                  </div>
                ))
              )}
            </div>
          )}
        </div>
      </div>
        </>
      ) : (
        <>
          {/* 设置页面 */}
          <Settings
            onRequestRestart={() => setShowRestartConfirm(true)}
            onClose={closeSettingsPage}
          />
        </>
      )}

      {/* 快捷键冲突提示模态框 */}
      {shortcutConflict && (
        <div className="shortcut-conflict-modal">
          <div className="conflict-content">
            <div className="conflict-header">
              <h3>⚠️ 快捷键冲突</h3>
              <button
                onClick={() => setShortcutConflict(null)}
                className="btn btn-small btn-secondary"
              >
                ✕
              </button>
            </div>
            <div className="conflict-body">
              <div className="conflict-message">
                <p>{shortcutConflict.message}</p>
              </div>
              <div className="conflict-suggestion">
                <p><strong>💡 解决方案：</strong></p>
                <p>请点击设置按钮，尝试以下备用快捷键组合：</p>
                <div className="alternative-shortcuts">
                  <button
                    className="shortcut-suggestion-btn"
                    onClick={() => setShortcutConflict(null)}
                  >
                    Ctrl+Alt+F7
                  </button>
                  <button
                    className="shortcut-suggestion-btn"
                    onClick={() => setShortcutConflict(null)}
                  >
                    Ctrl+Shift+F12
                  </button>
                  <button
                    className="shortcut-suggestion-btn"
                    onClick={() => setShortcutConflict(null)}
                  >
                    Ctrl+Alt+F9
                  </button>
                  <button
                    className="shortcut-suggestion-btn"
                    onClick={() => setShortcutConflict(null)}
                  >
                    Ctrl+Shift+V
                  </button>
                </div>
              </div>
              <div className="conflict-actions">
                <button
                  onClick={() => setShortcutConflict(null)}
                  className="btn btn-primary"
                >
                  知道了
                </button>
                <button
                  onClick={() => setShortcutConflict(null)}
                  className="btn btn-secondary"
                >
                  稍后处理
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 重启确认对话框 */}
      {showRestartConfirm && (
        <div className="shortcut-conflict-modal">
          <div className="conflict-content">
            <div className="conflict-header">
              <h3>🔄 重启应用</h3>
              <button
                onClick={() => setShowRestartConfirm(false)}
                className="btn btn-small btn-secondary"
              >
                ✕
              </button>
            </div>
            <div className="conflict-body">
              <div className="conflict-message">
                <p>快捷键已更新，立即重启让它生效？</p>
              </div>
              <div className="conflict-actions">
                <button
                  onClick={async () => {
                    setShowRestartConfirm(false);
                    try {
                      await invoke('restart_app');
                    } catch (error) {
                      console.error('重启应用失败:', error);
                    }
                  }}
                  className="btn btn-primary"
                >
                  立即重启
                </button>
                <button
                  onClick={() => setShowRestartConfirm(false)}
                  className="btn btn-secondary"
                >
                  稍后重启
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

          </div>
  );
}

export default App;
