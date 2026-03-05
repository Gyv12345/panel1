//! AI 配置持久化模块
//!
//! 负责读取/写入 Panel1 的 AI 模型配置，默认路径为 `~/.panel1/ai.toml`。

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// AI 协议类型（用于适配不同兼容接口）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AiProtocol {
    /// OpenAI 兼容协议
    Openai,
    /// Anthropic 兼容协议
    Anthropic,
}

impl AiProtocol {
    /// 返回协议字符串表示。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Openai => "openai",
            Self::Anthropic => "anthropic",
        }
    }

    /// 根据文本解析协议类型。
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "openai" | "open_ai" => Some(Self::Openai),
            "anthropic" | "claude" => Some(Self::Anthropic),
            _ => None,
        }
    }

    /// 返回该协议推荐的默认模型名。
    pub fn default_model(&self) -> &'static str {
        match self {
            Self::Openai => "gpt-4o-mini",
            Self::Anthropic => "claude-sonnet-4-5",
        }
    }
}

/// 存储中的模型配置档。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProfile {
    /// profile 名称（唯一标识）
    pub name: String,
    /// 协议类型（openai/anthropic）
    pub protocol: AiProtocol,
    /// 模型名称
    pub model: String,
    /// 自定义 API Base URL（可选）
    pub base_url: Option<String>,
    /// API Key（可选，模板 profile 可以为空）
    pub api_key: Option<String>,
    /// 说明信息（可选）
    pub description: Option<String>,
}

impl AiProfile {
    /// 生成脱敏后的 API Key，仅用于展示。
    pub fn masked_api_key(&self) -> String {
        self.api_key
            .as_deref()
            .map(mask_secret)
            .unwrap_or_else(|| "<empty>".to_string())
    }
}

/// 当前生效配置（从 active profile 派生）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// 当前 profile 名称
    pub profile_name: String,
    /// 协议类型
    pub protocol: AiProtocol,
    /// 模型名称
    pub model: String,
    /// 自定义 API Base URL（可选）
    pub base_url: Option<String>,
    /// API Key（可选）
    pub api_key: Option<String>,
    /// 说明信息（可选）
    pub description: Option<String>,
}

impl AiConfig {
    /// 生成脱敏后的 API Key，仅用于展示。
    pub fn masked_api_key(&self) -> String {
        self.api_key
            .as_deref()
            .map(mask_secret)
            .unwrap_or_else(|| "<empty>".to_string())
    }
}

/// 多 profile 持久化结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfigStore {
    /// 当前启用 profile
    pub active_profile: String,
    /// 可选 profile 列表
    pub profiles: Vec<AiProfile>,
}

impl Default for AiConfigStore {
    /// 返回默认空配置。
    fn default() -> Self {
        Self {
            active_profile: String::new(),
            profiles: Vec::new(),
        }
    }
}

impl AiConfigStore {
    /// 按名称获取 profile。
    pub fn profile(&self, name: &str) -> Option<&AiProfile> {
        self.profiles.iter().find(|profile| profile.name == name)
    }

    /// 按名称获取可变 profile。
    pub fn profile_mut(&mut self, name: &str) -> Option<&mut AiProfile> {
        self.profiles
            .iter_mut()
            .find(|profile| profile.name == name)
    }

    /// 获取当前 active profile。
    pub fn active_profile(&self) -> Option<&AiProfile> {
        self.profile(&self.active_profile)
    }

    /// 设置 active profile（若不存在返回 false）。
    pub fn set_active_profile(&mut self, name: &str) -> bool {
        if self.profile(name).is_none() {
            return false;
        }
        self.active_profile = name.to_string();
        true
    }

    /// 新增或替换 profile。
    pub fn upsert_profile(&mut self, profile: AiProfile) {
        if let Some(existing) = self.profile_mut(&profile.name) {
            *existing = profile;
        } else {
            self.profiles.push(profile);
        }

        if self.active_profile.trim().is_empty() {
            if let Some(first) = self.profiles.first() {
                self.active_profile = first.name.clone();
            }
        }
    }

    /// 删除指定 profile。
    pub fn remove_profile(&mut self, name: &str) -> bool {
        let before = self.profiles.len();
        self.profiles.retain(|profile| profile.name != name);
        if self.profiles.len() == before {
            return false;
        }

        if self.active_profile == name {
            self.active_profile = self
                .profiles
                .first()
                .map(|profile| profile.name.clone())
                .unwrap_or_default();
        }
        true
    }

