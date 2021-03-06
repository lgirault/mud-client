use std::io;

pub mod app;
pub mod app_events;
pub mod events;

use app::Message;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::symbols::DOT;
use tui::widgets::*;
use tui::{Frame, Terminal};

pub use app::{App, AppArea};

pub fn draw_app<B: Backend>(terminal: &mut Terminal<B>, app: &App) -> Result<(), io::Error> {
    terminal.draw(|mut f| {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());

        let left = chunks[0];
        let right = chunks[1];

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(95), Constraint::Percentage(5)].as_ref())
            .split(left);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(right);

        draw_main(&mut f, left_chunks[0], app);
        draw_input(&mut f, left_chunks[1], app);
        //        //draw_character_sheet
        draw_map(&mut f, right_chunks[0], app);
        draw_chat(&mut f, right_chunks[1], app);
    })
}

fn border_type(app: &App, area: AppArea) -> BorderType {
    if app.focused_area == area {
        return BorderType::Thick;
    } else {
        return BorderType::Plain;
    }
}

fn border_style(app: &App, area: AppArea) -> Style {
    if app.focused_area == area {
        return Style::default().fg(Color::Cyan);
    } else {
        return Style::default();
    }
}

fn block(app: &App, area: AppArea) -> Block<'static> {
    return Block::default()
        .border_type(border_type(app, area))
        .borders(Borders::ALL)
        .title(area.name())
        .border_style(border_style(app, area));
}

fn draw_main<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let t: Vec<Text> = app
        .messages
        .iter()
        .map(|msg| -> Text {
            let mut text: String;
            match msg {
                Message::UserInput(s) => {
                    text = String::new();
                    text.push_str("> ");
                    text.push_str(s.replace("\r\n", "\n").as_str());
                }
                Message::Network(s) => {
                    text = s.replace("\r\n", "\n"); //XXX TODO make if configurable
                }
            }
            Text::raw(text)
        })
        .collect();

    let w = Paragraph::new(t.iter())
        .raw(true)
        //.style(Style::default().fg(Color::Yellow))
        .block(block(app, AppArea::Main));

    f.render_widget(w, area);
}

fn draw_input<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let t = [Text::raw(&app.input)];
    let w = Paragraph::new(t.iter())
        .style(Style::default().fg(Color::Yellow))
        .block(block(app, AppArea::Input));
    f.render_widget(w, area);
}

fn draw_map<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let w = block(app, AppArea::Map);
    f.render_widget(w, area);
}

fn draw_chat<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let w = Tabs::default()
        .block(block(app, AppArea::Chat))
        .titles(&["Tab1", "Tab2", "Tab3", "Tab4"])
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow))
        .divider(DOT);

    f.render_widget(w, area)
}
