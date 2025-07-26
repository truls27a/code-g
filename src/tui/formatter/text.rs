/// Text formatting utilities for terminal output
pub struct TextFormatter;

impl TextFormatter {
    /// Reset all formatting
    pub fn reset() -> &'static str {
        "\x1B[0m"
    }

    /// Gray color
    pub fn gray() -> &'static str {
        "\x1B[90m"
    }

    /// Italic text
    pub fn italic() -> &'static str {
        "\x1B[3m"
    }

    /// Bold text
    pub fn bold() -> &'static str {
        "\x1B[1m"
    }

    /// Underline text
    pub fn underline() -> &'static str {
        "\x1B[4m"
    }

    /// Red color
    pub fn red() -> &'static str {
        "\x1B[91m"
    }

    /// Green color
    pub fn green() -> &'static str {
        "\x1B[92m"
    }

    /// Yellow color
    pub fn yellow() -> &'static str {
        "\x1B[93m"
    }

    /// Blue color
    pub fn blue() -> &'static str {
        "\x1B[94m"
    }

    /// Magenta color
    pub fn magenta() -> &'static str {
        "\x1B[95m"
    }

    /// Cyan color
    pub fn cyan() -> &'static str {
        "\x1B[96m"
    }

    /// White color
    pub fn white() -> &'static str {
        "\x1B[97m"
    }

    /// Format text with gray italic style (commonly used for tool calls)
    pub fn gray_italic(text: &str) -> String {
        format!(
            "{}{}{}{}",
            Self::gray(),
            Self::italic(),
            text,
            Self::reset()
        )
    }

    /// Format text with red italic style (commonly used for error messages)
    pub fn red_italic(text: &str) -> String {
        format!("{}{}{}{}", Self::red(), Self::italic(), text, Self::reset())
    }

    /// Format text with bold style
    pub fn bold_text(text: &str) -> String {
        format!("{}{}{}", Self::bold(), text, Self::reset())
    }

    /// Format text with a specific color
    pub fn colored_text(text: &str, color: &str) -> String {
        format!("{}{}{}", color, text, Self::reset())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gray_italic_formatting_works() {
        let result = TextFormatter::gray_italic("test text");
        assert_eq!(result, "\x1B[90m\x1B[3mtest text\x1B[0m");
    }

    #[test]
    fn red_italic_formatting_works() {
        let result = TextFormatter::red_italic("test text");
        assert_eq!(result, "\x1B[91m\x1B[3mtest text\x1B[0m");
    }

    #[test]
    fn bold_formatting_works() {
        let result = TextFormatter::bold_text("bold text");
        assert_eq!(result, "\x1B[1mbold text\x1B[0m");
    }

    #[test]
    fn colored_text_works() {
        let result = TextFormatter::colored_text("red text", TextFormatter::red());
        assert_eq!(result, "\x1B[91mred text\x1B[0m");
    }
}
