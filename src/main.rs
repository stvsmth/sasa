use crossterm::{
    cursor,
    event::{read, Event},
    style::{self, Stylize},
    terminal, ExecutableCommand, QueueableCommand, Result,
};
use std::io::{stdout, Write};

const BOTTOM_OFFSET: u16 = 8;

fn main() -> Result<()> {
    let mut stdout = stdout();

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    let (x_max, y_max) = terminal::size()?;
    
    // TODO: Exit early if we are below some useful dimensions.
    let y_max = y_max - BOTTOM_OFFSET; // Give some space for command prompt to reappear
    for y in 0..y_max {
        for x in 0..x_max {
            if (y == 0 || y == y_max - 1) || (x == 0 || x == x_max - 1) {
                stdout
                    .queue(cursor::MoveTo(x, y))?
                    .queue(style::PrintStyledContent("█".magenta()))?;
            } else if y == y_max / 2 {
                stdout
                    .queue(cursor::MoveTo(5, y))?
                    .queue(style::Print("Hello ..."))?;
            }
        }
    }
    stdout.flush()?;

    match read()? {
        Event::Key(_event) => (),
        Event::Mouse(_event) => (),
        Event::Resize(_width, _height) => (),
    }

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    for y in 0..y_max {
        for x in 0..x_max {
            if (y == 0 || y == y_max - 1) || (x == 0 || x == x_max - 1) {
                stdout
                    .queue(cursor::MoveTo(x, y))?
                    .queue(style::PrintStyledContent("█".white()))?;
            } else if y == y_max / 2 {
                stdout
                    .queue(cursor::MoveTo(5, y))?
                    .queue(style::Print("... World"))?;
            }
        }
    }
    stdout.flush()?;

    Ok(())
}
