use crossterm::event::{KeyEvent, KeyModifiers};
use ratatui::crossterm::event::KeyCode;

pub trait Editable {
    fn editable_text(&self) -> &String;
    fn editable_text_mut(&mut self) -> &mut String;
    fn wrapped(&self, width: u16) -> String;
    fn is_word_start(&self, index: usize) -> bool {
        if index == 0 {
            return true;
        }
        let previous_char = self.editable_text().chars().nth(index - 1);
        let current_char = self.editable_text().chars().nth(index);
        match (previous_char, current_char) {
            (Some(prev), Some(curr)) => !prev.is_alphanumeric() && curr.is_alphanumeric(),
            (None, Some(curr)) => curr.is_alphanumeric(),
            _ => false,
        }
    }
    fn is_word_end(&self, index: usize) -> bool {
        if index == self.editable_text().len() {
            return true;
        }
        let previous_char = self.editable_text().chars().nth(index - 1);
        let current_char = self.editable_text().chars().nth(index);
        match (previous_char, current_char) {
            (Some(prev), Some(curr)) => prev.is_alphanumeric() && !curr.is_alphanumeric(),
            (Some(prev), None) => prev.is_alphanumeric(),
            _ => false,
        }
    }
}

#[derive(Default, Clone)]
pub struct InputController {
    pub character_index: usize,
}

#[derive(PartialEq)]
pub enum InputAction {
    Continue,
    Done,
    NewItem,
}

