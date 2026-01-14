use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
};

use crate::app::{App, Mode};
use crate::buffer::OutputKind;

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

                let content = &output_line.content;

                // Build spans for this line
                let mut spans = vec![Span::styled(prefix, prefix_style)];

                // Check for search highlights
                if !search_state.query().is_empty() {
                    let matches: Vec<_> = search_state
                        .matches()
                        .iter()
                        .filter(|m| m.line == line_idx)
                        .collect();

                    if matches.is_empty() {
                        spans.push(Span::raw(content.as_str()));
                    } else {
                        let mut last_end = 0;
                        for m in matches {
                            if m.start > last_end {
                                spans.push(Span::raw(&content[last_end..m.start]));
                            }
                            let is_current = current_match_line == Some(line_idx);
                            let highlight_style = if is_current {
                                Style::default()
                                    .bg(Color::Yellow)
                                    .fg(Color::Black)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().bg(Color::DarkGray).fg(Color::White)
                            };
                            let end = m.start + m.len;
                            spans.push(Span::styled(&content[m.start..end], highlight_style));
                            last_end = end;
                        }
                        if last_end < content.len() {
                            spans.push(Span::raw(&content[last_end..]));
                        }
                    }
                } else {
                    spans.push(Span::raw(content.as_str()));
                }

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
            Mode::Normal => Style::default().bg(Color::Blue).fg(Color::White),
            Mode::Search => Style::default().bg(Color::Magenta).fg(Color::White),
        };

        let paragraph = Paragraph::new(content).style(style);
        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{OutputKind, OutputLine};
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
}
