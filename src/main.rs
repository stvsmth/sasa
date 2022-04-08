use crossterm::{
    cursor,
    event::{read, Event},
    style::{self, Color, Stylize},
    terminal, ExecutableCommand, QueueableCommand, Result,
};
use fake::{faker::company::en::CatchPhase, Fake};
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

    let slides = get_slide_content(5);
    let mut slide_content = slides.iter();
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

fn get_slide_content(slide_count: usize) -> Vec<Vec<Line>> {
    // FIXME: Take in y_height and compute the vertical center values for y positions.
    // Temporary helper function to generate content until we add Markdown parsing
    let mut slides = Vec::with_capacity(slide_count);
    for _ in 0..=slide_count {
        slides.push(vec![
            Line {
                y: 8,
                content: generate_buzzword_phrase(false),
            },
            Line {
                y: 9,
                content: generate_buzzword_phrase(true),
            },
            Line {
                y: 10,
                content: generate_buzzword_phrase(true),
            },
        ])
    }
    slides
}

fn generate_buzzword_phrase(with_bullet: bool) -> String {
    if with_bullet {
        format!("* {}", CatchPhase().fake::<String>())
    } else {
        CatchPhase().fake::<String>()
    }
}
