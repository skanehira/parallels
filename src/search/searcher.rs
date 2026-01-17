use tui_input::{Input, InputRequest};

use crate::buffer::OutputBuffer;

/// Search match information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// Line number (0-based)
    pub line: usize,
    /// Start position within the line (byte offset)
    pub start: usize,
    /// Length of matched string (in bytes)
    pub len: usize,
}

/// Search state management structure
pub struct SearchState {
    input: Input,
    matches: Vec<Match>,
    current_index: Option<usize>,
}

impl SearchState {
    /// Create an empty search state
    pub fn new() -> Self {
        Self {
            input: Input::default(),
            matches: Vec::new(),
            current_index: None,
        }
    }

    /// Get the search query
    pub fn query(&self) -> &str {
        self.input.value()
    }

    /// Handle input request from tui-input
    pub fn handle_input(&mut self, req: InputRequest) {
        self.input.handle(req);
    }

    /// Set search query and search the buffer for matches
    ///
    /// Uses smartcase: if query contains no uppercase letters, search is
    /// case-insensitive. If query contains uppercase, search is case-sensitive.
    ///
    /// TODO: Consider using more efficient search algorithms (e.g., Boyer-Moore,
    /// Aho-Corasick, or regex-based search) for better performance with large buffers.
    pub fn search(&mut self, query: &str, buffer: &OutputBuffer) {
        self.input = query.into();
        self.matches.clear();
        self.current_index = None;

        if query.is_empty() {
            return;
        }

        // Smartcase: case-insensitive if query has no uppercase letters
        let case_sensitive = query.chars().any(|c| c.is_uppercase());

        for (line_idx, line) in buffer.iter().enumerate() {
            // Use pre-stripped content for searching
            let content = line.plain();

            if case_sensitive {
                // Case-sensitive search
                let mut start = 0;
                while let Some(pos) = content[start..].find(query) {
                    let absolute_pos = start + pos;
                    self.matches.push(Match {
                        line: line_idx,
                        start: absolute_pos,
                        len: query.len(),
                    });
                    start = absolute_pos + query.len();
                }
            } else {
                // Case-insensitive search
                let query_lower = query.to_lowercase();
                let content_lower = content.to_lowercase();
                let mut start = 0;
                while let Some(pos) = content_lower[start..].find(&query_lower) {
                    let absolute_pos = start + pos;
                    self.matches.push(Match {
                        line: line_idx,
                        start: absolute_pos,
                        len: query.len(),
                    });
                    start = absolute_pos + query.len();
                }
            }
        }

        if !self.matches.is_empty() {
            self.current_index = Some(0);
        }
    }

    /// Get match results
    pub fn matches(&self) -> &[Match] {
        &self.matches
    }

    /// Get match count
    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// Get current match index (1-based, for display)
    pub fn current_match_display(&self) -> Option<usize> {
        self.current_index.map(|i| i + 1)
    }

    /// Get current match
    pub fn current_match(&self) -> Option<&Match> {
        self.current_index.and_then(|i| self.matches.get(i))
    }

    /// Move to next match and return its line number
    pub fn next_match(&mut self) -> Option<usize> {
        if self.matches.is_empty() {
            return None;
        }

        let new_index = match self.current_index {
            Some(i) => (i + 1) % self.matches.len(),
            None => 0,
        };
        self.current_index = Some(new_index);
        self.matches.get(new_index).map(|m| m.line)
    }

