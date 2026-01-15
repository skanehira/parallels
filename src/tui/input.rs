use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tui_input::backend::crossterm::to_input_request;

use crate::app::{App, Mode};

/// Handle key event and update app state
pub fn handle_key(app: &mut App, key: KeyEvent) {
    // Ctrl-C quits from any mode
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.quit();
        return;
    }

    match app.mode() {
        Mode::Normal => handle_normal_mode(app, key),
        Mode::Search => handle_search_mode(app, key),
    }
}

/// Handle key event in Normal mode
fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        // Tab navigation (Ctrl-h/l)
        KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.tab_manager_mut().prev_tab();
        }
        KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.tab_manager_mut().next_tab();
        }

        // Horizontal scroll (h/l/0)
        KeyCode::Char('h') => app.tab_manager_mut().current_tab_mut().scroll_left(),
        KeyCode::Char('l') => app.tab_manager_mut().current_tab_mut().scroll_right(),
        KeyCode::Char('0') => app.tab_manager_mut().current_tab_mut().scroll_to_left(),

        // Vertical scroll (j/k)
        KeyCode::Char('j') => app.tab_manager_mut().current_tab_mut().scroll_down(),
        KeyCode::Char('k') => app.tab_manager_mut().current_tab_mut().scroll_up(),

        // Half-page scroll
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.tab_manager_mut()
                .current_tab_mut()
                .scroll_half_page_down();
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.tab_manager_mut()
                .current_tab_mut()
                .scroll_half_page_up();
        }

        // Jump to top/bottom
        KeyCode::Char('g') => app.tab_manager_mut().current_tab_mut().scroll_to_top(),
        KeyCode::Char('G') => app.tab_manager_mut().current_tab_mut().scroll_to_bottom(),

        // Toggle auto-scroll
        KeyCode::Char('f') => app.tab_manager_mut().current_tab_mut().toggle_auto_scroll(),

        // Enter search mode
        KeyCode::Char('/') => {
            app.set_mode(Mode::Search);
            // Disable auto-scroll when entering search mode
            app.tab_manager_mut()
                .current_tab_mut()
                .set_auto_scroll(false);
        }

        // Navigate search matches (only when search is active)
        KeyCode::Char('n') => {
            if app.search_state().is_active()
                && let Some(line) = app.search_state_mut().next_match()
            {
                app.tab_manager_mut().current_tab_mut().scroll_to_line(line);
            }
        }
        KeyCode::Char('N') => {
            if app.search_state().is_active()
                && let Some(line) = app.search_state_mut().prev_match()
            {
                app.tab_manager_mut().current_tab_mut().scroll_to_line(line);
            }
        }

        _ => {}
    }
}