impl InputController {
    pub fn input<T: Editable>(
        &mut self,
        editable: &mut T,
        key: KeyEvent,
        width: u16,
    ) -> InputAction {
        if key.modifiers == KeyModifiers::CONTROL {
            match key.code {
                KeyCode::Left => self.move_cursor_word_left(editable),
                KeyCode::Right => self.move_cursor_word_right(editable),
                _ => {}
            };
        } else {
            match key.code {
                KeyCode::Char(to_insert) => self.enter_char(editable, to_insert),
                KeyCode::Backspace => self.delete_char_backwards(editable, width),
                KeyCode::Delete => self.delete_char(editable, width),
                KeyCode::Left => self.move_cursor_left(editable),
                KeyCode::Right => self.move_cursor_right(editable),
                KeyCode::Down => self.move_cursor_down(editable, width),
                KeyCode::Up => self.move_cursor_up(editable, width),
                KeyCode::Home => self.move_cursor_to_start(editable),
                KeyCode::End => self.move_cursor_to_end(editable),
                KeyCode::Esc => {
                    return InputAction::Done;
                }
                KeyCode::Enter => {
                    return InputAction::NewItem;
                }
                _ => {}
            };
        }
        InputAction::Continue
    }
    fn move_cursor_left<T: Editable>(&mut self, editable: &T) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(editable, cursor_moved_left);
    }

    fn move_cursor_right<T: Editable>(&mut self, editable: &T) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(editable, cursor_moved_right);
    }

    fn get_positioning_data<T: Editable>(
        &mut self,
        editable: &T,
        width: u16,
    ) -> (String, usize, usize, usize, usize, usize) {
        let wrapped_text = editable.wrapped(width);
        let mut line = 1;
        let mut index_in_line = 0;
        for i in 0..self.character_index {
            (line, index_in_line) = if wrapped_text.chars().nth(i).unwrap_or('\0') == '\n' {
                (line + 1, 0)
            } else {
                (line, index_in_line + 1)
            };
        }
        let current_line = line;
        let current_index_in_line = index_in_line;
        let last_i = self.character_index;
        (
            wrapped_text,
            last_i,
            line,
            index_in_line,
            current_line,
            current_index_in_line,
        )
    }

    fn move_cursor_down<T: Editable>(&mut self, editable: &T, width: u16) {
        let (
            wrapped_text,
            mut last_i,
            mut line,
            mut index_in_line,
            current_line,
            current_index_in_line,
        ) = self.get_positioning_data(editable, width);

        let mut resolved_index: Option<usize> = None;
        for i in self.character_index..wrapped_text.len() {
            last_i = i;
            (line, index_in_line) = if wrapped_text.chars().nth(i).unwrap_or('\0') == '\n' {
                (line + 1, 0)
            } else {
                (line, index_in_line + 1)
            };
            if line == current_line + 2 {
                last_i = i - 1;
                break;
            }
            if line == current_line + 1 && index_in_line == current_index_in_line {
                resolved_index = Some(i + 1);
                break;
            }
        }

        self.character_index = self.clamp_cursor(editable, resolved_index.unwrap_or(last_i));
    }

    fn move_cursor_up<T: Editable>(&mut self, editable: &T, width: u16) {
        let (
            wrapped_text,
            mut last_i,
            mut line,
            mut index_in_line,
            current_line,
            current_index_in_line,
        ) = self.get_positioning_data(editable, width);
        let x = wrapped_text
            .split("\n")
            .map(|line| line.to_string())
            .collect::<Vec<String>>();
        let mut resolved_index: Option<usize> = None;
        for i in (0..self.character_index).rev() {
            (line, index_in_line) = if wrapped_text.chars().nth(i).unwrap_or('\0') == '\n' {
                (line - 1, x[line - 2].len() - 1)
            } else {
                (line, index_in_line - 1)
            };
            if line == current_line - 2 {
                line += 1;
                last_i = i + x[line - 1].len();
                break;
            }
            if i == 0 {
                if current_line == 1 {
                    last_i = 0;
                } else {
                    last_i = i + x[line - 1].len() - 1;
                }
                break;
            }
            if line < current_line && index_in_line == current_index_in_line {
                resolved_index = Some(i - 1);
                break;
            }
        }

        self.character_index = self.clamp_cursor(editable, resolved_index.unwrap_or(last_i));
    }

    fn move_cursor_word_left<T: Editable>(&mut self, editable: &T) {
        let mut new_index = self.character_index.saturating_sub(1);
        while new_index > 0 && !editable.is_word_end(new_index) {
            new_index = new_index.saturating_sub(1);
        }
        self.character_index = self.clamp_cursor(editable, new_index.saturating_sub(1));
    }

    fn move_cursor_word_right<T: Editable>(&mut self, editable: &T) {
        let mut new_index = self.character_index + 1;
        let len = editable.editable_text().len();
        while new_index < len && !editable.is_word_start(new_index) {
            new_index = new_index.saturating_add(1);
        }
        self.character_index = self.clamp_cursor(editable, new_index);
    }

    fn move_cursor_to_start<T: Editable>(&mut self, editable: &T) {
        self.character_index = self.clamp_cursor(editable, 0);
    }

    fn move_cursor_to_end<T: Editable>(&mut self, editable: &T) {
        self.character_index = self.clamp_cursor(editable, editable.editable_text().len());
    }

    fn clamp_cursor<T: Editable>(&self, editable: &T, new_cursor_pos: usize) -> usize {
        let text = editable.editable_text();
        new_cursor_pos.clamp(0, text.chars().count())
    }

    fn enter_char<T: Editable>(&mut self, editable: &mut T, new_char: char) {
        let index = self.byte_index(editable);
        {
            let text = editable.editable_text_mut();
            text.insert(index, new_char);
        }
        self.move_cursor_right(editable);
    }
    fn byte_index<T: Editable>(&self, editable: &T) -> usize {
        let text = editable.editable_text();
        text.char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(text.len())
    }

    fn delete_char_backwards<T: Editable>(&mut self, editable: &mut T, width: u16) {
        let wrapped_text = editable.wrapped(width);

        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;
            let before_char_to_delete = wrapped_text.chars().take(from_left_to_current_index);
            let after_char_to_delete = wrapped_text.chars().skip(current_index);
            let text = editable.editable_text_mut();
            *text = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left(editable);
        }
    }

    fn delete_char<T: Editable>(&mut self, editable: &mut T, width: u16) {
        let wrapped_text = editable.wrapped(width);

        let is_not_cursor_rightmost = self.character_index < wrapped_text.chars().count();
        if is_not_cursor_rightmost {
            let current_index = self.character_index;
            let before_char_to_delete = wrapped_text.chars().take(current_index);
            let after_char_to_delete = wrapped_text.chars().skip(current_index + 1);
            let text = editable.editable_text_mut();
            *text = before_char_to_delete.chain(after_char_to_delete).collect();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent};

    struct MockEditable {
        text: String,
    }

    impl Editable for MockEditable {
        fn editable_text(&self) -> &String {
            &self.text
        }

        fn editable_text_mut(&mut self) -> &mut String {
            &mut self.text
        }

        fn wrapped(&self, _width: u16) -> String {
            self.text.clone()
        }
    }

    #[test]
    fn test_insert_character() {
        let mut controller = InputController::default();
        let mut editable = MockEditable {
            text: String::new(),
        };

        controller.input(&mut editable, KeyEvent::from(KeyCode::Char('a')), 80);
        assert_eq!(editable.text, "a");

        controller.input(&mut editable, KeyEvent::from(KeyCode::Char('b')), 80);
        assert_eq!(editable.text, "ab");
    }

    #[test]
    fn test_backspace_character() {
        let mut controller = InputController { character_index: 3 };
        let mut editable = MockEditable {
            text: String::from("abc"),
        };

        controller.input(&mut editable, KeyEvent::from(KeyCode::Left), 80);
        controller.input(&mut editable, KeyEvent::from(KeyCode::Left), 80);
        controller.input(&mut editable, KeyEvent::from(KeyCode::Backspace), 80);
        assert_eq!(editable.text, "bc");

        controller.input(&mut editable, KeyEvent::from(KeyCode::Right), 80);
        controller.input(&mut editable, KeyEvent::from(KeyCode::Backspace), 80);
        assert_eq!(editable.text, "c");

        controller.input(&mut editable, KeyEvent::from(KeyCode::Backspace), 80);
        assert_eq!(editable.text, "c"); // No change since cursor is at the beginning
    }

    #[test]
    fn test_delete_character() {
        let mut controller = InputController { character_index: 3 };
        let mut editable = MockEditable {
            text: String::from("abc"),
        };

        controller.input(&mut editable, KeyEvent::from(KeyCode::Left), 80);
        controller.input(&mut editable, KeyEvent::from(KeyCode::Left), 80);
        controller.input(&mut editable, KeyEvent::from(KeyCode::Delete), 80);
        assert_eq!(editable.text, "ac");

        controller.input(&mut editable, KeyEvent::from(KeyCode::Delete), 80);
        assert_eq!(editable.text, "a");

        controller.input(&mut editable, KeyEvent::from(KeyCode::Delete), 80);
        assert_eq!(editable.text, "a"); // No change since cursor is at the end

        controller.input(&mut editable, KeyEvent::from(KeyCode::Left), 80);
        controller.input(&mut editable, KeyEvent::from(KeyCode::Delete), 80);
        assert_eq!(editable.text, "");
    }

    #[test]
    fn test_move_cursor_left() {
        let mut controller = InputController { character_index: 3 };
        let mut editable = MockEditable {
            text: String::from("abc"),
        };

        controller.input(&mut editable, KeyEvent::from(KeyCode::Left), 80);
        assert_eq!(controller.character_index, 2);

        controller.input(&mut editable, KeyEvent::from(KeyCode::Left), 80);
        assert_eq!(controller.character_index, 1);

        controller.input(&mut editable, KeyEvent::from(KeyCode::Left), 80);
        assert_eq!(controller.character_index, 0);

        controller.input(&mut editable, KeyEvent::from(KeyCode::Left), 80);
        assert_eq!(controller.character_index, 0); // Cannot move left anymore
    }

    #[test]
    fn test_move_cursor_right() {
        let mut controller = InputController::default();
        let mut editable = MockEditable {
            text: String::from("abc"),
        };

        controller.input(&mut editable, KeyEvent::from(KeyCode::Right), 80);
        assert_eq!(controller.character_index, 1);

        controller.input(&mut editable, KeyEvent::from(KeyCode::Right), 80);
        assert_eq!(controller.character_index, 2);

        controller.input(&mut editable, KeyEvent::from(KeyCode::Right), 80);
        assert_eq!(controller.character_index, 3);

        controller.input(&mut editable, KeyEvent::from(KeyCode::Right), 80);
        assert_eq!(controller.character_index, 3); // Cannot move right anymore
    }

    #[test]
    fn test_move_cursor_up_and_down() {
        let mut controller = InputController::default();
        let mut editable = MockEditable {
            text: String::from("This is a wrapped\ntext\nwith three lines"),
        };

        controller.input(&mut editable, KeyEvent::from(KeyCode::Right), 80);
        controller.input(&mut editable, KeyEvent::from(KeyCode::Right), 80);
        controller.input(&mut editable, KeyEvent::from(KeyCode::Right), 80);
        controller.input(&mut editable, KeyEvent::from(KeyCode::Right), 80);
        controller.input(&mut editable, KeyEvent::from(KeyCode::Right), 80);
        controller.input(&mut editable, KeyEvent::from(KeyCode::Down), 80);
        assert_eq!(controller.character_index, 21);

        controller.input(&mut editable, KeyEvent::from(KeyCode::Down), 80);
        assert_eq!(controller.character_index, 26);

        controller.input(&mut editable, KeyEvent::from(KeyCode::Right), 80);
        controller.input(&mut editable, KeyEvent::from(KeyCode::Right), 80);
        controller.input(&mut editable, KeyEvent::from(KeyCode::Right), 80);
        controller.input(&mut editable, KeyEvent::from(KeyCode::Up), 80);
        assert_eq!(controller.character_index, 21);

        controller.input(&mut editable, KeyEvent::from(KeyCode::Down), 80);
        controller.input(&mut editable, KeyEvent::from(KeyCode::Down), 80);
        assert_eq!(controller.character_index, editable.text.len() - 1);

        controller.input(&mut editable, KeyEvent::from(KeyCode::Up), 80);
        controller.input(&mut editable, KeyEvent::from(KeyCode::Up), 80);
        controller.input(&mut editable, KeyEvent::from(KeyCode::Up), 80);
        assert_eq!(controller.character_index, 0);
    }

    #[test]
    fn test_escape_key() {
        let mut controller = InputController::default();
        let mut editable = MockEditable {
            text: String::from("abc"),
        };

        let continue_processing = controller.input(&mut editable, KeyEvent::from(KeyCode::Esc), 80);
        assert!(continue_processing == InputAction::Done);
    }

    #[test]
    fn test_move_cursor_to_start() {
        let mut controller = InputController { character_index: 3 };
        let mut editable = MockEditable {
            text: String::from("abc"),
        };

        controller.input(&mut editable, KeyEvent::from(KeyCode::Home), 80);
        assert_eq!(controller.character_index, 0);
    }

    #[test]
    fn test_move_cursor_to_end() {
        let mut controller = InputController::default();
        let mut editable = MockEditable {
            text: String::from("abc"),
        };

        controller.input(&mut editable, KeyEvent::from(KeyCode::End), 80);
        assert_eq!(controller.character_index, 3);
    }

    #[test]
    fn test_move_cursor_word_left() {
        let mut controller = InputController {
            character_index: 10,
        };
        let mut editable = MockEditable {
            text: String::from("hello world"),
        };

        controller.input(
            &mut editable,
            KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL),
            80,
        );
        assert_eq!(controller.character_index, 4);

        controller.input(
            &mut editable,
            KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL),
            80,
        );
        assert_eq!(controller.character_index, 0);
    }

    #[test]
    fn test_move_cursor_word_right() {
        let mut controller = InputController::default();
        let mut editable = MockEditable {
            text: String::from("hello world"),
        };

        controller.input(
            &mut editable,
            KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL),
            80,
        );
        assert_eq!(controller.character_index, 6);

        controller.input(
            &mut editable,
            KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL),
            80,
        );
        assert_eq!(controller.character_index, 11);
    }
}
