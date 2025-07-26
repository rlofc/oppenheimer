use crate::{board::Board, BoardItem, BoardList};

#[derive(Clone)]
pub struct SelectionBookmark {
    pub list: Option<usize>,
    pub item: Option<usize>,
}

impl SelectionBookmark {
    pub fn select(&self, board: &mut Board) {
        self.select_with_offset(board, 0);
    }
    pub fn select_with_offset(&self, board: &mut Board, offset: i32) {
        if let Some(list) = self.list {
            if list < board.lists.len() {
                board.current_list = Some(list);
                if let Some(item) = self.item {
                    if (item as i32 + offset) >= 0 {
                        let item = (item as i32 + offset) as usize;
                        if item < board.lists[list].items.len() {
                            board.lists[list].selected_item_index = Some(item);
                            board.lists[list].set_selection();
                        }
                    } else {
                        board.lists[list].selected_item_index = None;
                    }
                } else {
                    board.lists[list].selected_item_index = None;
                }
            } else {
                board.current_list = None;
            }
        }
    }
}

pub trait Command {
    fn apply(&mut self, board: &mut Board);
    fn revert(&mut self, board: &mut Board);
}

pub trait StagedCommand: Command {
    fn finalize(&mut self, board: &mut Board) -> bool;
    fn to_cmd(&self) -> Box<dyn Command>;
}

#[derive(Clone)]
pub struct ChangeTextCommand {
    pub list: usize,
    pub item: usize,
    pub old: String,
    pub new: String,
    pub bookmark: SelectionBookmark,
}

impl Command for ChangeTextCommand {
    fn apply(&mut self, board: &mut Board) {
        board.lists[self.list].items[self.item].text = self.new.clone();
        self.bookmark.select(board);
    }
    fn revert(&mut self, board: &mut Board) {
        board.lists[self.list].items[self.item].text = self.old.clone();
        self.bookmark.select(board);
    }
}

