use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
};

use crate::app::{App, Mode};
use crate::buffer::OutputKind;

/// A highlight range in original text positions
struct HighlightRange {
    start: usize,
    end: usize,
    is_current: bool,
}

/// Overlay search highlights on ANSI-parsed spans
///
/// Takes spans from ansi-to-tui and applies highlight styles to matching ranges.
fn overlay_highlights(
    spans: Vec<Span<'static>>,
    highlights: &[HighlightRange],
) -> Vec<Span<'static>> {
    if highlights.is_empty() {
        return spans;
    }

    let mut result = Vec::new();
    let mut pos = 0;

    for span in spans {
        let span_start = pos;
        let span_end = pos + span.content.len();
        let span_text = span.content.to_string();
        let base_style = span.style;

        // Find highlights that overlap with this span
        let overlapping: Vec<_> = highlights
            .iter()
            .filter(|h| h.start < span_end && h.end > span_start)
            .collect();

        if overlapping.is_empty() {
            // No highlights - keep span as-is
            result.push(Span::styled(span_text, base_style));
        } else {
            // Split span at highlight boundaries
            let mut current_pos = span_start;

            for highlight in overlapping {
                let hl_start = highlight.start.max(span_start);
                let hl_end = highlight.end.min(span_end);

                // Part before highlight
                if current_pos < hl_start {
                    let text = &span_text[current_pos - span_start..hl_start - span_start];
                    result.push(Span::styled(text.to_string(), base_style));
                }

                // Highlighted part - apply highlight style while preserving fg color
                let text = &span_text[hl_start - span_start..hl_end - span_start];
                let highlight_style = if highlight.is_current {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else {
                    Style::default().fg(Color::White).bg(Color::DarkGray)
                };
                result.push(Span::styled(text.to_string(), highlight_style));

                current_pos = hl_end;
            }

            // Part after all highlights
            if current_pos < span_end {
                let text = &span_text[current_pos - span_start..];
                result.push(Span::styled(text.to_string(), base_style));
            }
        }

        pos = span_end;
    }

    result
}

/// TUI rendering handler
pub struct Renderer;