    /// 转换当前 active profile 为 `AiConfig`。
    pub fn active_config(&self) -> Option<AiConfig> {
        let profile = self.active_profile()?;
        Some(AiConfig {
            profile_name: profile.name.clone(),
            protocol: profile.protocol,
            model: profile.model.clone(),
            base_url: profile.base_url.clone(),
            api_key: profile.api_key.clone(),
            description: profile.description.clone(),
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawConfigFile {
    Store(AiConfigStore),
    Legacy(LegacyAiConfig),
}

/// 旧版单 profile 配置格式（向后兼容读取）。
#[derive(Debug, Clone, Deserialize)]
struct LegacyAiConfig {
    protocol: AiProtocol,
    model: String,
    base_url: Option<String>,
    api_key: Option<String>,
}

/// 预置模型模板。
#[derive(Debug, Clone)]
pub struct AiModelPreset {
    /// profile 名称
    pub name: &'static str,
    /// 协议类型
    pub protocol: AiProtocol,
    /// 默认模型
    pub model: &'static str,
    /// 默认 Base URL（可选）
    pub base_url: Option<&'static str>,
    /// 说明
    pub description: &'static str,
}

impl AiModelPreset {
    /// 转换成可保存的 profile（不包含 API Key）。
    pub fn to_profile(&self) -> AiProfile {
        AiProfile {
            name: self.name.to_string(),
            protocol: self.protocol,
            model: self.model.to_string(),
            base_url: self.base_url.map(ToOwned::to_owned),
            api_key: None,
            description: Some(self.description.to_string()),
        }
    }
}

/// 返回内置模型模板（整理自 OpenClaw Wizard Provider 指引）。
pub fn builtin_model_presets() -> Vec<AiModelPreset> {
    vec![
        AiModelPreset {
            name: "openrouter-claude45",
            protocol: AiProtocol::Openai,
            model: "anthropic/claude-sonnet-4-5",
            base_url: Some("https://openrouter.ai/api/v1"),
            description: "OpenRouter（OpenAI 协议）+ Claude Sonnet 4.5",
        },
        AiModelPreset {
            name: "openrouter-deepseek-r1",
            protocol: AiProtocol::Openai,
            model: "deepseek/deepseek-r1",
            base_url: Some("https://openrouter.ai/api/v1"),
            description: "OpenRouter（OpenAI 协议）+ DeepSeek R1",
        },
        AiModelPreset {
            name: "moonshot-kimi-k2-5",
            protocol: AiProtocol::Openai,
            model: "kimi-k2-5-preview",
            base_url: Some("https://api.moonshot.ai/v1"),
            description: "Moonshot 国际站（OpenAI 协议）",
        },
        AiModelPreset {
            name: "moonshot-cn-kimi-k2-5",
            protocol: AiProtocol::Openai,
            model: "kimi-k2-5-preview",
            base_url: Some("https://api.moonshot.cn/v1"),
            description: "Moonshot 中国站（OpenAI 协议）",
        },
        AiModelPreset {
            name: "minimax-m2.5-anthropic",
            protocol: AiProtocol::Anthropic,
            model: "MiniMax-M2.5",
            base_url: Some("https://api.minimax.chat/anthropic/v1"),
            description: "MiniMax（Anthropic 协议）",
        },
        AiModelPreset {
            name: "deepseek-chat",
            protocol: AiProtocol::Openai,
            model: "deepseek-chat",
            base_url: Some("https://api.deepseek.com/v1"),
            description: "DeepSeek 官方（OpenAI 协议）",
        },
        AiModelPreset {
            name: "deepseek-reasoner",
            protocol: AiProtocol::Openai,
            model: "deepseek-reasoner",
            base_url: Some("https://api.deepseek.com/v1"),
            description: "DeepSeek R1 推理模型",
        },
        AiModelPreset {
            name: "qwen-plus",
            protocol: AiProtocol::Openai,
            model: "qwen-plus",
            base_url: Some("https://dashscope.aliyuncs.com/compatible-mode/v1"),
            description: "通义千问 DashScope（OpenAI 协议）",
        },
        AiModelPreset {
            name: "zhipu-plan-cn",
            protocol: AiProtocol::Anthropic,
            model: "glm-5",
            base_url: Some("https://open.bigmodel.cn/api/anthropic"),
            description: "智谱国内版（Anthropic 协议）+ GLM-5",
        },
        AiModelPreset {
            name: "glm-4.5",
            protocol: AiProtocol::Openai,
            model: "glm-4.5",
            base_url: Some("https://open.bigmodel.cn/api/paas/v4"),
            description: "智谱 GLM（OpenAI 协议）",
        },
    ]
}

/// 将内置模板写入配置（不覆盖已有 profile，除非 `overwrite=true`）。
pub fn seed_builtin_profiles(overwrite: bool) -> Result<AiConfigStore> {
    let mut store = load_ai_store()?;

    for preset in builtin_model_presets() {
        if !overwrite && store.profile(preset.name).is_some() {
            continue;
        }
        store.upsert_profile(preset.to_profile());
    }

    if store.active_profile.trim().is_empty() {
        store.active_profile = "deepseek-chat".to_string();
    }

    save_ai_store(&store)?;
    Ok(store)
}

/// 解析 AI 配置文件路径。
pub fn config_file_path() -> PathBuf {
    if let Ok(path) = std::env::var("PANEL1_AI_CONFIG_PATH") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".panel1").join("ai.toml")
}

/// 读取本地 AI 配置文件（active profile 视图）。
pub fn load_ai_config() -> Result<Option<AiConfig>> {
    let store = load_ai_store()?;
    Ok(store.active_config())
}

/// 保存 active AI 配置（写入/更新对应 profile）。
pub fn save_ai_config(config: &AiConfig) -> Result<PathBuf> {
    let mut store = load_ai_store()?;
    let profile_name = if config.profile_name.trim().is_empty() {
        if store.active_profile.trim().is_empty() {
            "default".to_string()
        } else {
            store.active_profile.clone()
        }
    } else {
        config.profile_name.trim().to_string()
    };

    let profile = AiProfile {
        name: profile_name.clone(),
        protocol: config.protocol,
        model: config.model.clone(),
        base_url: config.base_url.clone(),
        api_key: config.api_key.clone(),
        description: config.description.clone(),
    };

    store.upsert_profile(profile);
    store.active_profile = profile_name;
    save_ai_store(&store)
}

/// 读取完整配置存储。
pub fn load_ai_store() -> Result<AiConfigStore> {
    let path = config_file_path();
    if !path.exists() {
        return Ok(AiConfigStore::default());
    }

    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read AI config file: {}", path.display()))?;

    let parsed: RawConfigFile = toml::from_str(&raw)
        .with_context(|| format!("Failed to parse AI config file: {}", path.display()))?;

    let mut store = match parsed {
        RawConfigFile::Store(store) => store,
        RawConfigFile::Legacy(legacy) => AiConfigStore {
            active_profile: "default".to_string(),
            profiles: vec![AiProfile {
                name: "default".to_string(),
                protocol: legacy.protocol,
                model: legacy.model,
                base_url: legacy.base_url,
                api_key: legacy.api_key,
                description: Some("migrated-from-legacy-config".to_string()),
            }],
        },
    };

    normalize_store(&mut store);
    validate_store(&store)?;

    Ok(store)
}

/// 保存完整配置存储。
pub fn save_ai_store(store: &AiConfigStore) -> Result<PathBuf> {
    validate_store(store)?;

    let mut normalized = store.clone();
    normalize_store(&mut normalized);

    let path = config_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create AI config directory: {}", parent.display())
        })?;
    }

