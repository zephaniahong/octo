use crossterm::{
    cursor::{position, MoveLeft, MoveRight, MoveTo, MoveToColumn, MoveToNextLine},
    event::{self, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ScrollUp},
    ExecutableCommand, QueueableCommand, Result,
};
use std::io::{stdout, Stdout, Write};

fn print_message(stdout: &mut Stdout, msg: &str) -> Result<()> {
    stdout
        .queue(ScrollUp(1))?
        .queue(MoveToNextLine(1))?
        .queue(MoveToColumn(0))?
        .queue(Print(msg))?
        .queue(ScrollUp(1))?
        .queue(MoveToNextLine(1))?
        .queue(MoveToColumn(0))?;
    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let mut buffer = String::new();
    let mut stdout = stdout();
    let mut caret_pos;
    terminal::enable_raw_mode()?;

    'outer: loop {
        // print out prompt
        stdout
            .execute(SetForegroundColor(Color::Blue))?
            .execute(Print("> "))?
            .execute(ResetColor)?;

        let (input_start_col, _) = position()?;
        caret_pos = input_start_col;

        'input: loop {
            match read()? {
                Event::Key(KeyEvent { code, .. }) => match code {
                    KeyCode::Char(c) => {
                        let insertion_point = caret_pos - input_start_col;
                        stdout
                            .queue(Print(c))?
                            .queue(Print(&buffer[insertion_point.into()..]))?
                            .queue(MoveToColumn(caret_pos + 1))?;
                        stdout.flush()?;
                        buffer.insert(insertion_point.into(), c);
                        caret_pos += 1;
                    }
                    KeyCode::Backspace => {
                        let insertion_point: usize = (caret_pos - input_start_col).into();
                        if insertion_point == buffer.len() && !buffer.is_empty() {
                            buffer.pop();
                            stdout
                                .queue(MoveLeft(1))?
                                .queue(Print(" "))?
                                .queue(MoveLeft(1))?;
                        } else if insertion_point < buffer.len() && !buffer.is_empty() {
                            buffer.remove(insertion_point - 1);
                            stdout
                                .queue(MoveLeft(1))?
                                .queue(Print(&buffer[(insertion_point - 1)..]))?
                                .queue(Print(" "))?
                                .queue(MoveToColumn(caret_pos - 1))?;
                        }
                        stdout.flush()?;
                        caret_pos -= 1;
                    }
                    KeyCode::Enter => {
                        if buffer == "exit" {
                            break 'outer;
                        } else {
                            print_message(&mut stdout, &format!("Buffer: {buffer}"))?;
                            buffer.clear();
                            break 'input;
                        }
                    }
                    KeyCode::Left => {
                        if caret_pos > input_start_col {
                            stdout.execute(MoveLeft(1))?;
                            stdout.flush()?;
                            caret_pos -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if caret_pos < input_start_col + buffer.len() as u16 {
                            stdout.execute(MoveRight(1))?;
                            stdout.flush()?;
                            caret_pos += 1;
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
