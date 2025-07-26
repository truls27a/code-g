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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_screen_returns_correct_sequence() {
        let result = TerminalFormatter::clear_screen();
        assert_eq!(result, "\x1B[2J\x1B[1;1H");
    }

    #[test]
    fn save_cursor_returns_correct_sequence() {
        let result = TerminalFormatter::save_cursor();
        assert_eq!(result, "\x1B[s");
    }

    #[test]
    fn restore_cursor_returns_correct_sequence() {
        let result = TerminalFormatter::restore_cursor();
        assert_eq!(result, "\x1B[u");
    }

    #[test]
    fn move_to_bottom_returns_correct_sequence() {
        let result = TerminalFormatter::move_to_bottom();
        assert_eq!(result, "\x1B[999;1H");
    }

    #[test]
    fn clear_line_returns_correct_sequence() {
        let result = TerminalFormatter::clear_line();
        assert_eq!(result, "\x1B[K");
    }

    #[test]
    fn move_to_bottom_and_clear_returns_correct_sequence() {
        let result = TerminalFormatter::move_to_bottom_and_clear();
        assert_eq!(result, "\x1B[999;1H\x1B[K");
    }

    #[test]
    fn move_to_bottom_and_clear_combines_sequences() {
        // Test that the combined sequence is equivalent to move_to_bottom + clear_line
        let result = TerminalFormatter::move_to_bottom_and_clear();
        let expected = format!(
            "{}{}",
            TerminalFormatter::move_to_bottom(),
            TerminalFormatter::clear_line()
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn all_methods_return_valid_ansi_sequences() {
        // Test that all methods return strings that start with ESC character
        let methods = [
            TerminalFormatter::clear_screen(),
            TerminalFormatter::save_cursor(),
            TerminalFormatter::restore_cursor(),
            TerminalFormatter::move_to_bottom(),
            TerminalFormatter::clear_line(),
            TerminalFormatter::move_to_bottom_and_clear(),
        ];

        for sequence in methods {
            assert!(
                sequence.starts_with('\x1B'),
                "ANSI sequence should start with ESC character: {}",
                sequence
            );
            assert!(!sequence.is_empty(), "Sequence should not be empty");
        }
    }
}
