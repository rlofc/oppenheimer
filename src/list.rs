use std::cell::RefCell;

use crate::{
    config,
    input::{wrapping_presets, Editable},
};
use ratatui::{
    prelude::*,
    widgets::{ListItem, ListState},
};

#[derive(Clone, Default)]
pub struct BoardItem {
    pub text: String,
    pub done: bool,
    pub board: Option<usize>,
}

impl BoardItem {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            done: false,
            board: None,
        }
    }
    pub fn toggle(&mut self) {
        self.done = !self.done;
    }
    pub fn render(
        &self,
        index: usize,
        column_width: usize,
        is_dimmable: bool,
        styles: &config::Styles,
    ) -> ListItem {
        let mut text = Text::default();
        let (s, o) = textwrap::unfill(&self.text);
        let wrapped_text = textwrap::wrap(&s, wrapping_presets(o.width(column_width - 1)));
        for line_text in wrapped_text.iter() {
            let mut line = Line::default();
            if self.board.is_some() {
                line += "▍".fg(styles.fringe_on.fg).bg(styles.fringe_on.bg);
            } else {
                line += "▍".fg(styles.fringe_off.fg).bg(styles.fringe_off.bg);
            }
            let line_string = line_text.to_string();
            let mut action = String::new();
            let mut in_hash = false;

            for ch in line_string.chars() {
                if ch == '#' {
                    if !in_hash {
                        line += action.clone().fg(styles.item.fg).bg(styles.item.bg);
                        action.clear();
                        in_hash = true;
                    }
                    line += "#".fg(styles.tag_hashsign.fg).bg(styles.tag_hashsign.bg);
                } else if ch.is_whitespace() {
                    action.push(ch);
                    if in_hash {
                        line += action.clone().fg(styles.tag.fg).bg(styles.tag.bg);
                        action.clear();
                        in_hash = false;
                    }
                } else {
                    action.push(ch);
                }
            }
            if !action.is_empty() {
                if in_hash {
                    line += action.clone().fg(styles.tag.fg).bg(styles.tag.bg);
                } else {
                    line += action.clone().fg(styles.item.fg).bg(styles.item.bg);
                }
            }
            text.push_line(if self.done {
                line.dim().crossed_out()
            } else {
                line
            });
        }
        text.extend([""]);

        ListItem::new(if index > 0 && is_dimmable {
            text.dim()
        } else {
            text
        })
    }
}

impl Editable for BoardItem {
    fn editable_text(&self) -> &String {
        &self.text
    }
    fn editable_text_mut(&mut self) -> &mut String {
        &mut self.text
    }
    fn wrapped(&self, width: u16) -> String {
        let (s, o) = textwrap::unfill(&self.text);
        let wrapped_text = textwrap::wrap(&s, wrapping_presets(o.width(width as usize - 1)));
        wrapped_text.join("\n")
    }
}

#[derive(Default, Clone)]
pub struct BoardList {
    pub name: String,
    pub items: Vec<BoardItem>,
    pub state: RefCell<ListState>,
    pub selected_item_index: Option<usize>,
    pub width: u16,
    pub _color: Color,
}

impl Editable for BoardList {
    fn editable_text(&self) -> &String {
        &self.name
    }
    fn editable_text_mut(&mut self) -> &mut String {
        &mut self.name
    }
    fn wrapped(&self, _width: u16) -> String {
        self.name.clone()
    }
}

impl BoardList {
    pub fn remove_item(&mut self, index: usize) {
        self.items.remove(index);
        self.clear_selection();
    }
    pub fn select_previous(&mut self) {
        if let Some(selected_item_index) = self.selected_item_index {
            if selected_item_index > 0 {
                self.selected_item_index = Some((selected_item_index - 1).max(0))
            }
        }
        self.set_selection();
    }

    pub fn select_next(&mut self) {
        if let Some(selected_item_index) = self.selected_item_index {
            if selected_item_index < self.items.len() - 1 {
                self.selected_item_index = Some(selected_item_index + 1)
            }
        }
        self.set_selection();
    }

    pub fn set_selection_index(&mut self, index: usize) {
        self.selected_item_index = if self.items.is_empty() {
            None
        } else {
            Some(index.min(self.items.len().saturating_sub(1)))
        };
    }

