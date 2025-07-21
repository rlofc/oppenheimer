use crate::{commands::*, input::InputAction, list::*, InputController};

use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{List, ListItem, ListState, Paragraph},
    Frame,
};

#[derive(Default, Clone)]
pub struct Board {
    pub lists: Vec<BoardList>,
    pub current_list: Option<usize>,
    pub input_controller: InputController,
}

impl Board {
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

    fn render_items(&self, list: usize) -> Vec<ListItem> {
        let column_width = self.lists[list].width as usize;
        self.lists[list]
            .items
            .iter()
            .map(|i| i.render(column_width))
            .collect()
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
                    };
                    return Some(Box::new(cmd));
                }
            }
        }
        None
    }

    pub fn insert_item_to_current_list(&mut self) -> Option<Box<dyn StagedCommand>> {
        self.current_list.map(|current_list| {
            let list = &mut self.lists[current_list];
            let pos = list.selected_item_index.map_or(0, |index| index + 1);
            list.selected_item_index = Some(pos);
            list.items.insert(pos, BoardItem::default());
            list.state.borrow_mut().select_next();
            Box::new(AddItemCommand {
                list: current_list,
                item: pos,
                value: BoardItem::new(""),
            }) as Box<dyn StagedCommand>
        })
    }

    pub fn delete_selected_item(&mut self) -> Option<Box<dyn Command>> {
        self.current_list.and_then(|current_list| {
            self.lists[current_list].selected_item_index.map(|pos| {
                Box::new(DeleteItemCommand {
                    list: current_list,
                    item: pos,
                    value: BoardItem::new(""),
                }) as Box<dyn Command>
            })
        })
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
        }))
    }

    pub fn delete_selected_list(&mut self) -> Option<Box<dyn Command>> {
        self.current_list.map(|current_list| {
            Box::new(DeleteListCommand {
                list: current_list,
                value: BoardList::default(),
            }) as Box<dyn Command>
        })
    }

    pub fn shuffle_list_forward(&mut self) -> Option<Box<dyn Command>> {
        if let Some(current_list) = self.current_list {
            if current_list > 0 {
                let cmd = ShuffleListCommand {
                    from_index: current_list,
                    to_index: current_list - 1,
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
                }));
            }
        }
        None
    }

    pub fn process_input_for_title(&mut self, key: KeyEvent) -> InputAction {
        let width = self.lists[self.current_list.unwrap()].width;
        self.input_controller
            .input(&mut self.lists[self.current_list.unwrap()], key, width)
    }

    pub fn process_input_for_item(&mut self, key: KeyEvent) -> InputAction {
        let width = self.lists[self.current_list.unwrap()].width;
        let item = self.lists[self.current_list.unwrap()]
            .selected_item_index
            .unwrap();
        self.input_controller.input(
            &mut self.lists[self.current_list.unwrap()].items[item],
            key,
            width,
        )
    }

    pub fn current_raw_item_text(&self) -> &String {
        let item = self.lists[self.current_list.unwrap()]
            .selected_item_index
            .unwrap();
        &self.lists[self.current_list.unwrap()].items[item].text
    }

    fn current_item_wrapped_text(&self, trailing: usize) -> String {
        let column_width = self.lists[self.current_list.unwrap()].width;
        let mut result = String::new();
        if let Some(text) = self.current_list().unwrap().get_selected_item_text() {
            let (s, o) = textwrap::unfill(text);
            let wrapped_text = textwrap::wrap(&s, o.width(column_width as usize - 1));
            result = wrapped_text.join("\n");

            let mut last_width = wrapped_text.iter().last().unwrap().len();
            let mut trailing = trailing;
            while trailing > 0 {
                let spaces_to_add = (column_width as usize - last_width).min(trailing);
                result.push_str(&" ".repeat(spaces_to_add));
                trailing -= spaces_to_add;
                if trailing > 0 {
                    result.push('\n');
                    last_width = 0;
                }
            }
        }
        result
    }

    pub fn cursor_position_in_list_title(&self) -> (u16, u16) {
        let start_x: u16 = (0..self.current_list.unwrap())
            .map(|j| self.lists[j].width + 2)
            .sum();

        let wrapped_text = &self.lists[self.current_list.unwrap()].name;
        let (mut x, mut y) = (0, 1);

        for c in wrapped_text
            .chars()
            .take(self.input_controller.character_index)
        {
            if c == '\n' {
                y += 1;
                x = 0;
            } else {
                x += 1;
            }
        }
        (x as u16 + start_x, y as u16)
    }

    pub fn cursor_position_in_list_item(&self) -> (u16, u16) {
        let list_state = &self.lists[self.current_list.unwrap()].state.borrow();
        let column_width = self.lists[self.current_list.unwrap()].width as usize;
        let first_item_index = list_state.offset();
        let mut start_y = 0;
        for j in first_item_index
            ..self.lists[self.current_list.unwrap()]
                .selected_item_index
                .unwrap()
        {
            let text = &self.lists[self.current_list.unwrap()].items[j].text;
            let (s, o) = textwrap::unfill(text);
            let wrapped_text = textwrap::wrap(&s, o.width(column_width - 1));
            start_y += wrapped_text.len() + 1;
        }

        let start_x: u16 = self
            .lists
            .iter()
            .take(self.current_list.unwrap())
            .map(|list| list.width + 2)
            .sum();

        let original = self.current_raw_item_text();
        let missing = original.len() - original.trim_end().len();
        let wrapped_text = self.current_item_wrapped_text(missing);

        let mut x = 0;
        let mut y = 0;
        for (i, c) in wrapped_text.chars().enumerate() {
            if i == self.input_controller.character_index {
                break;
            }
            if c == '\n' {
                y += 1;
                x = 0;
            } else {
                x += 1;
            }
        }
        if x >= column_width - 1 {
            x = 0;
            y += 1;
        }
        (x as u16 + start_x + 1, y as u16 + start_y as u16)
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

        for (i, list) in self.lists.iter().enumerate() {
            let col = columns_top[i + 1];
            let lines = vec![
                Line::raw(""),
                Line::from(list.name.clone().to_uppercase())
                    .white()
                    .bold()
                    .apply_if(self.current_list == Some(i), |l| l.underlined()),
            ];
            frame.render_widget(Paragraph::new(lines), col);
        }

        let columns_center = vertical_layout.split(center);
        self.lists
            .iter_mut()
            .enumerate()
            .for_each(|(i, list)| list.width = columns_center[i + 1].width);

        for (i, list) in self.lists.iter().enumerate() {
            render_list(
                frame,
                columns_center[i + 1],
                &mut list.state.borrow_mut(),
                self.render_items(i),
            );
        }

        *columns_center.get(1).unwrap_or(&columns_center[0])
    }
}

trait ApplyIf: Sized {
    fn apply_if<F: FnOnce(Self) -> Self>(self, condition: bool, f: F) -> Self;
}

impl ApplyIf for Line<'_> {
    fn apply_if<F: FnOnce(Self) -> Self>(self, condition: bool, f: F) -> Self {
        if condition {
            f(self)
        } else {
            self
        }
    }
}

pub fn render_list(
    frame: &mut Frame,
    area: Rect,
    list_state: &mut ListState,
    modded_items: Vec<ListItem>,
) {
    let selected_type = Style::default().bg(Color::Black);
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
        cmd.unwrap().apply(&mut board);
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
        cmd.unwrap().apply(&mut board);
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
        cmd.unwrap().apply(&mut board);
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
        cmd.unwrap().apply(&mut board);
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
    fn test_shuffle_list_forward() {
        let mut board = board_with_empty_lists();
        board.current_list = Some(1);
        let cmd = board.shuffle_list_forward();
        assert!(cmd.is_some());
        cmd.unwrap().apply(&mut board);
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
        cmd.unwrap().apply(&mut board);
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
