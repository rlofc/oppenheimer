use crate::{
    commands::*,
    config::{BoardConfig, Styles},
    list::*,
};

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{List, ListItem, ListState, Paragraph},
};

#[derive(Clone, Default)]
pub struct Board {
    pub lists: Vec<BoardList>,
    pub current_list: Option<usize>,
    pub filter: String,
    pub config: BoardConfig,
    pub column_title_areas: Vec<Rect>,
    pub column_item_areas: Vec<Rect>,
    pub editing_item_index: Option<usize>,
}

impl Board {
    pub fn with_config(mut self, config: BoardConfig) -> Self {
        self.config = config;
        self
    }

    pub fn number_of_lists(&self) -> usize {
        self.lists.len()
    }
    pub fn move_down(&mut self) {
        if self.current_list.is_some() {
            self.lists[self.current_list.unwrap()].select_next()
        }
    }

    pub fn move_up(&mut self) {
        if self.current_list.is_some() {
            self.lists[self.current_list.unwrap()].select_previous()
        }
    }

    pub fn move_right(&mut self) {
        if let Some(current) = self.current_list {
            if current < self.lists.len() - 1 {
                self.lists[current].state.borrow_mut().select(None);
                self.current_list = Some(current + 1);
                self.lists[current + 1].set_selection();
            }
        } else if !self.lists.is_empty() {
            self.current_list = Some(0);
        }
    }

    pub fn move_left(&mut self) {
        if let Some(current) = self.current_list {
            if current > 0 {
                self.lists[current].state.borrow_mut().select(None);
                self.current_list = Some(current.saturating_sub(1));
                self.lists[self.current_list.unwrap()].set_selection();
            }
        } else if !self.lists.is_empty() {
            self.current_list = Some(0);
        }
    }