    let raw = toml::to_string_pretty(&normalized).context("Failed to serialize AI config")?;
    std::fs::write(&path, raw)
        .with_context(|| format!("Failed to write AI config file: {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permission = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&path, permission).with_context(|| {
            format!(
                "Failed to set AI config permissions to 600: {}",
                path.display()
            )
        })?;
    }

    Ok(path)
}

/// 脱敏密钥内容，仅用于展示日志或提示信息。
pub fn mask_secret(raw: &str) -> String {
    let value = raw.trim();
    if value.is_empty() {
        return "<empty>".to_string();
    }

    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= 8 {
        return "****".to_string();
    }

    let prefix: String = chars.iter().take(4).collect();
    let suffix: String = chars
        .iter()
        .rev()
        .take(4)
        .copied()
        .collect::<Vec<char>>()
        .into_iter()
        .rev()
        .collect();
    format!("{prefix}****{suffix}")
}

/// 规范化配置值（去除空白、空字符串转 None）。
fn normalize_store(store: &mut AiConfigStore) {
    store.active_profile = store.active_profile.trim().to_string();

    for profile in &mut store.profiles {
        profile.name = profile.name.trim().to_string();
        profile.model = profile.model.trim().to_string();
        profile.base_url = profile
            .base_url
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        profile.api_key = profile
            .api_key
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        profile.description = profile
            .description
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
    }

    store
        .profiles
        .retain(|profile| !profile.name.trim().is_empty() && !profile.model.trim().is_empty());

    if store.active_profile.is_empty() {
        if let Some(first) = store.profiles.first() {
            store.active_profile = first.name.clone();
        }
    }

    if store.profile(&store.active_profile).is_none() {
        if let Some(first) = store.profiles.first() {
            store.active_profile = first.name.clone();
        }
    }
}

/// 校验配置是否满足最小运行要求。
fn validate_store(store: &AiConfigStore) -> Result<()> {
    for profile in &store.profiles {
        if profile.name.trim().is_empty() {
            anyhow::bail!("AI profile name cannot be empty");
        }
        if profile.model.trim().is_empty() {
            anyhow::bail!("AI model cannot be empty (profile: {})", profile.name);
        }
    }

    if !store.profiles.is_empty() && store.active_profile.trim().is_empty() {
        anyhow::bail!("AI active_profile cannot be empty when profiles exist");
    }

    if !store.active_profile.trim().is_empty() && store.profile(&store.active_profile).is_none() {
        anyhow::bail!(
            "AI active_profile '{}' does not exist in profiles",
            store.active_profile
        );
    }

    Ok(())
}
