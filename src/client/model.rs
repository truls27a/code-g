use crate::client::providers::openai::schema::Model as OpenAiModel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the result of a chat completion operation.
///
/// This enum encapsulates the two possible outcomes of a chat completion:
/// either a simple message response or a request to call one or more tools.
/// It provides a unified interface for handling both text responses and
/// function calling scenarios.
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::model::{ChatResult, ToolCall};
/// use std::collections::HashMap;
///
/// // Simple message response
/// let message_result = ChatResult::Message {
///     content: "Hello! How can I help you?".to_string(),
///     turn_over: true,
/// };
///
/// // Tool call response
/// let mut args = HashMap::new();
/// args.insert("location".to_string(), "London".to_string());
/// let tool_call = ToolCall {
///     id: "call_123".to_string(),
///     name: "get_weather".to_string(),
///     arguments: args,
/// };
/// let tool_result = ChatResult::ToolCalls(vec![tool_call]);
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ChatResult {
    /// A text message response with turn management information
    Message { content: String, turn_over: bool },
    /// One or more tool calls that the assistant wants to execute
    ToolCalls(Vec<ToolCall>),
}

/// Represents different types of messages in a chat conversation.
///
/// This enum defines the four types of messages that can appear in an OpenAI
/// chat completion conversation. Each variant has specific fields appropriate
/// for its role in the conversation flow, from system instructions to tool
/// execution results.
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::model::{ChatMessage, AssistantMessage};
///
/// // System message for instructions
/// let system_msg = ChatMessage::System {
///     content: "You are a helpful assistant.".to_string(),
/// };
///
/// // User message
/// let user_msg = ChatMessage::User {
///     content: "What's the weather like?".to_string(),
/// };
///
/// // Assistant response
/// let assistant_msg = ChatMessage::Assistant {
///     message: AssistantMessage::Content("Let me check that for you.".to_string()),
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ChatMessage {
    /// System message containing instructions or context for the assistant
    System { content: String },
    /// User message containing human input or queries
    User { content: String },
    /// Assistant message containing AI responses or tool calls
    Assistant { message: AssistantMessage },
    /// Tool message containing the results of function executions
    Tool {
        content: String,
        tool_call_id: String,
        tool_name: String,
    },
}

/// Represents the content of an assistant's message.
///
/// This enum allows assistant messages to contain either simple text content
/// or requests to call tools/functions. It provides a clean separation between
/// conversational responses and function calling behavior.
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::model::{AssistantMessage, ToolCall};
/// use std::collections::HashMap;
///
/// // Text content
/// let text_message = AssistantMessage::Content(
///     "I can help you with that!".to_string()
/// );
///
/// // Tool calls
/// let mut args = HashMap::new();
/// args.insert("query".to_string(), "weather".to_string());
/// let tool_call = ToolCall {
///     id: "call_456".to_string(),
///     name: "search".to_string(),
///     arguments: args,
/// };
/// let tool_message = AssistantMessage::ToolCalls(vec![tool_call]);
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum AssistantMessage {
    /// Plain text content from the assistant
    Content(String),
    /// One or more tool calls the assistant wants to make
    ToolCalls(Vec<ToolCall>),
}

/// Represents the model used for chat completions.
///
/// This enum encapsulates the different models that can be used for chat
/// completions, currently supporting only OpenAI models. It provides a
/// unified interface for working with different AI providers and their
/// model configurations.
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::model::Model;
/// use code_g::chat_client::providers::openai::schema::Model as OpenAiModel;
///
/// let model = Model::OpenAi(OpenAiModel::Gpt4o);
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Model {
    OpenAi(OpenAiModel),
}

/// Represents a tool or function available to the assistant.
///
/// This struct defines a tool that the OpenAI assistant can call during
/// a conversation. Tools enable the assistant to perform actions beyond
/// text generation, such as retrieving data, making calculations, or
/// interacting with external systems.
///
/// # Fields
///
/// * `tool_type` - The type of tool (currently only functions are supported)
/// * `function` - The function definition including parameters and description
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::model::{Tool, ToolType, Function, Parameters, Property};
/// use std::collections::HashMap;
///
/// let mut properties = HashMap::new();
/// properties.insert("location".to_string(), Property {
///     prop_type: "string".to_string(),
///     description: "The city name".to_string(),
/// });
///
/// let tool = Tool {
///     tool_type: ToolType::Function,
///     function: Function {
///         name: "get_weather".to_string(),
///         description: "Get current weather for a location".to_string(),
///         parameters: Parameters {
///             param_type: "object".to_string(),
///             properties,
///             required: vec!["location".to_string()],
///             additional_properties: false,
///         },
///         strict: true,
///     },
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: ToolType,
    pub function: Function,
}

