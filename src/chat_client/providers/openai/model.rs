use serde::{Deserialize, Serialize};

/// Represents the available OpenAI models for chat completions.
///
/// This enum defines the different OpenAI models that can be used for chat
/// completions, each with different capabilities, performance characteristics,
/// and pricing. The enum uses serde renaming to match the exact model names
/// expected by the OpenAI API.
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::providers::openai::model::OpenAiModel;
///
/// let model = OpenAiModel::Gpt4o;
/// let mini_model = OpenAiModel::Gpt4oMini;
/// let latest_model = OpenAiModel::GptO3;
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum OpenAiModel {
    /// GPT-4o - Latest high-performance model with vision capabilities
    #[serde(rename = "gpt-4o")]
    Gpt4o,
    /// GPT-4o Mini - Smaller, faster, and more cost-effective variant
    #[serde(rename = "gpt-4o-mini")]
    Gpt4oMini,
    /// GPT-o3 - Next-generation model with enhanced reasoning capabilities
    #[serde(rename = "gpt-o3")]
    GptO3,
    /// GPT-o4 Mini - Compact version of the o4 model family
    #[serde(rename = "gpt-o4-mini")]
    GptO4Mini,
    /// GPT-o4 Mini High - High-performance variant of the o4 mini model
    #[serde(rename = "gpt-o4-mini-high")]
    GptO4MiniHigh,
}