    /// Move to previous match and return its line number
    pub fn prev_match(&mut self) -> Option<usize> {
        if self.matches.is_empty() {
            return None;
        }

        let new_index = match self.current_index {
            Some(i) => {
                if i == 0 {
                    self.matches.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.matches.len() - 1,
        };
        self.current_index = Some(new_index);
        self.matches.get(new_index).map(|m| m.line)
    }

    /// Clear search state
    pub fn clear(&mut self) {
        self.input.reset();
        self.matches.clear();
        self.current_index = None;
    }

    /// Clear only the input field (preserves matches)
    pub fn clear_input(&mut self) {
        self.input.reset();
    }

    /// Check if search is active (query is not empty)
    pub fn is_active(&self) -> bool {
        !self.query().is_empty()
    }

    /// Check if matches exist
    pub fn has_matches(&self) -> bool {
        !self.matches.is_empty()
    }
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{OutputKind, OutputLine};

    fn create_buffer_with_lines(lines: &[&str]) -> OutputBuffer {
        let mut buffer = OutputBuffer::new(100);
        for line in lines {
            buffer.push(OutputLine::new(OutputKind::Stdout, (*line).to_string()));
        }
        buffer
    }

    #[test]
    fn search_state_new_returns_empty_state() {
        let state = SearchState::new();
        assert!(state.query().is_empty());
        assert!(state.matches().is_empty());
        assert!(!state.is_active());
    }

    #[test]
    fn search_state_search_finds_simple_match() {
        let buffer = create_buffer_with_lines(&["hello world", "goodbye world"]);
        let mut state = SearchState::new();

        state.search("world", &buffer);

        assert_eq!(state.query(), "world");
        assert_eq!(state.match_count(), 2);
        assert!(state.is_active());

        let matches = state.matches();
        assert_eq!(matches[0].line, 0);
        assert_eq!(matches[0].start, 6);
        assert_eq!(matches[0].len, 5);

        assert_eq!(matches[1].line, 1);
        assert_eq!(matches[1].start, 8);
        assert_eq!(matches[1].len, 5);
    }

    #[test]
    fn search_state_search_finds_multiple_matches_in_same_line() {
        let buffer = create_buffer_with_lines(&["foo bar foo baz foo"]);
        let mut state = SearchState::new();

        state.search("foo", &buffer);

        assert_eq!(state.match_count(), 3);

        let matches = state.matches();
        assert_eq!(matches[0].start, 0);
        assert_eq!(matches[1].start, 8);
        assert_eq!(matches[2].start, 16);
    }

    #[test]
    fn search_state_search_returns_empty_when_no_match() {
        let buffer = create_buffer_with_lines(&["hello world"]);
        let mut state = SearchState::new();

        state.search("xyz", &buffer);

        assert!(state.matches().is_empty());
        assert_eq!(state.current_match_display(), None);
    }

    #[test]
    fn search_state_search_returns_empty_when_query_is_empty() {
        let buffer = create_buffer_with_lines(&["hello world"]);
        let mut state = SearchState::new();

        state.search("", &buffer);

        assert!(state.matches().is_empty());
        assert!(!state.is_active());
    }

    #[test]
    fn search_state_next_match_cycles_through_matches() {
        let buffer = create_buffer_with_lines(&["line1 foo", "line2", "line3 foo"]);
        let mut state = SearchState::new();
        state.search("foo", &buffer);

        assert_eq!(state.current_match_display(), Some(1));
        assert_eq!(state.current_match().unwrap().line, 0);

        let line = state.next_match();
        assert_eq!(line, Some(2));
        assert_eq!(state.current_match_display(), Some(2));

        let line = state.next_match();
        assert_eq!(line, Some(0)); // cycles back
        assert_eq!(state.current_match_display(), Some(1));
    }

    #[test]
    fn search_state_prev_match_cycles_through_matches() {
        let buffer = create_buffer_with_lines(&["line1 foo", "line2", "line3 foo"]);
        let mut state = SearchState::new();
        state.search("foo", &buffer);

        assert_eq!(state.current_match_display(), Some(1));

        let line = state.prev_match();
        assert_eq!(line, Some(2)); // cycles to last
        assert_eq!(state.current_match_display(), Some(2));

        let line = state.prev_match();
        assert_eq!(line, Some(0));
        assert_eq!(state.current_match_display(), Some(1));
    }

    #[test]
    fn search_state_next_match_returns_none_when_no_matches() {
        let buffer = create_buffer_with_lines(&["hello"]);
        let mut state = SearchState::new();
        state.search("xyz", &buffer);

        assert_eq!(state.next_match(), None);
    }

    #[test]
    fn search_state_prev_match_returns_none_when_no_matches() {
        let buffer = create_buffer_with_lines(&["hello"]);
        let mut state = SearchState::new();
        state.search("xyz", &buffer);

        assert_eq!(state.prev_match(), None);
    }

    #[test]
    fn search_state_clear_resets_state() {
        let buffer = create_buffer_with_lines(&["hello world"]);
        let mut state = SearchState::new();
        state.search("hello", &buffer);

        assert!(state.is_active());

        state.clear();

        assert!(state.query().is_empty());
        assert!(state.matches().is_empty());
        assert!(!state.is_active());
    }

    // Smartcase tests: lowercase query = case-insensitive, uppercase query = case-sensitive

    #[test]
    fn search_state_smartcase_lowercase_query_matches_both_cases() {
        let buffer = create_buffer_with_lines(&["Hello World", "hello world", "HELLO WORLD"]);
        let mut state = SearchState::new();

        // Lowercase query should match all cases
        state.search("hello", &buffer);
        assert_eq!(state.match_count(), 3);
        assert_eq!(state.matches()[0].line, 0);
        assert_eq!(state.matches()[1].line, 1);
        assert_eq!(state.matches()[2].line, 2);
    }

    #[test]
    fn search_state_smartcase_uppercase_query_matches_exact_case() {
        let buffer = create_buffer_with_lines(&["Hello World", "hello world", "HELLO WORLD"]);
        let mut state = SearchState::new();

        // Query with uppercase should be case-sensitive
        state.search("Hello", &buffer);
        assert_eq!(state.match_count(), 1);
        assert_eq!(state.matches()[0].line, 0);
    }

    #[test]
    fn search_state_smartcase_all_caps_query_matches_exact() {
        let buffer = create_buffer_with_lines(&["Hello World", "hello world", "HELLO WORLD"]);
        let mut state = SearchState::new();

        state.search("HELLO", &buffer);
        assert_eq!(state.match_count(), 1);
        assert_eq!(state.matches()[0].line, 2);
    }

    #[test]
    fn search_state_smartcase_preserves_match_positions() {
        let buffer = create_buffer_with_lines(&["Hello World", "hello world"]);
        let mut state = SearchState::new();

        // Case-insensitive search should return correct positions in original text
        state.search("world", &buffer);
        assert_eq!(state.match_count(), 2);
        // "World" at position 6 in "Hello World"
        assert_eq!(state.matches()[0].start, 6);
        assert_eq!(state.matches()[0].len, 5);
        // "world" at position 6 in "hello world"
        assert_eq!(state.matches()[1].start, 6);
        assert_eq!(state.matches()[1].len, 5);
    }

    // Multibyte character tests (Japanese, etc.)

    #[test]
    fn search_state_finds_japanese_text() {
        let buffer = create_buffer_with_lines(&["こんにちは世界", "さようなら世界", "hello world"]);
        let mut state = SearchState::new();

        state.search("世界", &buffer);
        assert_eq!(state.match_count(), 2);
        assert_eq!(state.matches()[0].line, 0);
        assert_eq!(state.matches()[1].line, 1);
    }

    #[test]
    fn search_state_japanese_match_has_correct_byte_positions() {
        // "こんにちは" = 5 chars * 3 bytes = 15 bytes
        // "世界" starts at byte 15
        let buffer = create_buffer_with_lines(&["こんにちは世界"]);
        let mut state = SearchState::new();

        state.search("世界", &buffer);
        assert_eq!(state.match_count(), 1);
        assert_eq!(state.matches()[0].start, 15); // byte position
        assert_eq!(state.matches()[0].len, 6); // "世界" = 2 chars * 3 bytes
    }

    #[test]
    fn search_state_finds_japanese_in_mixed_text() {
        let buffer =
            create_buffer_with_lines(&["Error: エラーが発生しました", "Warning: 警告メッセージ"]);
        let mut state = SearchState::new();

        state.search("エラー", &buffer);
        assert_eq!(state.match_count(), 1);
        assert_eq!(state.matches()[0].line, 0);
        // "Error: " = 7 bytes, then "エラー" starts
        assert_eq!(state.matches()[0].start, 7);
    }

    #[test]
    fn search_state_finds_multiple_japanese_matches_in_same_line() {
        let buffer = create_buffer_with_lines(&["エラー: エラーが発生、エラーを確認"]);
        let mut state = SearchState::new();

        state.search("エラー", &buffer);
        assert_eq!(state.match_count(), 3);
        // First "エラー" at position 0
        assert_eq!(state.matches()[0].start, 0);
        // Second "エラー" at position 10 (": " = 2 bytes + "エラー" = 9 bytes = 11, but let's check)
        // "エラー: " = 9 + 2 = 11 bytes, second "エラー" starts at 11
        // Actually: "エラー" (9) + ": " (2) = 11
    }

    #[test]
    fn search_state_japanese_with_ascii_query() {
        let buffer = create_buffer_with_lines(&["日本語とEnglishの混合", "純粋な日本語テキスト"]);
        let mut state = SearchState::new();

        // Search for ASCII in mixed text
        state.search("English", &buffer);
        assert_eq!(state.match_count(), 1);
        assert_eq!(state.matches()[0].line, 0);
    }

    #[test]
    fn search_state_emoji_search() {
        let buffer = create_buffer_with_lines(&["成功 ✓ 完了", "失敗 ✗ エラー", "✓ OK"]);
        let mut state = SearchState::new();

        state.search("✓", &buffer);
        assert_eq!(state.match_count(), 2);
        assert_eq!(state.matches()[0].line, 0);
        assert_eq!(state.matches()[1].line, 2);
    }

    #[test]
    fn clear_input_should_clear_query_but_preserve_matches() {
        let buffer = create_buffer_with_lines(&["hello world", "hello rust"]);
        let mut state = SearchState::new();

        state.search("hello", &buffer);
        assert_eq!(state.query(), "hello");
        assert_eq!(state.matches().len(), 2);

        state.clear_input();

        assert_eq!(state.query(), ""); // 入力がクリアされる
        assert_eq!(state.matches().len(), 2); // マッチは保持される
    }

    #[test]
    fn has_matches_returns_true_when_matches_exist_after_clear_input() {
        let buffer = create_buffer_with_lines(&["hello world"]);
        let mut state = SearchState::new();

        state.search("hello", &buffer);
        assert!(state.has_matches());

        state.clear_input();

        assert!(state.has_matches()); // クリア後もマッチは存在
    }

    #[test]
    fn search_replaces_previous_matches_with_new_ones() {
        let buffer = create_buffer_with_lines(&["hello world", "foo bar"]);
        let mut state = SearchState::new();

        // 最初の検索
        state.search("hello", &buffer);
        assert_eq!(state.matches().len(), 1);
        assert_eq!(state.matches()[0].line, 0); // "hello world" の行

        // 再検索 - 前のマッチが消えて新しいマッチに置き換わる
        state.search("foo", &buffer);
        assert_eq!(state.matches().len(), 1);
        assert_eq!(state.matches()[0].line, 1); // "foo bar" の行（前の "hello" のマッチはない）
    }

    #[test]
    fn search_after_clear_input_updates_matches_correctly() {
        let buffer = create_buffer_with_lines(&["hello world", "foo bar"]);
        let mut state = SearchState::new();

        // 最初の検索
        state.search("hello", &buffer);
        assert_eq!(state.matches().len(), 1);

        // 入力をクリア（マッチは保持）
        state.clear_input();
        assert!(state.has_matches());

        // 再検索 - 新しいマッチで更新される
        state.search("foo", &buffer);
        assert_eq!(state.matches().len(), 1);
        assert_eq!(state.matches()[0].line, 1); // "foo bar" の行のみ
    }
}
