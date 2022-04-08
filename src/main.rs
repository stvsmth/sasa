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
const CONTENT_MARGIN: u16 = 4;

struct Line {
    y: u16,
    content: String,
}

fn main() -> Result<()> {
    let (x_max, y_max) = terminal::size()?;
    let y_max = y_max - BOTTOM_OFFSET;

    // TODO: We'll read the input and construct the Line structs with computed Y values
    let input = vec![
        vec![
            Line {
                y: 8,
                content: "Hello ...".to_string(),
            },
            Line {
                y: 9,
                content: "... Cruel World".to_string(),
            },
            Line {
                y: 10,
                content: "--".to_string(),
            },
            Line {
                y: 11,
                content: "Yours, Sasa".to_string(),
            },
        ],
        vec![
            Line {
                y: 8,
                content: "This is".to_string(),
            },
            Line {
                y: 9,
                content: "... the second slide".to_string(),
            },
        ],
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
            Some(slide) => {
                for y in 0..y_max {
                    let content = match slide.iter().find(|l| l.y == y) {
                        None => "".to_string(),
                        Some(l) => l.content.clone(),
                    };
                    for x in 0..x_max {
                        // Draw border
                        if (y == 0 || y == y_max - 1) || (x == 0 || x == x_max - 1) {
                            stdout
                                .queue(cursor::MoveTo(x, y))?
                                .queue(style::PrintStyledContent("█".with(color)))?;
                        // Draw content
                        } else if !content.is_empty() {
                            let mut this_x = x + CONTENT_MARGIN;
                            for ch in content.chars() {
                                stdout
                                    .queue(cursor::MoveTo(this_x, y))?
                                    .queue(style::Print(ch))?;
                                this_x += 1;
                            }
                            break;
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
