use crossterm::{
    cursor,
    event::{read, Event},
    style::{self, Color, Stylize},
    terminal, ExecutableCommand, QueueableCommand, Result,
};
use rand::seq::SliceRandom;
use std::{
    collections::VecDeque,
    io::{stdout, Write},
};

const BOTTOM_OFFSET: u16 = 4;
const CONTENT_MARGIN: usize = 4;

fn main() -> Result<()> {
    let (x_max, y_max) = terminal::size()?;
    let y_max = y_max - BOTTOM_OFFSET;

    let input = vec![
        vec!["Hello ...", "... World"],
        vec!["Oh so", "Cruel World"],
        vec!["from", "Sasa", "your", "friendly", "presentation tool"],
        vec!["Goodbye ...", "for now"],
        vec!["Oh", "Cruel World"],
        vec!["with", "love"],
        vec!["Your", "S"],  // FIXME: We need to NOT reset the X coordinate.
    ];
    let mut slide_content = input.iter();

    // Random color generation
    // TODO: Can we get color value of terminal and have dark vs light mode options?
    let mut colors = [
        Color::Cyan,
        Color::DarkMagenta,
        Color::Green,
        Color::Red,
        Color::White,
        Color::Yellow,
    ];
    let mut rng = rand::thread_rng();
    colors.shuffle(&mut rng);
    let mut colors = VecDeque::from(colors);

    let mut stdout = stdout();
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    loop {
        let color = match colors.pop_front() {
            // Keep cycling through our colors.  TODO: This seems okay, but maybe there's a cleaner way.
            Some(c) => {
                colors.push_back(c);
                c
            }
            _ => unreachable!("Deque emptied, that shouldn't happen."),
        };

        match slide_content.next() {
            None => break,
            Some(lines) => {
                let height = lines.len();
                if height > y_max as usize - CONTENT_MARGIN {
                    panic!("Input too tall")
                }
                let half_height: u16 = ((height / 2) + 1)
                    .try_into()
                    .expect("Failed to downcast half-height");
                for y in 0..y_max {
                    for x in 0..x_max {
                        // Draw border
                        if (y == 0 || y == y_max - 1) || (x == 0 || x == x_max - 1) {
                            stdout
                                .queue(cursor::MoveTo(x, y))?
                                .queue(style::PrintStyledContent("█".with(color)))?;
                        // Draw content
                        } else if y >= (y_max / 2) - half_height && y < (y_max / 2) + half_height {
                            for line in lines.iter() {
                                stdout
                                    .queue(cursor::MoveTo(x, y))?
                                    .queue(style::Print(line))?;
                            }
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
