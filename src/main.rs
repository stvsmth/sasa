use crossterm::{
    cursor,
    event::{read, Event},
    style::{self, Color, Stylize},
    terminal, ExecutableCommand, QueueableCommand, Result,
};
use fake::{faker::company::en::CatchPhase, Fake};
use rand::{seq::SliceRandom, Rng};
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

    let num_slides = rng.gen_range(5..7);
    let slides = get_slide_content(num_slides, y_max as usize);
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

fn get_slide_content(slide_count: usize, max_height: usize) -> Vec<Vec<Line>> {
    // Temporary helper function to generate content until we add Markdown parsing
    let mut slides = Vec::with_capacity(slide_count);
    let mut rng = rand::thread_rng();

    // Center the lines vertically ... Assume the max height is 20
    // ... if we have 2 lines of text, those lines will be at 9, 10
    // ... if we have 3 lines of text, those lines will be at 9, 10, 11 etc
    for _ in 0..=slide_count {
        let num_lines = rng.gen_range(2..4);
        let mut y = (max_height / 2) - (num_lines / 2);
        let mut lines = Vec::with_capacity(num_lines);
        for j in 0..=num_lines {
            let with_bullet = if j == 0 {false} else {true};
            lines.push(
                Line {
                y: y as u16,
                content: generate_buzzword_phrase(with_bullet),
            });
            y += 1;
        }
        slides.push(lines);
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
