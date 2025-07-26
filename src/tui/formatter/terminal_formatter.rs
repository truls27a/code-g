/// Terminal control utilities
pub struct TerminalFormatter;

impl TerminalFormatter {
    /// Clear the entire screen and move cursor to top-left
    pub fn clear_screen() -> &'static str {
        "\x1B[2J\x1B[1;1H"
    }

    /// Save cursor position
    pub fn save_cursor() -> &'static str {
        "\x1B[s"
    }

    /// Restore cursor position
    pub fn restore_cursor() -> &'static str {
        "\x1B[u"
    }

    /// Move cursor to bottom of terminal
    pub fn move_to_bottom() -> &'static str {
        "\x1B[999;1H"
    }

    /// Clear current line
    pub fn clear_line() -> &'static str {
        "\x1B[K"
    }

    /// Move to bottom and clear line
    pub fn move_to_bottom_and_clear() -> &'static str {
        "\x1B[999;1H\x1B[K"
    }
}