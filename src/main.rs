use crossterm::{
    cursor::{position, MoveLeft, MoveRight, MoveToColumn, MoveToNextLine},
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self},
    ExecutableCommand, QueueableCommand, Result,
};
use std::io::{stdout, Stdout, Write};

struct LineBuffer {
    buffer: String,
    insertion_point: usize,
}

impl LineBuffer {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            insertion_point: 0,
        }
    }

    pub fn set_caret_pos(&mut self, pos: u16) {
        self.insertion_point = pos;
    }

    pub fn get_caret_pos(&self) -> u16 {
        self.insertion_point
    }

    pub fn get_buffer_length(&self) -> usize {
        self.buffer.len()
    }

    pub fn get_buffer(&self) -> &str {
        &self.buffer
    }

    pub fn slice_buffer(&self, pos: usize) -> &str {
        &self.buffer[pos..]
    }

    pub fn increment_caret_pos(&mut self) {
        self.insertion_point += 1;
    }

    pub fn decrement_caret_pos(&mut self) {
        self.insertion_point += 1;
    }

    pub fn insert_char(&mut self, pos: usize, c: char) {
        self.buffer.insert(pos, c);
    }

    pub fn remove_char(&mut self, pos: usize) {
        self.buffer.remove(pos);
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn pop(&mut self) -> Option<char> {
        self.buffer.pop()
    }

    pub fn calcuate_word_left(&mut self, input_start_col: u16) -> Option<(usize, &str)> {
        self.buffer
            .rmatch_indices(&[' ', '\t'][..])
            .find(|(index, _)| index < &(self.insertion_point as usize - input_start_col as usize))
    }

    pub fn calcuate_word_right(&mut self, input_start_col: u16) -> Option<(usize, &str)> {
        self.buffer
            .match_indices(&[' ', '\t'][..])
            .find(|(index, _)| index < &(self.insertion_point as usize - input_start_col as usize))
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

fn print_message(stdout: &mut Stdout, msg: &str) -> Result<()> {
    stdout
        .queue(Print("\n"))?
        .queue(MoveToColumn(0))?
        .queue(Print(msg))?
        .queue(Print("\n"))?
        .queue(MoveToColumn(0))?;
    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let mut stdout = stdout();
    let mut buffer = LineBuffer::new();
    terminal::enable_raw_mode()?;

    'outer: loop {
        // print out prompt
        stdout
            .execute(SetForegroundColor(Color::Blue))?
            .execute(Print("> "))?
            .execute(ResetColor)?;

        let (input_start_col, _) = position()?;
        buffer.set_caret_pos(input_start_col);

        'input: loop {
            match read()? {
                Event::Key(KeyEvent {
                    code, modifiers, ..
                }) => match code {
                    KeyCode::Char(c) => {
                        if modifiers == KeyModifiers::CONTROL && (c == 'd' || c == 'c') {
                            stdout
                                .queue(MoveToNextLine(1))?
                                .queue(Print("Exiting..."))?;
                            break 'outer;
                        }
                        let insertion_point = buffer.get_caret_pos() - input_start_col;
                        stdout
                            .queue(Print(c))?
                            .queue(Print(buffer.slice_buffer(insertion_point.into())))?
                            .queue(MoveToColumn(buffer.get_caret_pos() + 1))?;
                        stdout.flush()?;
                        buffer.increment_caret_pos();
                        buffer.insert_char(insertion_point.into(), c);
                    }
                    KeyCode::Backspace => {
                        let insertion_point: usize =
                            (buffer.get_caret_pos() - input_start_col).into();
                        if insertion_point == buffer.get_buffer_length() && !buffer.is_empty() {
                            buffer.pop();
                            stdout
                                .queue(MoveLeft(1))?
                                .queue(Print(" "))?
                                .queue(MoveLeft(1))?;
                        } else if insertion_point < buffer.get_buffer_length() && !buffer.is_empty()
                        {
                            buffer.remove_char(insertion_point - 1);
                            stdout
                                .queue(MoveLeft(1))?
                                .queue(Print(buffer.slice_buffer(insertion_point - 1)))?
                                .queue(Print(" "))?
                                .queue(MoveToColumn(buffer.get_caret_pos() - 1))?;
                        }
                        stdout.flush()?;
                        buffer.decrement_caret_pos();
                    }
                    KeyCode::Enter => {
                        if buffer.get_buffer() == "exit" {
                            break 'outer;
                        } else {
                            print_message(
                                &mut stdout,
                                &format!("Buffer: {}", buffer.get_buffer()),
                            )?;
                            buffer.clear();
                            break 'input;
                        }
                    }
                    KeyCode::Left => {
                        if buffer.get_caret_pos() > input_start_col {
                            if modifiers == KeyModifiers::ALT {
                                let whitespace_index = buffer.calcuate_word_left(input_start_col);
                                match whitespace_index {
                                    Some((index, _)) => {
                                        stdout.queue(MoveToColumn(
                                            index as u16 + input_start_col + 1,
                                        ))?;
                                        buffer.set_caret_pos(index as u16 + input_start_col + 1);
                                    }
                                    None => {
                                        stdout.queue(MoveToColumn(input_start_col))?;
                                        buffer.set_caret_pos(input_start_col);
                                    }
                                }
                            } else {
                                stdout.execute(MoveLeft(1))?;
                                buffer.decrement_caret_pos();
                            }
                            stdout.flush()?;
                        }
                    }
                    KeyCode::Right => {
                        if buffer.get_caret_pos()
                            < input_start_col + buffer.get_buffer_length() as u16
                        {
                            if modifiers == KeyModifiers::ALT {
                                let whitespace_index = buffer.calcuate_word_right(input_start_col);
                                match whitespace_index {
                                    Some((index, _)) => {
                                        stdout.queue(MoveToColumn(
                                            index as u16 + input_start_col + 1,
                                        ))?;
                                        buffer.set_caret_pos(index as u16 + input_start_col + 1);
                                    }
                                    None => {
                                        stdout.queue(MoveToColumn(
                                            buffer.get_buffer_length() as u16 + input_start_col,
                                        ))?;
                                        buffer.set_caret_pos(
                                            buffer.get_buffer_length() as u16 + input_start_col,
                                        );
                                    }
                                }
                            } else {
                                stdout.execute(MoveRight(1))?;
                                buffer.increment_caret_pos();
                            }
                            stdout.flush()?;
                        }
                    }
                    _ => {}
                },
                Event::Mouse(event) => {
                    println!("Mouse event: {:?}", event);
                }
                Event::FocusGained => todo!(),
                Event::FocusLost => todo!(),
                Event::Paste(_) => todo!(),
                Event::Resize(w, h) => {
                    print_message(&mut stdout, &format!("Width: {w}, Height: {h}"))?;
                    break 'input;
                    // need to redraw to ensure proper word wrapping
                }
            }
        }
    }
    terminal::disable_raw_mode()?;
    Ok(())
}
