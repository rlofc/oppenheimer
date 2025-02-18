use crate::{board::Board, BoardItem, BoardList};

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
}

impl Command for ChangeTextCommand {
    fn apply(&mut self, board: &mut Board) {
        board.lists[self.list].items[self.item].text = self.new.clone();
    }
    fn revert(&mut self, board: &mut Board) {
        board.lists[self.list].items[self.item].text = self.old.clone();
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
    }
    fn revert(&mut self, board: &mut Board) {
        board.lists.remove(self.list);
        board.current_list = None;
    }
}

pub struct DeleteListCommand {
    pub list: usize,
    pub value: BoardList,
}

impl Command for DeleteListCommand {
    fn apply(&mut self, board: &mut Board) {
        self.value = board.lists.remove(self.list);
        board.move_left();
    }
    fn revert(&mut self, board: &mut Board) {
        board.lists.insert(self.list, self.value.clone());
        if let Some(current_list) = board.current_list_mut() {
            current_list.clear_selection();
        }
        board.current_list = Some(self.list);
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
}

impl Command for AddItemCommand {
    fn apply(&mut self, board: &mut Board) {
        board.lists[self.list]
            .items
            .insert(self.item, self.value.clone());
    }
    fn revert(&mut self, board: &mut Board) {
        board.lists[self.list].items.remove(self.item);
        board.lists[self.list].set_selection_index(self.item - 1);
        board.lists[self.list].set_selection();
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
}

impl Command for DeleteItemCommand {
    fn apply(&mut self, board: &mut Board) {
        self.value = board.lists[self.list].items.remove(self.item);
        board.lists[self.list].set_selection_index(self.item - 1);
        board.lists[self.list].set_selection();
    }
    fn revert(&mut self, board: &mut Board) {
        board.lists[self.list]
            .items
            .insert(self.item, self.value.clone());
        board.lists[self.list].set_selection_index(self.item);
        board.lists[self.list].set_selection();
    }
}

pub struct ShuffleListCommand {
    pub from_index: usize,
    pub to_index: usize,
}

impl Command for ShuffleListCommand {
    fn apply(&mut self, board: &mut Board) {
        board.lists.swap(self.from_index, self.to_index);
        board.current_list = Some(self.to_index);
    }
    fn revert(&mut self, board: &mut Board) {
        board.lists.swap(self.to_index, self.from_index);
        board.current_list = Some(self.from_index);
    }
}

pub struct ShuffleItemCommand {
    pub list: usize,
    pub from_index: usize,
    pub to_index: usize,
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
        current_list.set_selection_index(self.from_index);
        current_list.set_selection();
    }
}

pub struct MoveItemCommand {
    pub from_list: usize,
    pub from_index: usize,
    pub to_list: usize,
    pub to_index: usize,
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
                current_list.set_selection_index(current_list.items.len() - 1);
            }
            self.to_index = current_list.selected_item_index.unwrap();
            current_list.set_selection();
        }
    }
    fn revert(&mut self, board: &mut Board) {
        let current_list = &mut board.lists[self.to_list];
        if let Some(item) = current_list.items.get(self.to_index).cloned() {
            current_list.remove_selected_item();
            board.current_list = Some(self.from_list);
            let current_list = &mut board.lists[self.from_list];
            current_list.items.insert(self.from_index, item);
            current_list.set_selection_index(self.from_index);
            current_list.set_selection();
        }
    }
}
