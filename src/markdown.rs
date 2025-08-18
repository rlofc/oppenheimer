use std::fs::File;
use std::io::Write;
use std::{fs, path::Path};

use crate::{App, Board, BoardItem, BoardList};

impl App {
    pub fn load_sub_board(&mut self, li: &markdown::mdast::ListItem, board: usize) {
        for board_parent in li.children.iter() {
            if let markdown::mdast::Node::List(board_children) = board_parent {
                for board_list in board_children.children.iter() {
                    let mut board_list_iterator = match board_list.children().iter().next() {
                        Some(child) => child.iter(),
                        None => {
                            self.quick_on_loading_error();
                            return;
                        }
                    };
                    if let Some(list_name) = board_list_iterator.next() {
                        self.load_list_name(board, list_name);
                    } else {
                        self.quick_on_loading_error();
                    }

                    if let Some(markdown::mdast::Node::List(item_list)) = board_list_iterator.next()
                    {
                        for list_item in item_list.children.iter() {
                            self.load_list_item(board, list_item);
                        }
                    }
                }
            }
        }
    }
    pub fn load_list_name(&mut self, board: usize, list_node: &markdown::mdast::Node) {
        if let markdown::mdast::Node::Paragraph(paragraph_node) = list_node {
            if let Some(markdown::mdast::Node::Text(text_node)) = paragraph_node.children.first() {
                self.boards[board].lists.push(BoardList {
                    name: text_node.value.clone(),
                    ..Default::default()
                });
            }
        }
    }

    pub fn load_list_item(&mut self, board: usize, list_item: &markdown::mdast::Node) {
        let is_checked = if let markdown::mdast::Node::ListItem(list_item_node) = list_item.clone()
        {
            list_item_node.checked.unwrap_or(false)
        } else {
            false
        };

        let has_sub_board =
            if let markdown::mdast::Node::ListItem(list_item_node) = list_item.clone() {
                list_item_node.children.len() > 1
            } else {
                false
            };

        if let Some(Some(markdown::mdast::Node::Paragraph(paragraph_node))) =
            list_item.children().iter().next().map(|n| n.iter().next())
        {
            for text_node in
                paragraph_node
                    .children
                    .iter()
                    .filter_map(|child_node| match child_node {
                        markdown::mdast::Node::Text(text_node) => Some(text_node),
                        _ => None,
                    })
            {
                self.boards[board]
                    .lists
                    .iter_mut()
                    .last()
                    .unwrap()
                    .items
                    .push(BoardItem {
                        text: text_node.value.clone(),
                        done: is_checked,
                        board: None,
                    });
                if has_sub_board {
                    if let markdown::mdast::Node::ListItem(list_item_node) = list_item.clone() {
                        self.boards.push(Board::default());

                        self.boards[board]
                            .lists
                            .iter_mut()
                            .last()
                            .unwrap()
                            .items
                            .iter_mut()
                            .last()
                            .unwrap()
                            .board = Some(self.boards.len() - 1);
                        self.load_sub_board(&list_item_node, self.boards.len() - 1);
                    }
                }
            }
        }
    }

