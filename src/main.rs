use crossterm::{
    cursor::{position, MoveLeft, MoveRight, MoveToColumn, MoveToNextLine},
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal, ExecutableCommand, QueueableCommand, Result,
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

    pub fn set_insertion_point(&mut self, pos: usize) {
        self.insertion_point = pos;
    }

    pub fn get_insertion_point(&self) -> usize {
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

    pub fn increment_insertion_point(&mut self) {
        self.insertion_point += 1;
    }

    pub fn decrement_insertion_point(&mut self) {
        self.insertion_point = self.insertion_point.saturating_sub(1);
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

    pub fn move_word_left(&mut self) -> usize {
        match self
            .buffer
            .rmatch_indices(&[' ', '\t'][..])
            .find(|(index, _)| index < &(self.insertion_point - 1))
        {
            Some((index, _)) => {
                self.insertion_point = index;
                index
            }
            None => {
                self.insertion_point = 0;
                0
            }
        }
    }

    pub fn move_word_right(&mut self) -> usize {
        match self
            .buffer
            .match_indices(&[' ', '\t'][..])
            .find(|(index, _)| index > &self.insertion_point)
        {
            Some((index, _)) => {
                self.insertion_point = index + 1;
                index
            }
            None => {
                self.insertion_point = self.get_buffer_length() - 1;
                self.get_insertion_point()
            }
        }
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

        let (prompt_offset, _) = position()?;

        'input: loop {
            match read()? {
                Event::Key(KeyEvent {
                    code, modifiers, ..
                }) => match code {
                    KeyCode::Char(c) => {
                        if modifiers == KeyModifiers::CONTROL && (c == 'd' || c == 'c') {
                            print_message(&mut stdout, "exiting...")?;
                            break 'outer;
                        }
                        let insertion_point = buffer.get_insertion_point();
                        stdout
                            .queue(Print(c))?
                            .queue(Print(buffer.slice_buffer(insertion_point)))?
                            .queue(MoveToColumn(insertion_point as u16 + prompt_offset + 1))?;
                        stdout.flush()?;
                        buffer.increment_insertion_point();
                        buffer.insert_char(insertion_point, c);
                    }
                    KeyCode::Backspace => {
                        let insertion_point = buffer.get_insertion_point();
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
                                .queue(MoveToColumn(
                                    buffer.get_insertion_point() as u16 + prompt_offset - 1,
                                ))?;
                        }
                        stdout.flush()?;
                        buffer.decrement_insertion_point();
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
                            buffer.set_insertion_point(0);
                            break 'input;
                        }
                    }
                    KeyCode::Left => {
                        if buffer.get_insertion_point() > 0 {
                            if modifiers == KeyModifiers::ALT {
                                let new_insertion_point = buffer.move_word_left();
                                stdout.queue(MoveToColumn(
                                    new_insertion_point as u16 + prompt_offset,
                                ))?;
                            } else {
                                stdout.execute(MoveLeft(1))?;
                                buffer.decrement_insertion_point();
                            }
                            stdout.flush()?;
                        }
                    }
                    KeyCode::Right => {
                        if buffer.get_insertion_point() < buffer.get_buffer_length() {
                            if modifiers == KeyModifiers::ALT {
                                let new_insertion_point = buffer.move_word_right();
                                stdout.queue(MoveToColumn(
                                    new_insertion_point as u16 + prompt_offset + 1,
                                ))?;
                            } else {
                                stdout.execute(MoveRight(1))?;
                                buffer.increment_insertion_point();
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
