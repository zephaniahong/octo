use crossterm::{
    cursor::{position, MoveToColumn, RestorePosition, SavePosition},
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear},
    ExecutableCommand, QueueableCommand, Result,
};
use std::io::{stdout, Stdout, Write};
mod line_buffer;
use line_buffer::LineBuffer;

#[macro_use]
extern crate log;

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

fn buffer_repaint(stdout: &mut Stdout, buffer: &LineBuffer, prompt_offset: u16) -> Result<()> {
    let new_index = buffer.get_insertion_point();
    let b = buffer.get_buffer();
    stdout
        .queue(MoveToColumn(prompt_offset))?
        .queue(Print(&b[0..new_index]))?
        .queue(SavePosition)?
        .queue(Print(&b[new_index..]))?
        .queue(Clear(terminal::ClearType::UntilNewLine))?
        .queue(RestorePosition)?;
    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();
    info!("Starting up!!!!");
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
                    code,
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => match code {
                    KeyCode::Char('c') | KeyCode::Char('d') => {
                        print_message(&mut stdout, "exiting...")?;
                        break 'outer;
                    }
                    KeyCode::Char('a') => {
                        buffer.set_insertion_point(0);
                        buffer_repaint(&mut stdout, &buffer, prompt_offset)?;
                    }
                    KeyCode::Char('u') => {
                        buffer.clear_buffer();
                        buffer_repaint(&mut stdout, &buffer, prompt_offset)?;
                    }
                    KeyCode::Char('k') => {
                        buffer.clear_to_end(buffer.get_insertion_point());
                        buffer_repaint(&mut stdout, &buffer, prompt_offset)?;
                    }
                    _ => {}
                },
                Event::Key(KeyEvent {
                    code, modifiers, ..
                }) => match code {
                    KeyCode::Char(c) => {
                        buffer.insert_char(buffer.get_insertion_point(), c);
                        buffer.increment_insertion_point();
                        buffer_repaint(&mut stdout, &buffer, prompt_offset)?;
                    }
                    KeyCode::Backspace => {
                        let insertion_point = buffer.get_insertion_point();
                        if insertion_point == buffer.get_buffer_length() && !buffer.is_empty() {
                            buffer.decrement_insertion_point();
                            buffer.pop();
                            buffer_repaint(&mut stdout, &buffer, prompt_offset)?;
                        } else if insertion_point < buffer.get_buffer_length() && !buffer.is_empty()
                        {
                            buffer.decrement_insertion_point();
                            buffer.remove_char(buffer.get_insertion_point());
                            buffer_repaint(&mut stdout, &buffer, prompt_offset)?;
                        }
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
                    // TODO: History
                    KeyCode::Up => {}
                    KeyCode::Left => {
                        if buffer.get_insertion_point() > 0 {
                            if modifiers == KeyModifiers::ALT {
                                buffer.move_word_left();
                                buffer_repaint(&mut stdout, &buffer, prompt_offset)?;
                            } else {
                                buffer.decrement_insertion_point();
                                buffer_repaint(&mut stdout, &buffer, prompt_offset)?;
                            }
                        }
                    }
                    KeyCode::Right => {
                        if buffer.get_insertion_point() < buffer.get_buffer_length() {
                            if modifiers == KeyModifiers::ALT {
                                buffer.move_word_right();
                                buffer_repaint(&mut stdout, &buffer, prompt_offset)?;
                            } else {
                                buffer.increment_insertion_point();
                                buffer_repaint(&mut stdout, &buffer, prompt_offset)?;
                            }
                        }
                    }
                    KeyCode::Home => {
                        buffer.set_insertion_point(0);
                        buffer_repaint(&mut stdout, &buffer, prompt_offset)?;
                    }
                    _ => {}
                },
                Event::Mouse(event) => {
                    println!("Mouse event1: {:?}", event);
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
