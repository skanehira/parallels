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
    /// TODO: Consider using more efficient search algorithms (e.g., Boyer-Moore,
    /// Aho-Corasick, or regex-based search) for better performance with large buffers.
    pub fn search(&mut self, query: &str, buffer: &OutputBuffer) {
        self.input = query.into();
        self.matches.clear();
        self.current_index = None;

        if query.is_empty() {
            return;
        }

        for (line_idx, line) in buffer.iter().enumerate() {
            // Use pre-stripped content for searching
            let content = line.plain();
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

    /// Check if search is active (query is not empty)
    pub fn is_active(&self) -> bool {
        !self.query().is_empty()
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
    use rstest::rstest;

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

    #[rstest]
    #[case("Hello", 0)]
    #[case("hello", 1)]
    fn search_state_is_case_sensitive(#[case] query: &str, #[case] expected_line: usize) {
        let buffer = create_buffer_with_lines(&["Hello World", "hello world"]);
        let mut state = SearchState::new();

        state.search(query, &buffer);
        assert_eq!(state.match_count(), 1);
        assert_eq!(state.matches()[0].line, expected_line);
    }
}
