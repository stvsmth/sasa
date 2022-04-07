use crossterm::{
    cursor,
    style::{self, Stylize},
    terminal, ExecutableCommand, QueueableCommand, Result,
};
use std::io::{stdout, Write};

fn main() -> Result<()> {
    let mut stdout = stdout();

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    let y_max = 40;
    let x_max = 150;
    for y in 0..y_max {
        for x in 0..x_max {
            if (y == 0 || y == y_max - 1) || (x == 0 || x == x_max - 1) {
                // in this loop we are more efficient by not flushing the buffer.
                stdout
                    .queue(cursor::MoveTo(x, y))?
                    .queue(style::PrintStyledContent("█".magenta()))?;
            } else if y == y_max / 2 {
                stdout
                    .queue(cursor::MoveTo(5, y))?
                    .queue(style::Print("Hello World"))?;
            }
        }
    }
    stdout.flush()?;
    Ok(())
}