    fn render_items(&'_ self, list: usize) -> Vec<ListItem<'_>> {
        let is_dimmable = {
            let dim_tailing_items = self.config.dim_tailing_items;
            let focused_list = self.current_list.unwrap_or(list + 1) == list;
            let current_item = self
                .current_list()
                .and_then(|l| l.selected_item_index)
                .unwrap_or(usize::MAX);

            let list_exception = true;

            move |index: usize| {
                !(focused_list && index == current_item || !dim_tailing_items || !list_exception)
            }
        };

        let column_width = self.lists[list].width as usize;
        let max_index = if Some(list) == self.current_list {
            self.editing_item_index.unwrap_or(usize::MAX)
        } else {
            usize::MAX
        };

        self.lists[list]
            .items
            .iter()
            .enumerate()
            .filter(|(i, _)| *i <= max_index)
            .filter(|(_, i)| i.text.to_lowercase().contains(&self.filter.to_lowercase()))
            .map(|(index, i)| {
                i.render(index, column_width, is_dimmable(index), &self.config.styles)
            })
            .collect()
    }

    pub fn get_selection_bookmark(&self) -> SelectionBookmark {
        let list = self.current_list;
        let item = list.and_then(|l| self.lists[l].selected_item_index);

        SelectionBookmark { list, item }
    }

    pub fn get_current_selection_index(&self) -> usize {
        self.lists[self.current_list.unwrap()]
            .selected_item_index
            .unwrap_or(0)
    }

    pub fn current_list(&self) -> Option<&BoardList> {
        if let Some(current_list) = self.current_list {
            Some(&self.lists[current_list])
        } else {
            None
        }
    }

    pub fn current_list_mut(&mut self) -> Option<&mut BoardList> {
        if let Some(current_list) = self.current_list {
            Some(&mut self.lists[current_list])
        } else {
            None
        }
    }

    pub fn prioritize_selected_item(&mut self) -> Option<Box<dyn Command>> {
        if let Some(current_list_index) = self.current_list {
            let current_list = self.current_list_mut().unwrap();
            if let Some(selected_item_index) = current_list.selected_item_index {
                if selected_item_index > 0 {
                    let cmd = ShuffleItemCommand {
                        list: current_list_index,
                        from_index: selected_item_index,
                        to_index: selected_item_index - 1,
                        bookmark: self.get_selection_bookmark(),
                    };
                    return Some(Box::new(cmd));
                }
            }
        }
        None
    }

    pub fn deprioritize_selected_item(&mut self) -> Option<Box<dyn Command>> {
        if let Some(current_list_index) = self.current_list {
            let current_list = self.current_list_mut().unwrap();
            if let Some(selected_item_index) = current_list.selected_item_index {
                if selected_item_index < current_list.items.len() - 1 {
                    let cmd = ShuffleItemCommand {
                        list: current_list_index,
                        from_index: selected_item_index,
                        to_index: selected_item_index + 1,
                        bookmark: self.get_selection_bookmark(),
                    };
                    return Some(Box::new(cmd));
                }
            }
        }
        None
    }

    pub fn move_to_prev_list(&mut self, index: usize) -> Option<Box<dyn Command>> {
        if let Some(current_list_index) = self.current_list {
            if current_list_index > 0 {
                let current_list = self.current_list_mut().unwrap();
                if let Some(selected_item_index) = current_list.selected_item_index {
                    let cmd = MoveItemCommand {
                        from_list: current_list_index,
                        to_list: current_list_index - 1,
                        from_index: selected_item_index,
                        to_index: index,
                        bookmark: self.get_selection_bookmark(),
                    };
                    return Some(Box::new(cmd));
                }
            }
        }
        None
    }

    pub fn move_to_next_list(&mut self, target_index: usize) -> Option<Box<dyn Command>> {
        if let Some(current_list_index) = self.current_list {
            if current_list_index < self.lists.len() - 1 {
                let current_list = self.current_list_mut().unwrap();
                if let Some(selected_item_index) = current_list.selected_item_index {
                    let cmd = MoveItemCommand {
                        from_list: current_list_index,
                        to_list: current_list_index + 1,
                        from_index: selected_item_index,
                        to_index: target_index,
                        bookmark: self.get_selection_bookmark(),
                    };
                    return Some(Box::new(cmd));
                }
            }
        }
        None
    }

    pub fn insert_item_to_current_list(&mut self) -> Option<Box<dyn StagedCommand>> {
        self.current_list
            .map(|current_list| {
                let list = &mut self.lists[current_list];
                let pos = if let Some(index) = list.selected_item_index {
                    (index + 1).min(list.items.len())
                } else {
                    0
                };
                list.selected_item_index = Some(pos);
                list.items.insert(pos, BoardItem::default());
                list.state.borrow_mut().select_next();
                Box::new(AddItemCommand {
                    list: current_list,
                    item: pos,
                    value: BoardItem::new(""),
                    bookmark: self.get_selection_bookmark(),
                }) as Box<dyn StagedCommand>
            })
            .or_else(|| None)
    }

    pub fn delete_selected_item(&mut self) -> Option<Box<dyn Command>> {
        self.current_list
            .and_then(|current_list| {
                self.lists[current_list].selected_item_index.map(|pos| {
                    Box::new(DeleteItemCommand {
                        list: current_list,
                        item: pos,
                        value: BoardItem::new(""),
                        bookmark: self.get_selection_bookmark(),
                    }) as Box<dyn Command>
                })
            })
            .or_else(|| None)
    }

    pub fn cut_selected_item(&mut self) -> Option<Box<dyn Command>> {
        self.current_list
            .and_then(|current_list| {
                self.lists[current_list].selected_item_index.map(|pos| {
                    Box::new(CutItemCommand {
                        list: current_list,
                        item: pos,
                        value: BoardItem::new(""),
                        bookmark: self.get_selection_bookmark(),
                        last_clipboard: None,
                    }) as Box<dyn Command>
                })
            })
            .or_else(|| None)
    }

    pub fn yank_selected_item(&mut self) -> Option<Box<dyn Command>> {
        self.current_list
            .and_then(|current_list| {
                self.lists[current_list].selected_item_index.map(|pos| {
                    Box::new(YankItemCommand {
                        list: current_list,
                        item: pos,
                        value: BoardItem::new(""),
                        last_clipboard: None,
                    }) as Box<dyn Command>
                })
            })
            .or_else(|| None)
    }

    pub fn paste_item(&mut self) -> Option<Box<dyn Command>> {
        self.current_list
            .map(|current_list| {
                let list = &mut self.lists[current_list];
                let pos = if let Some(index) = list.selected_item_index {
                    (index + 1).min(list.items.len())
                } else {
                    0
                };
                list.selected_item_index = Some(pos);
                list.state.borrow_mut().select_next();
                Box::new(PasteItemCommand {
                    list: current_list,
                    item: pos,
                    bookmark: self.get_selection_bookmark(),
                }) as Box<dyn Command>
            })
            .or_else(|| None)
    }

    pub fn insert_list_to_board(&mut self) -> Option<Box<dyn StagedCommand>> {
        let pos = if let Some(current_list) = self.current_list {
            self.lists[current_list].state.borrow_mut().select(None);
            current_list + 1
        } else {
            self.lists.len()
        };
        self.lists.insert(pos, BoardList::default());
        self.current_list = Some(pos);
        Some(Box::new(AddListCommand {
            list: pos,
            title: String::new(),
            bookmark: self.get_selection_bookmark(),
        }))
    }

    pub fn delete_selected_list(&mut self) -> Option<Box<dyn Command>> {
        self.current_list
            .map(|current_list| {
                Box::new(DeleteListCommand {
                    list: current_list,
                    value: BoardList::default(),
                    bookmark: self.get_selection_bookmark(),
                }) as Box<dyn Command>
            })
            .or_else(|| None)
    }

    pub fn toggle_selected_item(&mut self) -> Option<Box<dyn Command>> {
        self.current_list
            .and_then(|current_list| {
                self.lists[current_list].selected_item_index.map(|pos| {
                    Box::new(ToggleItemCommand {
                        list: current_list,
                        item: pos,
                        bookmark: self.get_selection_bookmark(),
                    }) as Box<dyn Command>
                })
            })
            .or_else(|| None)
    }

    pub fn shuffle_list_forward(&mut self) -> Option<Box<dyn Command>> {
        if let Some(current_list) = self.current_list {
            if current_list > 0 {
                let cmd = ShuffleListCommand {
                    from_index: current_list,
                    to_index: current_list - 1,
                    bookmark: self.get_selection_bookmark(),
                };
                return Some(Box::new(cmd));
            }
        }
        None
    }

    pub fn shuffle_list_back(&mut self) -> Option<Box<dyn Command>> {
        if let Some(current_list) = self.current_list {
            if current_list < self.lists.len() - 1 {
                let cmd = ShuffleListCommand {
                    from_index: current_list,
                    to_index: current_list + 1,
                    bookmark: self.get_selection_bookmark(),
                };
                return Some(Box::new(cmd));
            }
        }
        None
    }

    pub fn edit_current_item(&mut self) -> Option<Box<dyn StagedCommand>> {
        if let Some(list) = self.current_list {
            if let Some(item) = self.lists[list].selected_item_index {
                return Some(Box::new(ChangeTextCommand {
                    list,
                    item,
                    old: self.current_raw_item_text().clone(),
                    new: self.current_raw_item_text().clone(),
                    bookmark: self.get_selection_bookmark(),
                }));
            }
        }
        None
    }

    pub fn current_raw_item_text(&self) -> &String {
        let item = self.lists[self.current_list.unwrap()]
            .selected_item_index
            .unwrap();
        &self.lists[self.current_list.unwrap()].items[item].text
    }

    pub fn title_edit_rect(&self) -> Rect {
        let list = self.current_list.unwrap();
        let area = self
            .column_title_areas
            .get(list + 1)
            .copied()
            .unwrap_or_default();
        Rect::new(area.x, area.y + 1, area.width, 1)
    }

    pub fn item_edit_rect(&self, textarea: &ratatui_textarea::TextArea) -> Rect {
        let list_idx = self.current_list.unwrap();
        let list = &self.lists[list_idx];
        let item_area = self
            .column_item_areas
            .get(list_idx + 1)
            .copied()
            .unwrap_or_default();
        let column_width = list.width as usize;

        let list_state = &list.state.borrow();
        let first_item_index = list_state.offset();
        let selected = list.selected_item_index.unwrap();

        let mut offset_y: u16 = 0;
        for j in first_item_index..selected {
            let text = &list.items[j].text;
            let (s, o) = textwrap::unfill(text);
            let wrapped_text = textwrap::wrap(
                &s,
                o.break_words(false)
                    .word_splitter(textwrap::WordSplitter::NoHyphenation)
                    .width(column_width - 1),
            );
            offset_y += wrapped_text.len() as u16 + 1;
        }

        let text = textarea.lines().join("\n");
        let (_, opts) = textwrap::unfill(&text);
        let wrapped = textwrap::wrap(
            &text,
            opts.break_words(false)
                .word_splitter(textwrap::WordSplitter::NoHyphenation)
                .width(column_width - 1),
        );
        let height = wrapped.len().max(1) as u16;

        Rect::new(
            item_area.x + 1,
            item_area.y + offset_y,
            column_width as u16 - 1,
            height,
        )
    }

    pub fn render_items_below_edit(&self, frame: &mut Frame, textarea_rect: Rect) {
        let list_idx = self.current_list.unwrap();
        let list = &self.lists[list_idx];
        let item_area = self
            .column_item_areas
            .get(list_idx + 1)
            .copied()
            .unwrap_or_default();
        let column_width = list.width as usize;
        let styles = &self.config.styles;

        let list_state = &list.state.borrow();
        let first_item_index = list_state.offset();
        let selected = list.selected_item_index.unwrap();

        let dim_tailing_items = self.config.dim_tailing_items;
        let focused_list = self.current_list.unwrap_or(list_idx + 1) == list_idx;
        let current_item = self
            .current_list()
            .and_then(|l| l.selected_item_index)
            .unwrap_or(usize::MAX);
        let list_exception = true;

        let is_dimmable = |index: usize| {
            !(focused_list && index == current_item || !dim_tailing_items || !list_exception)
        };

        let mut cursor_y = textarea_rect.y + textarea_rect.height;

        for j in (selected + 1)..list.items.len() {
            if j < first_item_index {
                continue;
            }
            let styled_text = list.items[j].styled_text(j, column_width, is_dimmable(j), styles);

            for line in styled_text.lines.iter() {
                if cursor_y >= item_area.y + item_area.height {
                    return;
                }
                let paragraph = Paragraph::new(line.clone());
                let rect = Rect::new(item_area.x, cursor_y, column_width as u16, 1);
                frame.render_widget(paragraph, rect);
                cursor_y += 1;
            }
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, rect: Rect) -> Rect {
        if self.lists.is_empty() {
            let layout = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ]);
            let [_, center, _] = layout.areas(rect);
            frame.render_widget(
                Paragraph::new("Press CTRL+O to add a new list")
                    .centered()
                    .reversed(),
                center,
            );
            return rect;
        }

        let constraints = std::iter::once(Constraint::Length(2))
            .chain((0..self.number_of_lists()).map(|_| Constraint::Length(50)))
            .collect::<Vec<_>>();
        let vertical_layout = Layout::horizontal(constraints).spacing(2);
        let horizontal_layout = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]);

        let [top, center] = horizontal_layout.areas(rect);
        let columns_top = vertical_layout.split(top);
        let columns_center = vertical_layout.split(center);

        self.column_title_areas = columns_top.to_vec();
        self.column_item_areas = columns_center.to_vec();

        for (i, list) in self.lists.iter().enumerate() {
            let col = columns_top[i + 1];
            let lines = vec![
                Line::raw(""),
                Line::from(list.name.clone().to_uppercase())
                    .fg(self.config.styles.header.fg)
                    .bg(self.config.styles.header.bg)
                    .bold()
                    .apply_if(self.current_list == Some(i), |l| {
                        l.underlined()
                            .bg(self.config.styles.active_header.bg)
                            .fg(self.config.styles.active_header.fg)
                    }),
            ];
            frame.render_widget(Paragraph::new(lines), col);
        }

        self.lists
            .iter_mut()
            .enumerate()
            .for_each(|(i, list)| list.width = self.column_item_areas[i + 1].width);

        for (i, list) in self.lists.iter().enumerate() {
            render_list(
                frame,
                self.column_item_areas[i + 1],
                &mut list.state.borrow_mut(),
                self.render_items(i),
                &self.config.styles,
            );
        }

        *self
            .column_item_areas
            .get(1)
            .unwrap_or(&self.column_item_areas[0])
    }
}