    pub fn set_selection(&mut self) {
        if !self.items.is_empty() {
            if let Some(selected_item_index) = self.selected_item_index {
                self.state.borrow_mut().select(Some(selected_item_index))
            } else {
                self.selected_item_index = Some(0);
                self.state.borrow_mut().select(Some(0));
            }
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_item_index = None;
        self.state.borrow_mut().select(None);
    }

    pub fn get_selected_item_text(&self) -> Option<&str> {
        if !self.items.is_empty() {
            if let Some(selected_item_index) = self.selected_item_index {
                return Some(&self.items[selected_item_index].text);
            }
        }
        None
    }

    pub(crate) fn current_item(&self) -> Option<&BoardItem> {
        if !self.items.is_empty() {
            if let Some(selected_item_index) = self.selected_item_index {
                return Some(&self.items[selected_item_index]);
            }
        }
        None
    }
    pub(crate) fn current_item_mut(&mut self) -> Option<&mut BoardItem> {
        if !self.items.is_empty() {
            if let Some(selected_item_index) = self.selected_item_index {
                return Some(&mut self.items[selected_item_index]);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn test_remove_item() {
        let mut board_list = BoardList {
            name: String::from("Test Board"),
            items: vec![BoardItem::new("Item 1"), BoardItem::new("Item 2")],
            state: RefCell::new(ListState::default()),
            selected_item_index: Some(0),
            width: 10,
            _color: Color::default(),
        };

        board_list.remove_item(0);
        assert_eq!(board_list.items.len(), 1);
        assert_eq!(board_list.selected_item_index, None);
    }

    #[test]
    fn test_select_previous() {
        let mut board_list = BoardList {
            name: String::from("Test Board"),
            items: vec![BoardItem::new("Item 1"), BoardItem::new("Item 2")],
            state: RefCell::new(ListState::default()),
            selected_item_index: Some(1),
            width: 10,
            _color: Color::default(),
        };

        board_list.select_previous();
        assert_eq!(board_list.selected_item_index, Some(0));
    }

    #[test]
    fn test_select_next() {
        let mut board_list = BoardList {
            name: String::from("Test Board"),
            items: vec![BoardItem::new("Item 1"), BoardItem::new("Item 2")],
            state: RefCell::new(ListState::default()),
            selected_item_index: Some(0),
            width: 10,
            _color: Color::default(),
        };

        board_list.select_next();
        assert_eq!(board_list.selected_item_index, Some(1));
    }

    #[test]
    fn test_set_selection() {
        let mut board_list = BoardList {
            name: String::from("Test Board"),
            items: vec![BoardItem::new("Item 1"), BoardItem::new("Item 2")],
            state: RefCell::new(ListState::default()),
            selected_item_index: None,
            width: 10,
            _color: Color::default(),
        };

        board_list.set_selection();
        assert_eq!(board_list.selected_item_index, Some(0));
    }

    #[test]
    fn test_get_selected_item_text() {
        let board_list = BoardList {
            name: String::from("Test Board"),
            items: vec![BoardItem::new("Item 1"), BoardItem::new("Item 2")],
            state: RefCell::new(ListState::default()),
            selected_item_index: Some(1),
            width: 10,
            _color: Color::default(),
        };

        assert_eq!(board_list.get_selected_item_text(), Some("Item 2"));
    }

    #[test]
    fn test_current_item() {
        let board_list = BoardList {
            name: String::from("Test Board"),
            items: vec![BoardItem::new("Item 1"), BoardItem::new("Item 2")],
            state: RefCell::new(ListState::default()),
            selected_item_index: Some(1),
            width: 10,
            _color: Color::default(),
        };

        assert_eq!(
            board_list.current_item().map(|item| &item.text),
            Some(&"Item 2".to_string())
        );
    }

    #[test]
    fn test_current_item_mut() {
        let mut board_list = BoardList {
            name: String::from("Test Board"),
            items: vec![BoardItem::new("Item 1"), BoardItem::new("Item 2")],
            state: RefCell::new(ListState::default()),
            selected_item_index: Some(1),
            width: 10,
            _color: Color::default(),
        };

        if let Some(item) = board_list.current_item_mut() {
            item.text = String::from("Updated Item");
        }

        assert_eq!(board_list.items[1].text, "Updated Item");
    }
}
