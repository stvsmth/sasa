use crossterm::{
    cursor,
    event::{read, Event},
    style::{self, Stylize},
    terminal, ExecutableCommand, QueueableCommand, Result,
};
use std::io::{stdout, Write};

const BOTTOM_OFFSET: u16 = 4;

fn main() -> Result<()> {
    let (x_max, y_max) = terminal::size()?;
    let y_max = y_max - BOTTOM_OFFSET;

    let input = vec!["Hello ...", "... World", "from", "Sasa"];
    let mut lines = input.iter();

    let mut stdout = stdout();
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    loop {
        match lines.next() {
            None => break,
            Some(line) => {
                for y in 0..y_max {
                    for x in 0..x_max {
                        if (y == 0 || y == y_max - 1) || (x == 0 || x == x_max - 1) {
                            stdout
                                .queue(cursor::MoveTo(x, y))?
                                .queue(style::PrintStyledContent("█".white()))?;
                        } else if y == y_max / 2 {
                            stdout
                                .queue(cursor::MoveTo(5, y))?
                                .queue(style::Print(line))?;
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
            }
        }
    }
    Ok(())
}