/// Represents the type of tool available to the assistant.
///
/// This enum defines the different types of tools that can be registered
/// with the OpenAI API. Currently, only function calls are supported,
/// but this enum allows for future expansion to other tool types.
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::model::ToolType;
///
/// let tool_type = ToolType::Function;
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ToolType {
    /// Function tool type for calling custom functions
    Function,
}

/// Represents a function definition for tool calling.
///
/// This struct contains all the information needed to define a function
/// that the assistant can call, including its name, description, parameter
/// schema, and validation settings.
///
/// # Fields
///
/// * `name` - The name of the function to call
/// * `description` - Human-readable description of what the function does
/// * `parameters` - JSON schema defining the function's parameters
/// * `strict` - Whether to use strict parameter validation
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::model::{Function, Parameters, Property};
/// use std::collections::HashMap;
///
/// let mut properties = HashMap::new();
/// properties.insert("amount".to_string(), Property {
///     prop_type: "number".to_string(),
///     description: "The amount to calculate".to_string(),
/// });
///
/// let function = Function {
///     name: "calculate_tax".to_string(),
///     description: "Calculate tax on a given amount".to_string(),
///     parameters: Parameters {
///         param_type: "object".to_string(),
///         properties,
///         required: vec!["amount".to_string()],
///         additional_properties: false,
///     },
///     strict: true,
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Function {
    pub name: String,
    pub description: String,
    pub parameters: Parameters,
    pub strict: bool,
}

/// Represents the parameter schema for a function.
///
/// This struct defines the JSON schema for function parameters, following
/// the JSON Schema specification. It describes the structure, types, and
/// validation rules for the parameters that can be passed to a function.
///
/// # Fields
///
/// * `param_type` - The root type of the parameter schema (typically "object")
/// * `properties` - Map of parameter names to their property definitions
/// * `required` - List of required parameter names
/// * `additional_properties` - Whether additional properties are allowed
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::model::{Parameters, Property};
/// use std::collections::HashMap;
///
/// let mut properties = HashMap::new();
/// properties.insert("name".to_string(), Property {
///     prop_type: "string".to_string(),
///     description: "The user's name".to_string(),
/// });
/// properties.insert("age".to_string(), Property {
///     prop_type: "integer".to_string(),
///     description: "The user's age".to_string(),
/// });
///
/// let parameters = Parameters {
///     param_type: "object".to_string(),
///     properties,
///     required: vec!["name".to_string()],
///     additional_properties: false,
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    #[serde(rename = "type")]
    pub param_type: String, // usually "object"
    pub properties: HashMap<String, Property>,
    pub required: Vec<String>,
    pub additional_properties: bool,
}

/// Represents a single property definition in a function parameter schema.
///
/// This struct defines the type and description for an individual parameter
/// property, following JSON Schema conventions. It provides the basic
/// metadata needed for parameter validation and documentation.
///
/// # Fields
///
/// * `prop_type` - The JSON Schema type of this property (e.g., "string", "number")
/// * `description` - Human-readable description of this property's purpose
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::model::Property;
///
/// let string_prop = Property {
///     prop_type: "string".to_string(),
///     description: "The email address".to_string(),
/// };
///
/// let number_prop = Property {
///     prop_type: "number".to_string(),
///     description: "The price in USD".to_string(),
/// };
///
/// let boolean_prop = Property {
///     prop_type: "boolean".to_string(),
///     description: "Whether notifications are enabled".to_string(),
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    #[serde(rename = "type")]
    pub prop_type: String,
    pub description: String,
}

/// Represents a specific tool call made by the assistant.
///
/// This struct contains the details of a function call that the assistant
/// wants to execute, including a unique identifier, the function name,
/// and the arguments to pass to the function.
///
/// # Fields
///
/// * `id` - Unique identifier for this tool call
/// * `name` - The name of the function to call
/// * `arguments` - Map of argument names to their string values
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::model::ToolCall;
/// use std::collections::HashMap;
///
/// let mut arguments = HashMap::new();
/// arguments.insert("city".to_string(), "New York".to_string());
/// arguments.insert("units".to_string(), "metric".to_string());
///
/// let tool_call = ToolCall {
///     id: "call_weather_123".to_string(),
///     name: "get_current_weather".to_string(),
///     arguments,
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: HashMap<String, String>,
}
