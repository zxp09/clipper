use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use dirs::{data_dir, data_local_dir, config_dir};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: u64,
    pub content: String,
    pub timestamp: u64,
    pub is_favorite: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardData {
    pub items: Vec<ClipboardItem>,
    pub next_id: u64,
    pub settings: AppSettings,
    pub last_updated: u64,
    #[serde(default)]
    pub is_first_launch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub max_items: usize,
    pub max_size_mb: usize,
    pub auto_start: bool,
    pub shortcut: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        // 使用平台适配器获取默认快捷键
        let adapter = crate::platform::get_platform_adapter();
        Self {
            max_items: 100,
            max_size_mb: 50,
            auto_start: false,
            shortcut: adapter.default_shortcut(),
        }
    }
}

pub struct SimpleStorage {
    file_path: PathBuf,
    pub data: ClipboardData,
}

impl SimpleStorage {
    pub fn resolve_storage_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let mut candidates = Vec::new();
        candidates.push(data_local_dir());
        candidates.push(data_dir());
        candidates.push(config_dir());

        for candidate in candidates.into_iter().flatten() {
            let mut base = candidate.clone();
            base.push("clipper");
            if fs::create_dir_all(&base).is_ok() {
                base.push("clipboard_data.json");
                return Ok(base);
            }
        }

        let mut fallback = std::env::current_dir()?;
        fallback.push(".clipper");
        fs::create_dir_all(&fallback)?;
        fallback.push("clipboard_data.json");
        Ok(fallback)
    }

    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut path = Self::resolve_storage_path()?;

        if !path.exists() {
            let mut legacy = std::env::current_dir()?;
            legacy.push("clipboard_data.json");
            if legacy.exists() {
                if let Err(err) = fs::copy(&legacy, &path) {
                    eprintln!("迁移旧版剪切板数据失败: {}", err);
                }
            }
        }

        let data = if path.exists() {
            let content = fs::read_to_string(&path)?;

            // 首先尝试解析为完整结构
            match serde_json::from_str::<ClipboardData>(&content) {
                Ok(mut data) => {
                    // 如果成功解析但没有last_updated字段，添加当前时间
                    if data.last_updated == 0 {
                        data.last_updated = SystemTime::now()
                            .duration_since(UNIX_EPOCH)?
                            .as_secs();
                        // 立即保存更新的数据
                        let updated_content = serde_json::to_string_pretty(&data)?;
                        fs::write(&path, updated_content)?;
                    }
                    data
                }
                Err(_) => {
                    // 如果解析失败，尝试作为旧版本数据解析
                    #[derive(Deserialize)]
                    struct OldClipboardData {
                        items: Vec<ClipboardItem>,
                        next_id: u64,
                        settings: AppSettings,
                    }

                    let old_data: OldClipboardData = serde_json::from_str(&content)
                        .map_err(|e| format!("解析剪切板数据失败: {}", e))?;

                    // 转换为新格式并添加last_updated字段
                    let new_data = ClipboardData {
                        items: old_data.items,
                        next_id: old_data.next_id,
                        settings: old_data.settings,
                        last_updated: SystemTime::now()
                            .duration_since(UNIX_EPOCH)?
                            .as_secs(),
                        is_first_launch: false,
                    };

                    // 保存更新后的数据
                    let updated_content = serde_json::to_string_pretty(&new_data)?;
                    fs::write(&path, updated_content)?;

                    new_data
                }
            }
        } else {
            ClipboardData {
                items: Vec::new(),
                next_id: 1,
                settings: AppSettings::default(),
                last_updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)?
                    .as_secs(),
                is_first_launch: true,
            }
        };

        Ok(Self {
            file_path: path,
            data,
        })
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }

    pub fn add_item(&mut self, content: String) -> Result<u64, Box<dyn std::error::Error>> {
        // 检查重复内容
        if let Some(last_item) = self.data.items.last() {
            if last_item.content == content {
                return Ok(last_item.id);
            }
        }

        // 检查大文本 (>1MB)
        if content.len() > 1024 * 1024 {
            return Err("Content too large (>1MB)".into());
        }

        let item = ClipboardItem {
            id: self.data.next_id,
            content,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs(),
            is_favorite: false,
        };

        self.data.items.push(item);
        self.data.next_id += 1;

        // 更新最后修改时间
        self.data.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        // 清理旧项目
        self.enforce_item_limit()?;

        self.save()?;
        Ok(self.data.next_id - 1)
    }

    pub fn get_history(&self, limit: usize) -> Vec<ClipboardItem> {
        let mut items: Vec<ClipboardItem> = self.data.items.clone();
        // 按时间戳降序排列（最新的在前）
        items.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // 限制返回数量
        items.truncate(limit);
        items
    }

    pub fn get_all_items(&self) -> Vec<ClipboardItem> {
        let mut items: Vec<ClipboardItem> = self.data.items.clone();
        // 按时间戳降序排列（最新的在前）
        items.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        items
    }

    pub fn get_item_by_id(&self, id: u64) -> Option<&ClipboardItem> {
        self.data.items.iter().find(|item| item.id == id)
    }

    pub fn remove_item(&mut self, id: u64) -> Result<bool, Box<dyn std::error::Error>> {
        let original_len = self.data.items.len();
        self.data.items.retain(|item| item.id != id);
        let removed = self.data.items.len() < original_len;

        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    pub fn set_item_favorite(&mut self, id: u64, is_favorite: bool) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(item) = self.data.items.iter_mut().find(|item| item.id == id) {
            if item.is_favorite != is_favorite {
                item.is_favorite = is_favorite;
                self.data.last_updated = SystemTime::now()
                    .duration_since(UNIX_EPOCH)?
                    .as_secs();
                self.save()?;
            }
            return Ok(true);
        }
        Ok(false)
    }

    pub fn clear_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.data.items.clear();
        self.data.next_id = 1;
        self.save()?;
        Ok(())
    }

    pub fn search_items(&self, query: &str) -> Vec<ClipboardItem> {
        let mut items: Vec<ClipboardItem> = if query.is_empty() {
            self.data.items.clone()
        } else {
            self.data.items
                .iter()
                .filter(|item| item.content.to_lowercase().contains(&query.to_lowercase()))
                .cloned()
                .collect()
        };

        // 按时间戳降序排列（最新的在前）
        items.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        items
    }

    pub fn get_last_updated(&self) -> u64 {
        self.data.last_updated
    }

    pub fn enforce_item_limit(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let max_items = self.data.settings.max_items;

        if self.data.items.len() > max_items {
            let remove_count = self.data.items.len() - max_items;
            // 保留收藏的项目
            let mut to_remove = Vec::new();

            for (index, item) in self.data.items.iter().enumerate() {
                if !item.is_favorite && to_remove.len() < remove_count {
                    to_remove.push(index);
                }
            }

            // 从后往前删除，避免索引错位
            for &index in to_remove.iter().rev() {
                self.data.items.remove(index);
            }
        }

        Ok(())
    }
}

// 类型别名，便于在 Tauri 命令中使用
pub type SharedStorage = Arc<Mutex<SimpleStorage>>;