trait ApplyIf: Sized {
    fn apply_if<F: FnOnce(Self) -> Self>(self, condition: bool, f: F) -> Self;
}

impl ApplyIf for Line<'_> {
    fn apply_if<F: FnOnce(Self) -> Self>(self, condition: bool, f: F) -> Self {
        if condition { f(self) } else { self }
    }
}

pub fn render_list(
    frame: &mut Frame,
    area: Rect,
    list_state: &mut ListState,
    modded_items: Vec<ListItem>,
    styles: &Styles,
) {
    let selected_type = Style::default().bg(styles.selected.bg);
    let list = List::new(modded_items)
        .style(Color::White)
        .highlight_style(selected_type);
    frame.render_stateful_widget(list, area, list_state);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board_with_empty_lists() -> Board {
        Board {
            lists: vec![
                BoardList::default(),
                BoardList::default(),
                BoardList::default(),
            ],
            current_list: Some(0),
            ..Default::default()
        }
    }

    #[test]
    fn test_move_between_lists() {
        let mut board = board_with_empty_lists();
        board.move_left();
        assert_eq!(board.current_list, Some(0));
        board.move_right();
        assert_eq!(board.current_list, Some(1));
        board.move_right();
        assert_eq!(board.current_list, Some(2));
        board.move_right();
        assert_eq!(board.current_list, Some(2));
        board.move_left();
        assert_eq!(board.current_list, Some(1));
        board.move_left();
        assert_eq!(board.current_list, Some(0));
        board.move_left();
        assert_eq!(board.current_list, Some(0));
    }

    fn board_with_a_short_list() -> Board {
        Board {
            lists: vec![BoardList {
                items: vec![
                    BoardItem::new("item 1"),
                    BoardItem::new("item 2"),
                    BoardItem::new("item 3"),
                ],
                selected_item_index: Some(0),
                ..Default::default()
            }],
            current_list: Some(0),
            ..Default::default()
        }
    }

    impl<'a> Context<'a> {
        pub fn from_board(board: &'a mut Board) -> Self {
            Context {
                board,
                clipboard: None,
            }
        }
        pub fn with_clipboard(mut self, content: String) -> Self {
            self.clipboard = Some(content);
            self
        }
    }

    #[test]
    fn test_move_within_list() {
        let mut board = board_with_a_short_list();
        board.move_up();
        assert_eq!(board.get_current_selection_index(), 0);
        board.move_down();
        assert_eq!(board.get_current_selection_index(), 1);
        board.move_down();
        assert_eq!(board.get_current_selection_index(), 2);
        board.move_down();
        assert_eq!(board.get_current_selection_index(), 2);
        board.move_up();
        assert_eq!(board.get_current_selection_index(), 1);
        board.move_up();
        assert_eq!(board.get_current_selection_index(), 0);
        board.move_up();
        assert_eq!(board.get_current_selection_index(), 0);
    }

    #[test]
    fn test_prioritize() {
        let mut board = board_with_a_short_list();
        board.move_down();
        let cmd = board.prioritize_selected_item();
        cmd.unwrap().apply(&mut Context::from_board(&mut board));
        assert_eq!(board.get_current_selection_index(), 0);
        assert_eq!(
            board
                .current_list_mut()
                .unwrap()
                .get_selected_item_text()
                .unwrap(),
            "item 2"
        );
    }

    #[test]
    fn test_deprioritize() {
        let mut board = board_with_a_short_list();
        let cmd = board.deprioritize_selected_item();
        cmd.unwrap().apply(&mut Context::from_board(&mut board));
        assert_eq!(board.get_current_selection_index(), 1);
        assert_eq!(
            board
                .current_list_mut()
                .unwrap()
                .get_selected_item_text()
                .unwrap(),
            "item 1"
        );
    }

    fn boards_with_two_short_lists() -> Board {
        Board {
            lists: vec![
                BoardList {
                    name: "list 1".to_string(),
                    items: vec![
                        BoardItem::new("list 1 item 1"),
                        BoardItem::new("list 1 item 2"),
                        BoardItem::new("list 1 item 3"),
                    ],
                    selected_item_index: Some(0),
                    ..Default::default()
                },
                BoardList {
                    name: "list 2".to_string(),
                    items: vec![
                        BoardItem::new("list 2 item 1"),
                        BoardItem::new("list 2 item 2"),
                        BoardItem::new("list 2 item 3"),
                    ],
                    ..Default::default()
                },
            ],
            current_list: Some(0),
            ..Default::default()
        }
    }
    #[test]
    fn test_move_item_between_lists() {
        let mut board = boards_with_two_short_lists();
        let cmd = board.move_to_next_list(0);
        cmd.unwrap().apply(&mut Context::from_board(&mut board));
        assert_eq!(board.get_current_selection_index(), 0);
        assert_eq!(board.current_list_mut().unwrap().name, "list 2");
        assert_eq!(
            board
                .current_list_mut()
                .unwrap()
                .get_selected_item_text()
                .unwrap(),
            "list 1 item 1"
        );
        board.move_down();
        board.move_down();
        board.move_down();
        let cmd = board.move_to_prev_list(3);
        cmd.unwrap().apply(&mut Context::from_board(&mut board));
        assert_eq!(board.get_current_selection_index(), 2);
        assert_eq!(board.current_list_mut().unwrap().name, "list 1");
        assert_eq!(
            board
                .current_list_mut()
                .unwrap()
                .get_selected_item_text()
                .unwrap(),
            "list 2 item 3"
        );
    }

    #[test]
    fn test_insert_new_item() {
        let mut board = board_with_empty_lists();
        let cmd = board.insert_item_to_current_list();
        cmd.unwrap().finalize(&mut board);
        assert_eq!(board.get_current_selection_index(), 0);
        assert_eq!(board.current_list_mut().unwrap().items.len(), 1);
    }

    #[test]
    fn test_delete_item() {
        let mut board = board_with_a_short_list();
        let cmd = board.delete_selected_item();
        cmd.unwrap().apply(&mut Context::from_board(&mut board));
        assert_eq!(board.get_current_selection_index(), 0);
        assert_eq!(board.current_list_mut().unwrap().items.len(), 2);
    }

    #[test]
    fn test_yank_and_paste() {
        let mut board = boards_with_two_short_lists();
        let v = {
            let cmd = board.yank_selected_item();
            let mut context = Context::from_board(&mut board);
            cmd.unwrap().apply(&mut context);
            let clipboard_content = context.clipboard.unwrap();
            assert_eq!(clipboard_content, "list 1 item 1");
            clipboard_content
        };
        {
            board.move_right();
            let cmd = board.paste_item();
            cmd.unwrap()
                .apply(&mut Context::from_board(&mut board).with_clipboard(v));
        }
        assert_eq!(board.current_list_mut().unwrap().items.len(), 4);
    }

    #[test]
    fn test_shuffle_list_forward() {
        let mut board = board_with_empty_lists();
        board.current_list = Some(1);
        let cmd = board.shuffle_list_forward();
        assert!(cmd.is_some());
        cmd.unwrap().apply(&mut Context::from_board(&mut board));
        assert_eq!(board.current_list, Some(0));
    }

    #[test]
    fn test_shuffle_list_forward_at_start() {
        let mut board = board_with_empty_lists();
        board.current_list = Some(0);
        let cmd = board.shuffle_list_forward();
        assert!(cmd.is_none());
    }

    #[test]
    fn test_shuffle_list_back() {
        let mut board = board_with_empty_lists();
        board.current_list = Some(0);
        let cmd = board.shuffle_list_back();
        assert!(cmd.is_some());
        cmd.unwrap().apply(&mut Context::from_board(&mut board));
        assert_eq!(board.current_list, Some(1));
    }

    #[test]
    fn test_shuffle_list_back_at_end() {
        let mut board = board_with_empty_lists();
        board.current_list = Some(board.lists.len() - 1);
        let cmd = board.shuffle_list_back();
        assert!(cmd.is_none());
    }
}
