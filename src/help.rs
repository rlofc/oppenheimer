use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Clear, Padding, Paragraph},
    Frame,
};

pub fn show_help_popup(frame: &mut Frame) {
    let area = frame.area();

    fn center_horizontal(area: Rect, width: u16) -> Rect {
        let [area] = Layout::horizontal([Constraint::Length(width)])
            .flex(Flex::Center)
            .areas(area);
        area
    }
    fn center_vertical(area: Rect, height: u16) -> Rect {
        let [area] = Layout::vertical([Constraint::Length(height)])
            .flex(Flex::Center)
            .areas(area);
        area
    }
    let area = center_vertical(center_horizontal(area, 60), 30);

    let block = Block::default()
        .title("Help")
        .padding(Padding::new(2, 2, 1, 1))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White).bg(Color::Black));

    let keybindings = vec![
        ("o", "Add new item to the current list"),
        ("d", "Delete selected item"),
        ("Enter", "Edit current item"),
        ("Ctrl-o", "Add new list to the current board"),
        ("Ctrl-d", "Delete the current list"),
        ("Space", "Toggle current item strikethrough"),
        ("/", "Search for items"),
        ("Down or j", "Move down"),
        ("Up or k", "Move up"),
        ("Right or l", "Move right"),
        ("Left or h", "Move left"),
        ("Ctrl-h", "Move item to previous list"),
        ("Ctrl-l", "Move item to next list"),
        ("Ctrl-j", "Deprioritize selected item"),
        ("Ctrl-k", "Prioritize selected item"),
        ("Shift-h", "Shuffle list right"),
        ("Shift-l", "Shuffle list left"),
        ("Tab", "Navigate to the item child-board"),
        ("Esc", "Go back to the parent-board"),
        ("u", "Undo action"),
        ("r", "Redo action"),
        ("q", "Quit"),
    ];
    let mut items: Vec<ratatui::text::Line> = keybindings
        .iter()
        .map(|(key, action)| {
            ratatui::text::Line::from(vec![
                ratatui::text::Span::raw(*key).bold().yellow(),
                ratatui::text::Span::raw(" ".repeat(14 - key.len())),
                ratatui::text::Span::styled(*action, Style::new()),
            ])
        })
        .collect::<Vec<ratatui::text::Line>>();
    items.insert(
        0,
        ratatui::text::Line::from(vec![
            ratatui::text::Span::raw("key").bold().underlined(),
            ratatui::text::Span::raw("           "),
            ratatui::text::Span::raw("Action").bold().underlined(),
        ]),
    );

    let content = Paragraph::new(items)
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .block(block);

    let footer = Paragraph::new("Press any key to close").dark_gray();

    frame.render_widget(Clear, area);
    frame.render_widget(content, area);
    frame.render_widget(
        footer.alignment(Alignment::Center),
        Rect::new(area.x + 1, area.y + area.height - 2, area.width - 2, 1),
    );
}