    pub fn load_md(&mut self, filename: &Path, board: usize) {
        use std::fs;
        let markdown_content =
            fs::read_to_string(filename.to_str().unwrap()).expect("Unable to read file");
        let x = markdown::to_mdast(
            &markdown_content,
            &markdown::ParseOptions {
                constructs: markdown::Constructs {
                    gfm_task_list_item: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();
        let mut lists: Vec<BoardList> = Vec::new();
        for e in x.children().iter().next().unwrap().iter() {
            match e {
                markdown::mdast::Node::List(l) => {
                    for i in l.children.iter() {
                        let checked = if let markdown::mdast::Node::ListItem(li) = i.clone() {
                            li.checked.unwrap_or(false)
                        } else {
                            false
                        };
                        for p in i.children().iter().next().unwrap().iter() {
                            if let markdown::mdast::Node::Paragraph(p) = p {
                                for t in p.children.iter() {
                                    if let markdown::mdast::Node::Text(t) = t {
                                        if let Some(list) = lists.iter_mut().last() {
                                            list.items.push(BoardItem {
                                                text: t.value.clone(),
                                                done: checked,
                                                board: None,
                                            });
                                            if let markdown::mdast::Node::ListItem(li) = i.clone() {
                                                if li.children.len() > 1 {
                                                    self.boards.push(Board::default());
                                                    list.items.iter_mut().last().unwrap().board =
                                                        Some(self.boards.len() - 1);
                                                    self.load_sub_board(&li, self.boards.len() - 1);
                                                }
                                            }
                                        } else {
                                            self.quick_on_loading_error();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                markdown::mdast::Node::Heading(h) => {
                    if h.depth == 2 {
                        if let Some(markdown::mdast::Node::Text(name)) = h.children.first() {
                            lists.push(BoardList {
                                name: name.value.clone(),
                                ..Default::default()
                            });
                        } else {
                            self.quick_on_loading_error();
                        }
                    }
                }
                _ => {}
            }
        }
        self.boards[board].lists = lists;
    }

    pub fn write_sub_board(&self, file: &mut File, board: usize, level: usize) {
        let board = &self.boards[board];
        for list in board.lists.iter() {
            writeln!(file, "{}- {}", " ".repeat(level * 2), list.name)
                .expect("Unable to write to file");
            for item in list.items.iter() {
                let checkmark = if item.done { "[x]" } else { "[ ]" };
                writeln!(
                    file,
                    "{}- {} {}",
                    " ".repeat((level * 2) + 2),
                    checkmark,
                    item.text.replace('\n', " ")
                )
                .expect("Unable to write to file");
                if let Some(board_index) = item.board {
                    self.write_sub_board(file, board_index, level + 2);
                }
            }
        }
    }

    pub fn write_md(&self, filename: &Path) {
        let board = &self.boards[0];
        let temp_file_path = format!("{}.tmp", filename.display());
        let mut file = File::create(&temp_file_path).expect("Unable to create temporary file");
        writeln!(file, "# Project Name").expect("Unable to write to file");
        for list in board.lists.iter() {
            writeln!(file, "## {}", list.name).expect("Unable to write to file");
            for item in list.items.iter() {
                let checkmark = if item.done { "[x]" } else { "[ ]" };
                writeln!(file, "- {} {}", checkmark, item.text.replace('\n', " "))
                    .expect("Unable to write to file");
                if let Some(board_index) = item.board {
                    self.write_sub_board(&mut file, board_index, 1);
                }
            }
        }
        fs::rename(temp_file_path, filename.to_str().unwrap())
            .expect("Unable to rename temporary file");
    }

    fn quick_on_loading_error(&self) {
        ratatui::restore();
        eprintln!("Error: file content is not an oppenheimer-compatible markdown");
        std::process::exit(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn setup_test_environment() -> App {
        App {
            boards: vec![Board {
                lists: vec![],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[test]
    fn test_load_md() {
        let mut app = setup_test_environment();

        let markdown_content = r#"
# Project Name
## To Do
- [ ] Task 1
- [x] Task 2 with sub-items
  - SubList
    - [x] Sub-task 1
    - [ ] Sub-task 2
## Done
- [x] Completed Task
"#;

        let temp_file_path = PathBuf::from("test_load_md.md");
        fs::write(&temp_file_path, markdown_content).expect("Unable to write test markdown file");

        app.load_md(&temp_file_path, 0);

        assert_eq!(app.boards[0].lists.len(), 2);
        assert_eq!(app.boards[0].lists[0].name, "To Do");
        assert_eq!(app.boards[0].lists[1].name, "Done");

        assert_eq!(app.boards[0].lists[0].items.len(), 2);
        assert_eq!(app.boards[0].lists[0].items[0].text, "Task 1");
        assert!(!app.boards[0].lists[0].items[0].done);
        assert_eq!(
            app.boards[0].lists[0].items[1].text,
            "Task 2 with sub-items"
        );
        assert!(app.boards[0].lists[0].items[1].done);

        assert!(app.boards[0].lists[0].items[1].board.is_some());
        let sub_board_index = app.boards[0].lists[0].items[1].board.unwrap();
        assert_eq!(app.boards[sub_board_index].lists.len(), 1);
        assert_eq!(app.boards[sub_board_index].lists[0].items.len(), 2);

        fs::remove_file(&temp_file_path).expect("Unable to remove test markdown file");
    }

    #[test]
    fn test_write_md() {
        let mut app = setup_test_environment();

        app.boards[0].lists.push(BoardList {
            name: "To Do".to_string(),
            items: vec![
                BoardItem {
                    text: "Task 1".to_string(),
                    done: false,
                    board: None,
                },
                BoardItem {
                    text: "Task 2 with sub-items".to_string(),
                    done: true,
                    board: Some(1),
                },
            ],
            ..Default::default()
        });

        app.boards.push(Board {
            lists: vec![BoardList {
                name: "Sub-tasks".to_string(),
                items: vec![
                    BoardItem {
                        text: "Sub-task 1".to_string(),
                        done: true,
                        board: None,
                    },
                    BoardItem {
                        text: "Sub-task 2".to_string(),
                        done: false,
                        board: None,
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        });

        let temp_file_path = PathBuf::from("test_write_md.md");
        app.write_md(&temp_file_path);

        let written_content =
            fs::read_to_string(&temp_file_path).expect("Unable to read written test markdown file");
        let expected_content = r#"# Project Name
## To Do
- [ ] Task 1
- [x] Task 2 with sub-items
  - Sub-tasks
    - [x] Sub-task 1
    - [ ] Sub-task 2
"#;

        assert_eq!(written_content, expected_content);

        fs::remove_file(&temp_file_path).expect("Unable to remove written test markdown file");
    }
}
