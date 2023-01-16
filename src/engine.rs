use std::collections::VecDeque;

use crate::line_buffer::LineBuffer;

const HISTORY_SIZE: usize = 100;

pub enum EditCommand {
    MoveToStart,
    MoveToEnd,
    MoveLeft,
    MoveRight,
    MoveWordLeft,
    MoveWordRight,
    InsertChar(char),
    Backspace,
    Delete,
    AppendToHistory,
    PreviousHistory,
    NextHistory,
    Clear,
    CutToEnd,
    InsertCutBuffer,
}

pub struct Engine {
    line_buffer: LineBuffer,
    cut_buffer: String,
    history: VecDeque<String>,
    history_cursor: i64,
    pub has_history: bool,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            line_buffer: LineBuffer::new(),
            history: VecDeque::with_capacity(HISTORY_SIZE),
            history_cursor: 1i64,
            has_history: false,
            cut_buffer: String::new(),
        }
    }

    pub fn run_edit_commands(&mut self, commands: &[EditCommand]) {
        for command in commands {
            match command {
                EditCommand::MoveToStart => {
                    self.line_buffer.set_insertion_point(0);
                }
                EditCommand::MoveToEnd => todo!(),
                EditCommand::MoveLeft => self.line_buffer.decrement_insertion_point(),
                EditCommand::MoveRight => self.line_buffer.increment_insertion_point(),
                EditCommand::MoveWordLeft => {
                    self.line_buffer.move_word_left();
                }
                EditCommand::MoveWordRight => {
                    self.line_buffer.move_word_right();
                }
                EditCommand::InsertChar(c) => {
                    let insertion_point = self.line_buffer.get_insertion_point();
                    self.line_buffer.insert_char(insertion_point, *c);
                }
                EditCommand::Backspace => {
                    let insertion_point = self.get_insertion_point();
                    if insertion_point == self.get_buffer_length() && !self.is_empty() {
                        self.pop();
                    } else if insertion_point < self.get_buffer_length() && insertion_point > 0 {
                        self.decrement_insertion_point();
                        self.remove_char(self.get_insertion_point());
                    }
                }
                EditCommand::Delete => todo!(),
                EditCommand::AppendToHistory => {
                    if self.history.len() + 1 == HISTORY_SIZE {
                        self.history.pop_back();
                    }
                    if !self.line_buffer.get_buffer().is_empty() {
                        self.history.push_front(self.get_buffer().to_string());
                        self.has_history = true;
                        self.history_cursor = -1;
                    }
                }
                EditCommand::Clear => {
                    self.line_buffer.clear();
                    self.line_buffer.set_insertion_point(0);
                }
                EditCommand::PreviousHistory => {
                    if self.has_history && self.history_cursor < (self.history.len() as i64 - 1) {
                        self.history_cursor += 1;
                        let history_entry = self
                            .history
                            .get(self.history_cursor as usize)
                            .unwrap()
                            .clone();
                        self.set_buffer(history_entry);
                        self.line_buffer.move_to_end();
                    }
                }
                EditCommand::NextHistory => {
                    if self.history_cursor >= 0 {
                        self.history_cursor -= 1;
                    } else {
                        let new_buffer = if self.history_cursor < 0 {
                            String::new()
                        } else {
                            self.history
                                .get(self.history_cursor as usize)
                                .unwrap()
                                .clone()
                        };
                        self.set_buffer(new_buffer.clone());
                        self.move_to_end();
                    }
                }
                EditCommand::CutToEnd => {
                    let cut_slice = String::from(&self.get_buffer()[self.get_insertion_point()..]);
                    if !cut_slice.is_empty() {
                        self.cut_buffer.replace_range(.., &cut_slice);
                        self.clear_to_end(self.get_insertion_point());
                    }
                }
                EditCommand::InsertCutBuffer => {
                    self.insert_string(self.get_insertion_point(), &self.cut_buffer.to_string());
                    self.set_insertion_point(self.get_insertion_point() + self.cut_buffer.len());
                }
            }
        }
    }

    // fn get_grapheme_indices(&self) -> Vec<(usize, &str)> {
    //     UnicodeSegmentation::grapheme_indices(self.line_buffer.get_buffer(), true).collect()
    // }

    pub fn move_to_end(&mut self) {
        self.line_buffer.move_to_end();
    }

    pub fn set_buffer(&mut self, buffer: String) {
        self.line_buffer.set_buffer(buffer);
    }

    pub fn set_insertion_point(&mut self, pos: usize) {
        self.line_buffer.set_insertion_point(pos);
    }

    pub fn get_insertion_point(&self) -> usize {
        self.line_buffer.get_insertion_point()
    }

    pub fn get_buffer_length(&self) -> usize {
        self.line_buffer.get_buffer_length()
    }

    pub fn get_buffer(&self) -> &str {
        self.line_buffer.get_buffer()
    }

    pub fn increment_insertion_point(&mut self) {
        self.line_buffer.increment_insertion_point();
    }

    pub fn decrement_insertion_point(&mut self) {
        self.line_buffer.decrement_insertion_point();
    }

    pub fn insert_char(&mut self, pos: usize, c: char) {
        self.line_buffer.insert_char(pos, c);
    }

    pub fn insert_string(&mut self, pos: usize, s: &str) {
        self.line_buffer.insert_string(pos, s);
    }

    pub fn remove_char(&mut self, pos: usize) {
        self.line_buffer.remove_char(pos);
    }

    pub fn is_empty(&self) -> bool {
        self.line_buffer.is_empty()
    }

    pub fn pop(&mut self) -> Option<char> {
        self.line_buffer.pop()
    }

    pub fn clear_buffer(&mut self) {
        self.line_buffer.clear_buffer();
    }

    pub fn clear_to_end(&mut self, pos: usize) {
        self.line_buffer.clear_to_end(pos);
    }

    pub fn move_word_left(&mut self) -> usize {
        self.line_buffer.move_word_left()
    }

    pub fn move_word_right(&mut self) -> usize {
        self.line_buffer.move_word_right()
    }

    pub fn clear(&mut self) {
        self.line_buffer.clear();
    }
}