impl StagedCommand for ChangeTextCommand {
    fn finalize(&mut self, board: &mut Board) -> bool {
        self.new = board.lists[self.list].items[self.item].text.clone();
        self.new != self.old
    }
    fn to_cmd(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct AddListCommand {
    pub list: usize,
    pub title: String,
    pub bookmark: SelectionBookmark,
}

impl Command for AddListCommand {
    fn apply(&mut self, board: &mut Board) {
        board.lists.insert(
            self.list,
            BoardList {
                name: self.title.clone(),
                ..Default::default()
            },
        );
        self.bookmark.select(board);
    }
    fn revert(&mut self, board: &mut Board) {
        board.lists.remove(self.list);
        self.bookmark.select(board);
    }
}

pub struct DeleteListCommand {
    pub list: usize,
    pub value: BoardList,
    pub bookmark: SelectionBookmark,
}

impl Command for DeleteListCommand {
    fn apply(&mut self, board: &mut Board) {
        self.value = board.lists.remove(self.list);
        self.bookmark.select(board);
    }
    fn revert(&mut self, board: &mut Board) {
        board.lists.insert(self.list, self.value.clone());
        if let Some(current_list) = board.current_list_mut() {
            current_list.clear_selection();
        }
        // board.current_list = Some(self.list);
        self.bookmark.select(board);
    }
}

impl StagedCommand for AddListCommand {
    fn finalize(&mut self, board: &mut Board) -> bool {
        self.title = board.lists[self.list].name.clone();
        !self.title.is_empty()
    }
    fn to_cmd(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct AddItemCommand {
    pub list: usize,
    pub item: usize,
    pub value: BoardItem,
    pub bookmark: SelectionBookmark,
}

impl Command for AddItemCommand {
    fn apply(&mut self, board: &mut Board) {
        board.lists[self.list]
            .items
            .insert(self.item, self.value.clone());
        self.bookmark.select(board);
    }
    fn revert(&mut self, board: &mut Board) {
        board.lists[self.list].items.remove(self.item);
        self.bookmark.select_with_offset(board, -1);
    }
}

impl StagedCommand for AddItemCommand {
    fn finalize(&mut self, board: &mut Board) -> bool {
        self.value = board.lists[self.list].items[self.item].clone();
        !self.value.text.is_empty()
    }
    fn to_cmd(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

pub struct DeleteItemCommand {
    pub list: usize,
    pub item: usize,
    pub value: BoardItem,
    pub bookmark: SelectionBookmark,
}

impl Command for DeleteItemCommand {
    fn apply(&mut self, board: &mut Board) {
        self.value = board.lists[self.list].items.remove(self.item);
        board.lists[self.list].set_selection_index(self.item.saturating_sub(1));
        board.lists[self.list].set_selection();
    }
    fn revert(&mut self, board: &mut Board) {
        board.lists[self.list]
            .items
            .insert(self.item, self.value.clone());
        self.bookmark.select(board);
    }
}

pub struct ShuffleListCommand {
    pub from_index: usize,
    pub to_index: usize,
    pub bookmark: SelectionBookmark,
}

impl Command for ShuffleListCommand {
    fn apply(&mut self, board: &mut Board) {
        board.lists.swap(self.from_index, self.to_index);
        board.current_list = Some(self.to_index);
        self.bookmark.select(board);
    }
    fn revert(&mut self, board: &mut Board) {
        board.lists.swap(self.to_index, self.from_index);
        self.bookmark.select(board);
    }
}

pub struct ShuffleItemCommand {
    pub list: usize,
    pub from_index: usize,
    pub to_index: usize,
    pub bookmark: SelectionBookmark,
}

impl Command for ShuffleItemCommand {
    fn apply(&mut self, board: &mut Board) {
        let current_list = &mut board.lists[self.list];
        current_list.items.swap(self.from_index, self.to_index);
        current_list.set_selection_index(self.to_index);
        current_list.set_selection();
    }
    fn revert(&mut self, board: &mut Board) {
        let current_list = &mut board.lists[self.list];
        current_list.items.swap(self.to_index, self.from_index);
        self.bookmark.select(board);
    }
}

pub struct MoveItemCommand {
    pub from_list: usize,
    pub from_index: usize,
    pub to_list: usize,
    pub to_index: usize,
    pub bookmark: SelectionBookmark,
}

impl Command for MoveItemCommand {
    fn apply(&mut self, board: &mut Board) {
        let current_list = &mut board.lists[self.from_list];
        if let Some(item) = current_list.items.get(self.from_index).cloned() {
            current_list.remove_selected_item();
            board.current_list = Some(self.to_list);
            let current_list = &mut board.lists[self.to_list];
            if self.to_index < current_list.items.len() {
                current_list.items.insert(self.to_index, item);
                current_list.set_selection_index(self.to_index);
            } else {
                current_list.items.push(item);
                current_list.set_selection_index(current_list.items.len().saturating_sub(1));
            }
            self.to_index = current_list.selected_item_index.unwrap();
            current_list.set_selection();
        }
    }
    fn revert(&mut self, board: &mut Board) {
        let current_list = &mut board.lists[self.to_list];
        if let Some(item) = current_list.items.get(self.to_index).cloned() {
            current_list.items.remove(self.to_index);
            board.current_list = Some(self.from_list);
            let current_list = &mut board.lists[self.from_list];
            current_list.items.insert(self.from_index, item);
        }
        self.bookmark.select(board);
    }
}

pub struct ToggleItemCommand {
    pub list: usize,
    pub item: usize,
    pub bookmark: SelectionBookmark,
}

impl Command for ToggleItemCommand {
    fn apply(&mut self, board: &mut Board) {
        let current_list = &mut board.lists[self.list];
        current_list.items[self.item].toggle();
        self.bookmark.select(board);
    }
    fn revert(&mut self, board: &mut Board) {
        let current_list = &mut board.lists[self.list];
        current_list.items[self.item].toggle();
        self.bookmark.select(board);
    }
}
