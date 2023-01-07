use crossterm::{
    cursor::{position, MoveLeft, MoveRight, MoveToColumn},
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal, ExecutableCommand, QueueableCommand, Result,
};
use std::io::{stdout, Stdout, Write};
mod line_buffer;
use line_buffer::LineBuffer;

#[macro_use]
extern crate log;

fn print_message(stdout: &mut Stdout, msg: &str) -> Result<()> {
    stdout
        .queue(Print("\n"))?
        .queue(MoveToColumn(1))?
        .queue(Print(msg))?
        .queue(Print("\n"))?
        .queue(MoveToColumn(1))?;
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

        let (mut prompt_offset, _) = position()?;
        prompt_offset += 1;

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
                        if insertion_point == buffer.get_buffer_length() {
                            stdout.queue(Print(c))?;
                        } else {
                            stdout
                                .queue(Print(c))?
                                .queue(Print(buffer.slice_buffer(insertion_point)))?
                                .queue(MoveToColumn(insertion_point as u16 + prompt_offset + 1))?;
                        }
                        stdout.flush()?;
                        buffer.insert_char(insertion_point, c);
                        buffer.increment_insertion_point();
                    }
                    // TODO: Error when using alt to go to start and then backspace
                    KeyCode::Backspace => {
                        let insertion_point = buffer.get_insertion_point();
                        if insertion_point == buffer.get_buffer_length() && !buffer.is_empty() {
                            buffer.decrement_insertion_point();
                            buffer.pop();
                            stdout
                                .queue(MoveLeft(1))?
                                .queue(Print(" "))?
                                .queue(MoveLeft(1))?;
                        } else if insertion_point < buffer.get_buffer_length() && !buffer.is_empty()
                        {
                            buffer.decrement_insertion_point();
                            let insertion_point = buffer.get_insertion_point();
                            buffer.remove_char(insertion_point);
                            stdout
                                .queue(MoveLeft(1))?
                                .queue(Print(buffer.slice_buffer(insertion_point)))?
                                .queue(Print(" "))?
                                .queue(MoveToColumn(
                                    buffer.get_insertion_point() as u16 + prompt_offset,
                                ))?;
                        }
                        stdout.flush()?;
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