/// Handle key event in Search mode
fn handle_search_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        // Exit search mode
        KeyCode::Esc => {
            app.set_mode(Mode::Normal);
        }

        // Confirm search and return to normal mode
        KeyCode::Enter => {
            app.set_mode(Mode::Normal);
        }

        // Delegate to tui-input for text editing (Emacs-like keybindings)
        _ => {
            if let Some(req) = to_input_request(&Event::Key(key)) {
                app.search_state_mut().handle_input(req);
                let query = app.search_state().query().to_string();
                app.search_in_current_tab(&query);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{OutputKind, OutputLine};

    fn create_app_with_output() -> App {
        let mut app = App::new(vec!["cmd1".into(), "cmd2".into()], 100);
        // Add some output lines
        for i in 0..20 {
            app.tab_manager_mut()
                .current_tab_mut()
                .push_output(OutputLine::new(OutputKind::Stdout, format!("line{}", i)));
        }
        app.tab_manager_mut()
            .current_tab_mut()
            .set_visible_lines(10);
        app.tab_manager_mut().current_tab_mut().scroll_to_top();
        app
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn key_with_ctrl(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    // Normal mode tests

    #[test]
    fn input_ctrl_c_quits_from_normal_mode() {
        let mut app = App::new(vec!["cmd".into()], 100);
        assert!(!app.should_quit());

        handle_key(&mut app, key_with_ctrl('c'));
        assert!(app.should_quit());
    }

    #[test]
    fn input_ctrl_c_quits_from_search_mode() {
        let mut app = App::new(vec!["cmd".into()], 100);
        app.set_mode(Mode::Search);
        assert!(!app.should_quit());

        handle_key(&mut app, key_with_ctrl('c'));
        assert!(app.should_quit());
    }

    #[test]
    fn input_normal_mode_ctrl_h_switches_to_prev_tab() {
        let mut app = App::new(vec!["cmd1".into(), "cmd2".into()], 100);
        app.tab_manager_mut().next_tab(); // Move to tab 1
        assert_eq!(app.tab_manager().active_index(), 1);

        handle_key(&mut app, key_with_ctrl('h'));
        assert_eq!(app.tab_manager().active_index(), 0);
    }

    #[test]
    fn input_normal_mode_ctrl_l_switches_to_next_tab() {
        let mut app = App::new(vec!["cmd1".into(), "cmd2".into()], 100);
        assert_eq!(app.tab_manager().active_index(), 0);

        handle_key(&mut app, key_with_ctrl('l'));
        assert_eq!(app.tab_manager().active_index(), 1);
    }

    #[test]
    fn input_normal_mode_h_scrolls_left() {
        let mut app = create_app_with_output();
        // First scroll right
        app.tab_manager_mut().current_tab_mut().scroll_right();
        app.tab_manager_mut().current_tab_mut().scroll_right();
        assert_eq!(app.tab_manager().current_tab().horizontal_scroll(), 2);

        handle_key(&mut app, key(KeyCode::Char('h')));
        assert_eq!(app.tab_manager().current_tab().horizontal_scroll(), 1);
    }

    #[test]
    fn input_normal_mode_l_scrolls_right() {
        let mut app = create_app_with_output();
        assert_eq!(app.tab_manager().current_tab().horizontal_scroll(), 0);

        handle_key(&mut app, key(KeyCode::Char('l')));
        assert_eq!(app.tab_manager().current_tab().horizontal_scroll(), 1);
    }

    #[test]
    fn input_normal_mode_0_scrolls_to_left() {
        let mut app = create_app_with_output();
        app.tab_manager_mut().current_tab_mut().scroll_right();
        app.tab_manager_mut().current_tab_mut().scroll_right();
        assert_eq!(app.tab_manager().current_tab().horizontal_scroll(), 2);

        handle_key(&mut app, key(KeyCode::Char('0')));
        assert_eq!(app.tab_manager().current_tab().horizontal_scroll(), 0);
    }

    #[test]
    fn input_normal_mode_j_scrolls_down() {
        let mut app = create_app_with_output();
        assert_eq!(app.tab_manager().current_tab().scroll_offset(), 0);

        handle_key(&mut app, key(KeyCode::Char('j')));
        assert_eq!(app.tab_manager().current_tab().scroll_offset(), 1);
    }

    #[test]
    fn input_normal_mode_k_scrolls_up() {
        let mut app = create_app_with_output();
        app.tab_manager_mut().current_tab_mut().scroll_to_line(5);

        handle_key(&mut app, key(KeyCode::Char('k')));
        assert_eq!(app.tab_manager().current_tab().scroll_offset(), 4);
    }

    #[test]
    fn input_normal_mode_ctrl_d_scrolls_half_page_down() {
        let mut app = create_app_with_output();

        handle_key(&mut app, key_with_ctrl('d'));
        assert_eq!(app.tab_manager().current_tab().scroll_offset(), 5); // half of 10
    }

    #[test]
    fn input_normal_mode_ctrl_u_scrolls_half_page_up() {
        let mut app = create_app_with_output();
        app.tab_manager_mut().current_tab_mut().scroll_to_line(10);

        handle_key(&mut app, key_with_ctrl('u'));
        assert_eq!(app.tab_manager().current_tab().scroll_offset(), 5);
    }

    #[test]
    fn input_normal_mode_g_scrolls_to_top() {
        let mut app = create_app_with_output();
        app.tab_manager_mut().current_tab_mut().scroll_to_line(10);

        handle_key(&mut app, key(KeyCode::Char('g')));
        assert_eq!(app.tab_manager().current_tab().scroll_offset(), 0);
    }

    #[test]
    fn input_normal_mode_shift_g_scrolls_to_bottom() {
        let mut app = create_app_with_output();

        handle_key(&mut app, key(KeyCode::Char('G')));
        assert_eq!(app.tab_manager().current_tab().scroll_offset(), 10); // 20 - 10
    }

    #[test]
    fn input_normal_mode_f_toggles_auto_scroll() {
        let mut app = App::new(vec!["cmd".into()], 100);
        assert!(app.tab_manager().current_tab().auto_scroll());

        handle_key(&mut app, key(KeyCode::Char('f')));
        assert!(!app.tab_manager().current_tab().auto_scroll());

        handle_key(&mut app, key(KeyCode::Char('f')));
        assert!(app.tab_manager().current_tab().auto_scroll());
    }

    #[test]
    fn input_normal_mode_slash_enters_search_mode() {
        let mut app = App::new(vec!["cmd".into()], 100);
        assert_eq!(app.mode(), Mode::Normal);

        handle_key(&mut app, key(KeyCode::Char('/')));
        assert_eq!(app.mode(), Mode::Search);
    }

    #[test]
    fn input_normal_mode_slash_disables_auto_scroll() {
        let mut app = App::new(vec!["cmd".into()], 100);
        assert!(app.tab_manager().current_tab().auto_scroll());

        handle_key(&mut app, key(KeyCode::Char('/')));
        assert!(!app.tab_manager().current_tab().auto_scroll());
    }

    #[test]
    fn input_normal_mode_slash_preserves_search_query() {
        let mut app = create_app_with_output();
        // Set up a previous search
        app.search_in_current_tab("line1");
        assert_eq!(app.search_state().query(), "line1");

        // Enter search mode
        handle_key(&mut app, key(KeyCode::Char('/')));

        // Query should be preserved
        assert_eq!(app.search_state().query(), "line1");
    }

    // Search mode tests

    #[test]
    fn input_search_mode_esc_returns_to_normal() {
        let mut app = App::new(vec!["cmd".into()], 100);
        app.set_mode(Mode::Search);

        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.mode(), Mode::Normal);
    }

    #[test]
    fn input_search_mode_char_updates_query() {
        let mut app = App::new(vec!["cmd".into()], 100);
        app.set_mode(Mode::Search);

        handle_key(&mut app, key(KeyCode::Char('h')));
        handle_key(&mut app, key(KeyCode::Char('e')));
        handle_key(&mut app, key(KeyCode::Char('l')));

        assert_eq!(app.search_state().query(), "hel");
    }

    #[test]
    fn input_search_mode_backspace_removes_char() {
        let mut app = App::new(vec!["cmd".into()], 100);
        app.set_mode(Mode::Search);

        handle_key(&mut app, key(KeyCode::Char('h')));
        handle_key(&mut app, key(KeyCode::Char('i')));
        assert_eq!(app.search_state().query(), "hi");

        handle_key(&mut app, key(KeyCode::Backspace));
        assert_eq!(app.search_state().query(), "h");
    }

    #[test]
    fn input_search_mode_n_adds_to_query() {
        let mut app = App::new(vec!["cmd".into()], 100);
        app.set_mode(Mode::Search);

        handle_key(&mut app, key(KeyCode::Char('n')));

        // n should be added to query, not trigger match navigation
        assert_eq!(app.search_state().query(), "n");
    }

    #[test]
    fn input_search_mode_upper_n_adds_to_query() {
        let mut app = App::new(vec!["cmd".into()], 100);
        app.set_mode(Mode::Search);

        handle_key(&mut app, key(KeyCode::Char('N')));

        // N should be added to query, not trigger match navigation
        assert_eq!(app.search_state().query(), "N");
    }

    #[test]
    fn input_search_mode_enter_returns_to_normal() {
        let mut app = App::new(vec!["cmd".into()], 100);
        app.set_mode(Mode::Search);

        handle_key(&mut app, key(KeyCode::Enter));

        assert_eq!(app.mode(), Mode::Normal);
    }

    #[test]
    fn input_normal_mode_n_moves_to_next_match_when_search_active() {
        let mut app = create_app_with_output();
        // Search for "line1" which appears in lines 1, 10, 11, etc.
        app.search_in_current_tab("line1");

        let initial_match = app.search_state().current_match_display();

        handle_key(&mut app, key(KeyCode::Char('n')));

        // Should have moved to next match
        let new_match = app.search_state().current_match_display();
        assert_ne!(initial_match, new_match);
    }

    #[test]
    fn input_normal_mode_upper_n_moves_to_prev_match_when_search_active() {
        let mut app = create_app_with_output();
        app.search_in_current_tab("line1");

        handle_key(&mut app, key(KeyCode::Char('N')));

        // Should wrap to last match
        assert!(app.search_state().current_match_display().is_some());
    }

    #[test]
    fn input_normal_mode_n_does_nothing_when_no_search() {
        let mut app = create_app_with_output();
        let initial_offset = app.tab_manager().current_tab().scroll_offset();

        handle_key(&mut app, key(KeyCode::Char('n')));

        // Should not change anything
        assert_eq!(
            app.tab_manager().current_tab().scroll_offset(),
            initial_offset
        );
    }

    // Emacs-like keybindings tests (via tui-input)

    #[test]
    fn input_search_mode_ctrl_h_deletes_char() {
        let mut app = App::new(vec!["cmd".into()], 100);
        app.set_mode(Mode::Search);

        // Type "hello"
        for c in "hello".chars() {
            handle_key(&mut app, key(KeyCode::Char(c)));
        }
        assert_eq!(app.search_state().query(), "hello");

        // Ctrl+H should delete last character
        handle_key(&mut app, key_with_ctrl('h'));
        assert_eq!(app.search_state().query(), "hell");
    }

    #[test]
    fn input_search_mode_ctrl_w_deletes_word() {
        let mut app = App::new(vec!["cmd".into()], 100);
        app.set_mode(Mode::Search);

        // Type "hello world"
        for c in "hello world".chars() {
            handle_key(&mut app, key(KeyCode::Char(c)));
        }
        assert_eq!(app.search_state().query(), "hello world");

        // Ctrl+W should delete "world"
        handle_key(&mut app, key_with_ctrl('w'));
        assert_eq!(app.search_state().query(), "hello ");
    }

    #[test]
    fn input_search_mode_ctrl_u_clears_line() {
        let mut app = App::new(vec!["cmd".into()], 100);
        app.set_mode(Mode::Search);

        // Type "hello world"
        for c in "hello world".chars() {
            handle_key(&mut app, key(KeyCode::Char(c)));
        }
        assert_eq!(app.search_state().query(), "hello world");

        // Ctrl+U should clear to beginning of line
        handle_key(&mut app, key_with_ctrl('u'));
        assert_eq!(app.search_state().query(), "");
    }
}
