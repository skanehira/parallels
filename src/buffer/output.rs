use std::collections::VecDeque;

/// Output type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputKind {
    Stdout,
    Stderr,
}

/// Output line structure
#[derive(Debug, Clone)]
pub struct OutputLine {
    /// Output type
    pub kind: OutputKind,
    /// Content (may contain ANSI escape sequences)
    pub content: String,
}

impl OutputLine {
    /// Create a new OutputLine
    pub fn new(kind: OutputKind, content: String) -> Self {
        Self { kind, content }
    }

    /// Return a string with display prefix
    pub fn with_prefix(&self) -> String {
        let prefix = match self.kind {
            OutputKind::Stdout => "[stdout]",
            OutputKind::Stderr => "[stderr]",
        };
        format!("{} {}", prefix, self.content)
    }
}

/// Ring buffer for output lines
///
/// When max lines is exceeded, old lines are automatically discarded.
/// Uses VecDeque internally for O(1) removal from the front.
pub struct OutputBuffer {
    lines: VecDeque<OutputLine>,
    max_lines: usize,
}

impl OutputBuffer {
    /// Create a buffer with specified max lines
    ///
    /// # Arguments
    /// * `max_lines` - Maximum number of lines to keep (0 for unlimited)
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: VecDeque::new(),
            max_lines,
        }
    }

    /// Add an output line
    ///
    /// When max_lines is exceeded, the oldest line is discarded.
    pub fn push(&mut self, line: OutputLine) {
        if self.max_lines > 0 && self.lines.len() >= self.max_lines {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    /// Get lines in specified range
    ///
    /// # Arguments
    /// * `start` - Start index (0-based)
    /// * `count` - Number of lines to get
    ///
    /// # Returns
    /// Lines in the specified range. Empty or partial result if out of bounds.
    pub fn get_range(&self, start: usize, count: usize) -> Vec<&OutputLine> {
        self.lines.iter().skip(start).take(count).collect()
    }

    /// Return the number of lines in the buffer
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Return an iterator over all lines (for search)
    pub fn iter(&self) -> impl Iterator<Item = &OutputLine> {
        self.lines.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn output_buffer_push_adds_line_to_buffer() {
        let mut buffer = OutputBuffer::new(100);
        buffer.push(OutputLine::new(OutputKind::Stdout, "hello".into()));

        assert_eq!(buffer.len(), 1);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn output_buffer_push_discards_oldest_line_when_max_exceeded() {
        let mut buffer = OutputBuffer::new(3);
        buffer.push(OutputLine::new(OutputKind::Stdout, "line1".into()));
        buffer.push(OutputLine::new(OutputKind::Stdout, "line2".into()));
        buffer.push(OutputLine::new(OutputKind::Stdout, "line3".into()));
        buffer.push(OutputLine::new(OutputKind::Stdout, "line4".into()));

        assert_eq!(buffer.len(), 3);
        let lines = buffer.get_range(0, 3);
        assert_eq!(lines[0].content, "line2");
        assert_eq!(lines[1].content, "line3");
        assert_eq!(lines[2].content, "line4");
    }

    #[test]
    fn output_buffer_push_unlimited_when_max_lines_is_zero() {
        let mut buffer = OutputBuffer::new(0);
        for i in 0..1000 {
            buffer.push(OutputLine::new(OutputKind::Stdout, format!("line{}", i)));
        }

        assert_eq!(buffer.len(), 1000);
    }

    #[test]
    fn output_buffer_get_range_returns_correct_lines() {
        let mut buffer = OutputBuffer::new(100);
        for i in 0..10 {
            buffer.push(OutputLine::new(OutputKind::Stdout, format!("line{}", i)));
        }

        let lines = buffer.get_range(3, 4);
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0].content, "line3");
        assert_eq!(lines[1].content, "line4");
        assert_eq!(lines[2].content, "line5");
        assert_eq!(lines[3].content, "line6");
    }

    #[test]
    fn output_buffer_get_range_returns_partial_when_exceeds_buffer() {
        let mut buffer = OutputBuffer::new(100);
        for i in 0..5 {
            buffer.push(OutputLine::new(OutputKind::Stdout, format!("line{}", i)));
        }

        let lines = buffer.get_range(3, 10);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].content, "line3");
        assert_eq!(lines[1].content, "line4");
    }

    #[test]
    fn output_buffer_get_range_returns_empty_when_start_exceeds_buffer() {
        let mut buffer = OutputBuffer::new(100);
        for i in 0..5 {
            buffer.push(OutputLine::new(OutputKind::Stdout, format!("line{}", i)));
        }

        let lines = buffer.get_range(10, 5);
        assert!(lines.is_empty());
    }

    #[test]
    fn output_buffer_iter_returns_all_lines() {
        let mut buffer = OutputBuffer::new(100);
        buffer.push(OutputLine::new(OutputKind::Stdout, "line1".into()));
        buffer.push(OutputLine::new(OutputKind::Stderr, "line2".into()));
        buffer.push(OutputLine::new(OutputKind::Stdout, "line3".into()));

        let contents: Vec<_> = buffer.iter().map(|l| l.content.as_str()).collect();
        assert_eq!(contents, vec!["line1", "line2", "line3"]);
    }

    #[rstest]
    #[case(OutputKind::Stdout, "hello", "[stdout] hello")]
    #[case(OutputKind::Stderr, "error", "[stderr] error")]
    fn output_line_with_prefix_returns_correct_prefix(
        #[case] kind: OutputKind,
        #[case] content: &str,
        #[case] expected: &str,
    ) {
        let line = OutputLine::new(kind, content.into());
        assert_eq!(line.with_prefix(), expected);
    }
}
