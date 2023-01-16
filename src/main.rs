use crossterm::{
    cursor::{position, MoveToColumn, RestorePosition, SavePosition},
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear},
    ExecutableCommand, QueueableCommand, Result,
};
use std::io::{stdout, Stdout, Write};
mod engine;
mod line_buffer;
use engine::{EditCommand, Engine};

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

fn buffer_repaint(stdout: &mut Stdout, engine: &Engine, prompt_offset: u16) -> Result<()> {
    let new_index = engine.get_insertion_point();
    let b = engine.get_buffer();
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
    let mut engine = Engine::new();
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
                    // TODO: Implement move to end for ctrl-e
                    KeyCode::Char('a') => {
                        engine.run_edit_commands(&[EditCommand::MoveToStart]);
                    }
                    KeyCode::Char('f') => {
                        engine.run_edit_commands(&[EditCommand::MoveRight]);
                    }
                    KeyCode::Char('b') => {
                        engine.run_edit_commands(&[EditCommand::MoveLeft]);
                    }
                    KeyCode::Char('u') => {
                        engine
                            .run_edit_commands(&[EditCommand::MoveToStart, EditCommand::CutToEnd]);
                    }
                    KeyCode::Char('k') => {
                        engine.run_edit_commands(&[EditCommand::CutToEnd]);
                    }
                    KeyCode::Char('y') => {
                        engine.run_edit_commands(&[EditCommand::InsertCutBuffer]);
                    }
                    _ => {}
                },
                Event::Key(KeyEvent {
                    code,
                    modifiers: KeyModifiers::ALT,
                    ..
                }) => match code {
                    KeyCode::Right => {
                        engine.run_edit_commands(&[EditCommand::MoveWordRight]);
                    }
                    KeyCode::Left => {
                        engine.run_edit_commands(&[EditCommand::MoveWordLeft]);
                    }
                    _ => {}
                },
                Event::Key(KeyEvent { code, .. }) => match code {
                    KeyCode::Char(c) => {
                        engine.run_edit_commands(&[
                            EditCommand::InsertChar(c),
                            EditCommand::MoveRight,
                        ]);
                    }
                    // TODO: Backspace should not work from start of string
                    KeyCode::Backspace => {
                        engine.run_edit_commands(&[EditCommand::Backspace]);
                    }
                    KeyCode::Enter => {
                        if engine.get_buffer() == "exit" {
                            break 'outer;
                        } else {
                            let buffer = String::from(engine.get_buffer());
                            engine.run_edit_commands(&[
                                EditCommand::AppendToHistory,
                                EditCommand::Clear,
                            ]);
                            print_message(&mut stdout, &format!("Buffer: {}", buffer))?;
                            break 'input;
                        }
                    }
                    // TODO: History
                    KeyCode::Up => {
                        engine.run_edit_commands(&[EditCommand::PreviousHistory]);
                    }
                    KeyCode::Down => {
                        engine.run_edit_commands(&[EditCommand::NextHistory]);
                    }
                    KeyCode::Left => {
                        engine.run_edit_commands(&[EditCommand::MoveLeft]);
                    }
                    KeyCode::Right => {
                        engine.run_edit_commands(&[EditCommand::MoveRight]);
                    }
                    KeyCode::Home => {
                        engine.set_insertion_point(0);
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
            buffer_repaint(&mut stdout, &engine, prompt_offset)?;
        }
    }
    terminal::disable_raw_mode()?;
    Ok(())
}