impl Renderer {
    /// Render application state
    pub fn render(frame: &mut Frame, app: &App) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tab bar
                Constraint::Min(1),    // Output area
                Constraint::Length(1), // Status bar
            ])
            .split(frame.area());

        Self::render_tab_bar(frame, app, chunks[0]);
        Self::render_output_area(frame, app, chunks[1]);
        Self::render_status_bar(frame, app, chunks[2]);
    }

    /// Render the tab bar
    fn render_tab_bar(frame: &mut Frame, app: &App, area: Rect) {
        let tab_manager = app.tab_manager();
        let titles: Vec<Line> = tab_manager
            .iter()
            .map(|tab| Line::from(Span::raw(tab.display_name())))
            .collect();

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Commands"))
            .select(tab_manager.active_index())
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .divider("|");

        frame.render_widget(tabs, area);
    }

    /// Render the output area
    fn render_output_area(frame: &mut Frame, app: &App, area: Rect) {
        let tab = app.tab_manager().current_tab();
        let buffer = tab.buffer();
        let scroll_offset = tab.scroll_offset();

        let search_state = app.search_state();
        let current_match_line = search_state.current_match().map(|m| m.line);

        // Account for border (subtract 2 for top and bottom borders)
        let visible_height = area.height.saturating_sub(2) as usize;

        let lines: Vec<Line> = buffer
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(visible_height)
            .map(|(line_idx, output_line)| {
                let prefix = match output_line.kind {
                    OutputKind::Stdout => "[stdout] ",
                    OutputKind::Stderr => "[stderr] ",
                };

                let prefix_style = match output_line.kind {
                    OutputKind::Stdout => Style::default().fg(Color::Green),
                    OutputKind::Stderr => Style::default().fg(Color::Red),
                };

                let prefix_span = Span::styled(prefix, prefix_style);

                // Use pre-parsed spans from OutputLine
                let base_spans: Vec<Span<'static>> = output_line.spans().to_vec();

                // Check for search highlights
                let final_spans = if !search_state.query().is_empty() {
                    // Search active - overlay highlights on ANSI-parsed spans
                    let matches: Vec<_> = search_state
                        .matches()
                        .iter()
                        .filter(|m| m.line == line_idx)
                        .collect();

                    if matches.is_empty() {
                        base_spans
                    } else {
                        // Search positions are in stripped text coordinates
                        // ansi-to-tui spans are also in stripped text coordinates
                        // So we use the positions directly without conversion
                        let highlights: Vec<HighlightRange> = matches
                            .iter()
                            .map(|m| HighlightRange {
                                start: m.start,
                                end: m.start + m.len,
                                is_current: current_match_line == Some(line_idx),
                            })
                            .collect();

                        overlay_highlights(base_spans, &highlights)
                    }
                } else {
                    base_spans
                };

                let mut spans = vec![prefix_span];
                spans.extend(final_spans);
                Line::from(spans)
            })
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Output"))
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Render the status bar
    fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
        let mode = app.mode();
        let search_state = app.search_state();
        let tab = app.tab_manager().current_tab();

        let content = match mode {
            Mode::Normal => {
                let auto_scroll = if tab.auto_scroll() { "ON" } else { "OFF" };
                format!(
                    " NORMAL | Auto-scroll: {} | h/l:tabs j/k:scroll /:search q:quit",
                    auto_scroll
                )
            }
            Mode::Search => {
                let query = search_state.query();
                let total = search_state.match_count();
                let match_info = if let Some(current) = search_state.current_match_display() {
                    format!(" ({}/{})", current, total)
                } else if !query.is_empty() {
                    " (no matches)".to_string()
                } else {
                    String::new()
                };
                format!(
                    " SEARCH: {}{} | n/N:next/prev Esc:cancel",
                    query, match_info
                )
            }
        };

        let style = match mode {
            Mode::Normal => Style::default().fg(Color::Blue),
            Mode::Search => Style::default().fg(Color::Magenta),
        };

        let paragraph = Paragraph::new(content).style(style);
        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{OutputKind, OutputLine};
    use ansi_to_tui::IntoText;
    use ratatui::{Terminal, backend::TestBackend};

    /// Convert terminal buffer to string for snapshot testing
    fn buffer_to_string(terminal: &Terminal<TestBackend>) -> String {
        let buffer = terminal.backend().buffer();
        let mut result = String::new();

        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = &buffer[(x, y)];
                result.push_str(cell.symbol());
            }
            result.push('\n');
        }

        result
    }

    /// Create test app with given commands
    fn create_test_app(commands: Vec<&str>) -> App {
        App::new(commands.into_iter().map(String::from).collect(), 100)
    }

    /// Create test app with output lines
    fn create_test_app_with_output(commands: Vec<&str>, lines: Vec<(&str, OutputKind)>) -> App {
        let mut app = create_test_app(commands);
        // Disable auto-scroll to keep scroll at top
        app.tab_manager_mut()
            .current_tab_mut()
            .set_auto_scroll(false);
        for (content, kind) in lines {
            app.tab_manager_mut()
                .current_tab_mut()
                .push_output(OutputLine::new(kind, content.to_string()));
        }
        app
    }

    #[test]
    fn renderer_tab_bar_single_tab() {
        let app = create_test_app(vec!["echo hello"]);
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Renderer::render(frame, &app);
            })
            .unwrap();

        insta::assert_snapshot!(buffer_to_string(&terminal));
    }

    #[test]
    fn renderer_tab_bar_multiple_tabs() {
        let mut app = create_test_app(vec!["cmd1", "cmd2", "cmd3"]);
        app.tab_manager_mut().next_tab(); // Activate second tab
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Renderer::render(frame, &app);
            })
            .unwrap();

        insta::assert_snapshot!(buffer_to_string(&terminal));
    }

    #[test]
    fn renderer_output_area_with_stdout_stderr() {
        let app = create_test_app_with_output(
            vec!["test"],
            vec![
                ("hello world", OutputKind::Stdout),
                ("error message", OutputKind::Stderr),
                ("another line", OutputKind::Stdout),
            ],
        );
        let backend = TestBackend::new(50, 12);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Renderer::render(frame, &app);
            })
            .unwrap();

        insta::assert_snapshot!(buffer_to_string(&terminal));
    }

    #[test]
    fn renderer_status_bar_normal_mode() {
        let app = create_test_app(vec!["test"]);
        let backend = TestBackend::new(50, 8);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Renderer::render(frame, &app);
            })
            .unwrap();

        insta::assert_snapshot!(buffer_to_string(&terminal));
    }

    #[test]
    fn renderer_status_bar_search_mode() {
        use crate::app::Mode;

        let mut app = create_test_app_with_output(
            vec!["test"],
            vec![
                ("hello world", OutputKind::Stdout),
                ("hello there", OutputKind::Stdout),
                ("goodbye", OutputKind::Stdout),
            ],
        );
        app.set_mode(Mode::Search);
        app.search_in_current_tab("hello");

        let backend = TestBackend::new(50, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Renderer::render(frame, &app);
            })
            .unwrap();

        insta::assert_snapshot!(buffer_to_string(&terminal));
    }

    #[test]
    fn renderer_full_layout() {
        use crate::app::Mode;

        let mut app = create_test_app_with_output(
            vec!["cmd1", "cmd2"],
            vec![
                ("Starting...", OutputKind::Stdout),
                ("Processing...", OutputKind::Stdout),
                ("Warning: something", OutputKind::Stderr),
                ("Done.", OutputKind::Stdout),
            ],
        );
        app.set_mode(Mode::Search);
        app.search_in_current_tab("Done");

        let backend = TestBackend::new(60, 15);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Renderer::render(frame, &app);
            })
            .unwrap();

        insta::assert_snapshot!(buffer_to_string(&terminal));
    }

    // Tests for overlay_highlights function
    #[test]
    fn overlay_highlights_with_no_highlights_returns_original_spans() {
        let spans = vec![Span::raw("hello world".to_string())];
        let result = overlay_highlights(spans.clone(), &[]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "hello world");
    }

    #[test]
    fn overlay_highlights_highlights_middle_of_span() {
        let spans = vec![Span::raw("hello world".to_string())];
        let highlights = vec![HighlightRange {
            start: 6,
            end: 11,
            is_current: true,
        }];
        let result = overlay_highlights(spans, &highlights);

        // Should split into: "hello " + "world" (highlighted)
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].content, "hello ");
        assert_eq!(result[1].content, "world");
        assert_eq!(result[1].style.bg, Some(Color::Cyan));
    }

    #[test]
    fn overlay_highlights_with_ansi_text_highlights_correctly() {
        // Simulate what ansi-to-tui produces for "\x1b[31mERROR\x1b[0m: timeout"
        // ansi-to-tui strips ANSI codes, so spans contain only visible text
        let spans = vec![
            Span::styled("ERROR".to_string(), Style::default().fg(Color::Red)),
            Span::raw(": timeout".to_string()),
        ];

        // Search for "ERROR" - positions are in STRIPPED text (0-5)
        let highlights = vec![HighlightRange {
            start: 0,
            end: 5,
            is_current: true,
        }];

        let result = overlay_highlights(spans, &highlights);

        // "ERROR" should be highlighted
        assert_eq!(result[0].content, "ERROR");
        assert_eq!(result[0].style.bg, Some(Color::Cyan));
        // ": timeout" should remain unchanged
        assert_eq!(result[1].content, ": timeout");
        assert_eq!(result[1].style.bg, None);
    }

    #[test]
    fn overlay_highlights_search_error_in_ansi_colored_text() {
        // Real case: "\x1b[31m✗ ERROR: Connection timeout\x1b[0m"
        // ansi-to-tui produces: one span with "✗ ERROR: Connection timeout" in red
        let spans = vec![Span::styled(
            "✗ ERROR: Connection timeout".to_string(),
            Style::default().fg(Color::Red),
        )];

        // Search for "ERROR" - in stripped text it's at position 2-7
        // "✗ " is 4 bytes (✗ is 3 bytes + space), "ERROR" starts at byte 4
        let text = "✗ ERROR: Connection timeout";
        let error_start = text.find("ERROR").unwrap();
        let error_end = error_start + "ERROR".len();

        let highlights = vec![HighlightRange {
            start: error_start,
            end: error_end,
            is_current: true,
        }];

        let result = overlay_highlights(spans, &highlights);

        // Should have 3 spans: "✗ " + "ERROR" (highlighted) + ": Connection timeout"
        assert_eq!(result.len(), 3, "Expected 3 spans, got {:?}", result);
        assert_eq!(result[0].content, "✗ ");
        assert_eq!(result[1].content, "ERROR");
        assert_eq!(result[1].style.bg, Some(Color::Cyan));
        assert_eq!(result[2].content, ": Connection timeout");
    }

    #[test]
    fn renderer_search_with_ansi_text_highlights_correct_position() {
        // Test the full flow: ANSI text + search
        let raw_content = "\x1b[31m✗ ERROR: Connection timeout\x1b[0m";

        // 1. Parse ANSI (what ansi-to-tui does)
        let ansi_text = raw_content.into_text().unwrap();
        let base_spans: Vec<Span<'static>> = ansi_text.lines.into_iter().next().unwrap().spans;

        // 2. Derive stripped text from spans (same as OutputLine::stripped())
        let stripped: String = base_spans.iter().map(|s| s.content.to_string()).collect();
        assert_eq!(stripped, "✗ ERROR: Connection timeout");

        // Search for "ERROR" in stripped text
        let search_start = stripped.find("ERROR").unwrap();
        let search_end = search_start + "ERROR".len();

        // 3. Apply highlights using stripped positions directly
        let highlights = vec![HighlightRange {
            start: search_start,
            end: search_end,
            is_current: true,
        }];
        let result = overlay_highlights(base_spans, &highlights);

        // Verify "ERROR" is highlighted
        let highlighted_text: String = result
            .iter()
            .filter(|s| s.style.bg == Some(Color::Cyan))
            .map(|s| s.content.to_string())
            .collect();

        assert_eq!(highlighted_text, "ERROR");
    }
}
