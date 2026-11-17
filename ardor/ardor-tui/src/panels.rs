//! Pure ratatui panel constructors (DESIGN §D2): widgets are
//! built from the shared screen lines, so the full-screen TUI
//! and line mode render identical content. Rendering is tested
//! headlessly through ratatui's `TestBackend`.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::facade::ArdorFacade;
use crate::screens::screen_lines;
use crate::{LINE_CHARS_MAX, TUI_VERSION, truncate_chars};

/// Build the text widget for one screen's lines.
pub fn lines_widget(lines: &[String], title: &str) -> Paragraph<'static> {
    let body: Vec<String> = lines
        .iter()
        .map(|line| truncate_chars(line, LINE_CHARS_MAX))
        .collect();
    Paragraph::new(body.join("\n"))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title.to_string()),
        )
        .wrap(Wrap { trim: false })
}

/// Render the whole console: header, current screen, footer.
pub fn render<F: ArdorFacade>(frame: &mut Frame, app: &App<F>) {
    let area = frame.area();
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(area);
    let header = Paragraph::new(format!(
        "ardor-tui {TUI_VERSION} — {} — mode {} — session {}",
        app.screen.label(),
        app.mode.label(),
        app.facade.session_id()
    ));
    frame.render_widget(header, chunks[0]);
    let lines = screen_lines(app);
    // Show the tail that fits the body height.
    let height = usize::from(chunks[1].height.saturating_sub(2));
    let start = lines.len().saturating_sub(height.max(1));
    frame.render_widget(lines_widget(&lines[start..], app.screen.label()), chunks[1]);
    let footer = Paragraph::new(
        "1-5 screens  tab next  j/k select  enter run  a approve  x abort  m mode  p project  r refresh  q quit",
    );
    frame.render_widget(footer, chunks[2]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facade::FakeFacade;
    use crate::triggers::{TriggerEngine, parse_rules};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use serde_json::json;

    fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
        let buffer = terminal.backend().buffer();
        let mut text = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                text.push_str(buffer[(x, y)].symbol());
            }
            text.push('\n');
        }
        text
    }

    #[test]
    fn first_screen_renders_headless() {
        let mut fake = FakeFacade::new();
        fake.script_status(Ok(json!({
            "server": "ardor", "server_version": "1.0.1", "protocol": "2026-07-28"
        })));
        fake.script_tools(Ok(json!({"tools": []})));
        fake.script_call("list_projects", Ok(json!({"projects": []})));
        let engine =
            TriggerEngine::new(parse_rules(crate::triggers::EMBEDDED_RULES).expect("rules"));
        let mut app = App::new(fake, engine);
        app.refresh_overview();
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal.draw(|frame| render(frame, &app)).expect("draw");
        let text = buffer_text(&terminal);
        assert!(text.contains("ardor-tui"), "{text}");
        assert!(text.contains("Overview"), "{text}");
        assert!(text.contains("mode plan"), "{text}");
    }

    #[test]
    fn every_screen_renders_headless() {
        let engine =
            TriggerEngine::new(parse_rules(crate::triggers::EMBEDDED_RULES).expect("rules"));
        let mut app = App::new(FakeFacade::new(), engine);
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).expect("terminal");
        for screen in crate::app::Screen::ALL {
            app.screen = screen;
            terminal.draw(|frame| render(frame, &app)).expect("draw");
            let text = buffer_text(&terminal);
            assert!(
                text.contains(screen.label()),
                "screen {}: {text}",
                screen.label()
            );
        }
    }

    #[test]
    fn lines_widget_truncates_long_lines() {
        let long = "x".repeat(500);
        let widget = lines_widget(&[long], "t");
        let backend = TestBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| frame.render_widget(widget, frame.area()))
            .expect("draw");
        let text = buffer_text(&terminal);
        assert!(text.contains('\u{2026}'), "{text}");
    }
}
