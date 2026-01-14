use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Mode};

/// Handle key event and update app state
pub fn handle_key(app: &mut App, key: KeyEvent) {
    match app.mode() {
        Mode::Normal => handle_normal_mode(app, key),
        Mode::Search => handle_search_mode(app, key),
    }
}

/// Handle key event in Normal mode
fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        // Quit
        KeyCode::Char('q') => app.quit(),

        // Tab navigation
        KeyCode::Char('h') => app.tab_manager_mut().prev_tab(),
        KeyCode::Char('l') => app.tab_manager_mut().next_tab(),

        // Scroll
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

        // Navigate matches
        KeyCode::Char('n') => {
            if let Some(line) = app.search_state_mut().next_match() {
                app.tab_manager_mut().current_tab_mut().scroll_to_line(line);
            }
        }
        KeyCode::Char('N') => {
            if let Some(line) = app.search_state_mut().prev_match() {
                app.tab_manager_mut().current_tab_mut().scroll_to_line(line);
            }
        }

        // Delete character from query
        KeyCode::Backspace => {
            let mut query = app.search_state().query().to_string();
            query.pop();
            app.search_in_current_tab(&query);
        }

        // Add character to query
        KeyCode::Char(c) => {
            let mut query = app.search_state().query().to_string();
            query.push(c);
            app.search_in_current_tab(&query);
        }

        _ => {}
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
    fn input_normal_mode_q_quits() {
        let mut app = App::new(vec!["cmd".into()], 100);
        assert!(!app.should_quit());

        handle_key(&mut app, key(KeyCode::Char('q')));
        assert!(app.should_quit());
    }

    #[test]
    fn input_normal_mode_h_switches_to_prev_tab() {
        let mut app = App::new(vec!["cmd1".into(), "cmd2".into()], 100);
        app.tab_manager_mut().next_tab(); // Move to tab 1
        assert_eq!(app.tab_manager().active_index(), 1);

        handle_key(&mut app, key(KeyCode::Char('h')));
        assert_eq!(app.tab_manager().active_index(), 0);
    }

    #[test]
    fn input_normal_mode_l_switches_to_next_tab() {
        let mut app = App::new(vec!["cmd1".into(), "cmd2".into()], 100);
        assert_eq!(app.tab_manager().active_index(), 0);

        handle_key(&mut app, key(KeyCode::Char('l')));
        assert_eq!(app.tab_manager().active_index(), 1);
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
    fn input_search_mode_n_moves_to_next_match() {
        let mut app = create_app_with_output();
        // Search for "line1" which appears in lines 1, 10, 11, etc.
        app.search_in_current_tab("line1");
        app.set_mode(Mode::Search);

        // First match is at line 1, next should go to line 10, 11, etc.
        let initial_match = app.search_state().current_match_display();

        handle_key(&mut app, key(KeyCode::Char('n')));

        // Should have moved to next match
        let new_match = app.search_state().current_match_display();
        assert_ne!(initial_match, new_match);
    }

    #[test]
    fn input_search_mode_upper_n_moves_to_prev_match() {
        let mut app = create_app_with_output();
        app.search_in_current_tab("line1");
        app.set_mode(Mode::Search);

        handle_key(&mut app, key(KeyCode::Char('N')));

        // Should wrap to last match
        assert!(app.search_state().current_match_display().is_some());
    }
}
