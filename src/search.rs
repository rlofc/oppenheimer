use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Position, Rect},
    style::{Color, Stylize},
    Frame,
};

use crate::{
    board::Board,
    input::{Editable, InputController},
};

#[derive(Default)]
struct SearchToken {
    value: String,
}

impl Editable for SearchToken {
    fn editable_text(&self) -> &String {
        &self.value
    }
    fn editable_text_mut(&mut self) -> &mut String {
        &mut self.value
    }
    fn wrapped(&self, _width: u16) -> String {
        self.value.clone()
    }
}

#[derive(Default, Clone)]
pub struct FilteredBoardView {
    board_view: Vec<Vec<(usize, usize)>>,
}

impl FilteredBoardView {
    pub fn navigate_actual_board(&self, board: &mut Board, key: &KeyEvent) {
        match key.code {
            KeyCode::Down => {
                if let Some(current_list_index) = board.current_list {
                    if board.get_current_selection_index()
                        < self.view_items_in_list(current_list_index) - 1
                    {
                        board.move_down()
                    }
                }
            }
            KeyCode::Up => board.move_up(),
            KeyCode::Right => {
                if let Some(mut current_list_index) = board.current_list {
                    if self.view_has_items_right_of(current_list_index) {
                        board.move_right();
                        current_list_index += 1;
                    }
                    while self.view_list_on_right_is_empty(current_list_index) {
                        board.move_right();
                        current_list_index += 1;
                    }
                }
            }
            KeyCode::Left => {
                if let Some(mut current_list_index) = board.current_list {
                    if self.view_has_items_left_of(current_list_index) {
                        board.move_left();
                        current_list_index -= 1;
                    }

                    while self.view_list_on_left_is_empty(current_list_index) {
                        board.move_left();
                        current_list_index -= 1;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn select_item_from_view(&self, board: &mut Board) {
        if let Some(current_list_index) = board.current_list {
            let sel = board
                .get_current_selection_index()
                .min(self.view_items_in_list(current_list_index) - 1);

            let mapped = self.actual_index_for_reflected_item(current_list_index, sel);

            board.current_list_mut().unwrap().selected_item_index = Some(mapped);
            board.current_list_mut().unwrap().set_selection();
        }
    }

    pub fn update_view_selection(&self, board: &mut Board) {
        for list in board.lists.iter_mut() {
            list.clear_selection();
        }
        board.current_list = None;
        if let Some((list, item)) = self.available_view_selection() {
            board.current_list = Some(list);
            board.current_list_mut().unwrap().set_selection_index(item);
            board.current_list_mut().unwrap().set_selection();
        }
    }

    fn available_view_selection(&self) -> Option<(usize, usize)> {
        self.board_view
            .iter()
            .enumerate()
            .flat_map(|(list_index, list)| list.iter().map(move |item| (list_index, item.0)))
            .next()
    }
    fn view_has_items_left_of(&self, current_list_index: usize) -> bool {
        (0..current_list_index)
            .rev()
            .any(|l| !self.board_view[l].is_empty())
    }

    fn view_list_on_right_is_empty(&self, current_list_index: usize) -> bool {
        self.board_view[current_list_index].len() == 0
            && current_list_index < self.board_view.len() - 1
    }
    fn view_list_on_left_is_empty(&self, current_list_index: usize) -> bool {
        current_list_index > 0 && self.board_view[current_list_index].is_empty()
    }

    fn view_has_items_right_of(&self, current_list_index: usize) -> bool {
        (current_list_index + 1..self.board_view.len()).any(|l| !self.board_view[l].is_empty())
    }

    fn view_items_in_list(&self, current_list_index: usize) -> usize {
        self.board_view[current_list_index].len()
    }

    fn actual_index_for_reflected_item(
        &self,
        current_list_index: usize,
        view_index: usize,
    ) -> usize {
        self.board_view[current_list_index][view_index].1
    }
}

#[derive(Default)]
pub struct SearchController {
    token: SearchToken,
    input: InputController,
}

impl SearchController {
    pub fn draw(&self, frame: &mut Frame) {
        frame.set_cursor_position(Position::new(
            1 + self.input.character_index as u16,
            frame.area().height - 1,
        ));
        let widget = ratatui::widgets::Paragraph::new(format!("/{}", self.token.value.clone()))
            .bg(Color::DarkGray);
        frame.render_widget(
            widget,
            Rect::new(0, frame.area().height - 1, frame.area().width - 1, 1),
        );
    }

    pub fn reflect(&self, board: &Board) -> FilteredBoardView {
        let mut board_view: Vec<Vec<(usize, usize)>> = Vec::new();
        for l in board.lists.iter() {
            let mut list_view: Vec<(usize, usize)> = Vec::new();
            let mut partial_index: usize = 0;
            for (index, i) in l.items.iter().enumerate() {
                if i.text
                    .to_lowercase()
                    .contains(&self.token.value.to_lowercase())
                {
                    list_view.push((partial_index, index));
                    partial_index += 1;
                }
            }
            board_view.push(list_view);
        }
        FilteredBoardView { board_view }
    }

    pub fn input(&mut self, key: KeyEvent) -> String {
        self.input.input(&mut self.token, key, 60);
        self.token.value.clone()
    }

    pub fn clear(&mut self) {
        self.input.character_index = 0;
        self.token.value = "".to_string();
    }
}
