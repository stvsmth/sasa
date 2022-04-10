use chrono::Local;
use crossterm::{
    cursor,
    event::{read, Event},
    style::{self, Color, Stylize},
    terminal, ExecutableCommand, QueueableCommand, Result,
};
use fake::{faker::company::en::CatchPhase, Fake};
use image2ascii::string2ascii;
use rand::{seq::SliceRandom, Rng};
use std::fs;
use std::{
    collections::VecDeque,
    io::{stdout, Write},
    thread, time,
};

const DRAW_CH: char = '*';

const BOTTOM_OFFSET: u16 = 4;
const CONTENT_MARGIN: u16 = 4;

struct Line {
    y: u16,
    content: String,
    is_animated: bool,
    animation_rate: u64,
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
    stdout
        .execute(terminal::Clear(terminal::ClearType::All))?
        .execute(cursor::Hide)?;

    let num_slides = rng.gen_range(3..=5);
    let slides = generate_buzzword_slides(num_slides, y_max as usize);
    let num_slides = slides.len(); // We may add an ending slide
    let mut slide_content = slides.iter();
    let mut slide_n = 0;
    loop {
        let color = match colors.pop_front() {
            // Keep cycling through our colors.  TODO: This seems okay, but maybe there's a cleaner way.
            // Don't want random, because we want to make sure border changes on every slide change
            Some(c) => {
                colors.push_back(c);
                c
            }
            _ => unreachable!("Deque emptied, that shouldn't happen."),
        };
        match slide_content.next() {
            None => break,
            Some(slide) => {
                slide_n += 1;
                for y in 0..y_max {
                    // FIXME: This is dumb, should I be using if-let?
                    let (content, is_animated, rate) = match slide.iter().find(|l| l.y == y) {
                        None => ("".to_string(), false, 0),
                        Some(l) => (l.content.clone(), l.is_animated, l.animation_rate),
                    };
                    for x in 0..x_max {
                        // Draw border
                        if (y == 0 || y == y_max - 1) || [0, 1, x_max - 2, x_max - 1].contains(&x) {
                            // if (y == 0 || y == y_max - 1) || (x == 0 || x == x_max - 1) {
                            stdout
                                .queue(cursor::MoveTo(x, y))?
                                .queue(style::PrintStyledContent("█".with(color)))?;
                        // Draw line of text
                        } else if !content.is_empty() {
                            let mut this_x = x + CONTENT_MARGIN;
                            for ch in content.chars() {
                                if is_animated {
                                    stdout
                                        .queue(cursor::MoveTo(this_x, y))?
                                        .queue(style::PrintStyledContent("█".with(color)))?;
                                    stdout.flush()?;
                                    thread::sleep(time::Duration::from_millis(rate));
                                }
                                stdout
                                    .queue(cursor::MoveTo(this_x, y))?
                                    .queue(style::Print(ch))?;
                                this_x += 1;
                            }
                            // ... finish of this line with a border element, then break for this line
                            stdout
                                .queue(cursor::MoveTo(x_max - 2, y))?
                                .queue(style::PrintStyledContent("█".with(color)))?
                                .queue(cursor::MoveTo(x_max - 1, y))?
                                .queue(style::PrintStyledContent("█".with(color)))?;
                            break;
                        // Draw the footer, but not on the last slide
                        } else if y == y_max - CONTENT_MARGIN + 1  {
                            let footer = format!("{} of {}", slide_n, num_slides);
                            stdout
                                .queue(cursor::MoveTo(x_max - 12, y))?
                                .queue(style::Print(footer.as_str()))?;
                        }
                    }
                }
                stdout.queue(style::Print("\n"))?;
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

fn generate_buzzword_slides(slide_count: usize, max_height: usize) -> Vec<Vec<Line>> {
    let mut slides = Vec::with_capacity(slide_count);
    let mut rng = rand::thread_rng();

    // Center the lines vertically ... Assume the max height is 20
    // ... if we have 2 lines of text, those lines will be at 9, 10
    // ... if we have 3 lines of text, those lines will be at 9, 10, 11 etc
    // ... TODO: Currently not setting transition for first 2 slides
    for i in 0..slide_count {
        let num_lines = rng.gen_range(2..=4);
        let mut y = (max_height / 2) - (num_lines / 2);
        let mut lines = Vec::with_capacity(num_lines);
        for j in 0..=num_lines {
            let with_bullet = j != 0;
            lines.push(Line {
                y: y as u16,
                content: generate_buzzword_phrase(with_bullet),
                is_animated: i > 2,
                animation_rate: 8,
            });
            y += 1;
        }
        slides.push(lines);
    }

    // Add a `The End` slide
    let height = (max_height / 2) as f32;
    let now = Local::now();
    let message = format!("{}", now.format("%H:%M"));
    let c2d = string2ascii(message.as_str(), height, DRAW_CH, Option::None, None).unwrap();
    let ascii_lines = c2d.to_lines();
    let num_lines = ascii_lines.len();
    let mut y = (max_height / 2) - (num_lines / 2);
    let mut lines = Vec::with_capacity(num_lines + 1);

    lines.push(Line {
        y: (y - 1) as u16,
        content: "The end".to_string(),
        is_animated: false,
        animation_rate: 0,
    });
    y += 1;
    for line in ascii_lines {
        // image2ascii will contain space above/below to allow for ascender/descenders
        // trim that out if we are not using those values, it will throw off centering
        let mut candidate = line.clone();
        candidate.retain(|c| !c.is_whitespace());
        if candidate.is_empty() {
            continue;
        }

        lines.push(Line {
            y: y as u16,
            content: line,
            is_animated: true,
            animation_rate: 1,
        });
        y += 1;
    }
    slides.push(lines);

    // Add ToDo slide
    let todo_text: String = read_todo();
    let todo_lines: Vec<&str> = todo_text.split('\n').into_iter().collect();
    let num_lines = todo_lines.len();
    let mut y = (max_height / 2) - (num_lines / 2);
    let mut lines = Vec::with_capacity(num_lines);

    for line in todo_lines {
        lines.push(Line {
            y: y as u16,
            content: line.to_string(),
            is_animated: false,
            animation_rate: 0,
        });
        y += 1;
    }
    slides.push(lines);
    
    slides
}

fn generate_buzzword_phrase(with_bullet: bool) -> String {
    if with_bullet {
        format!("* {}", CatchPhase().fake::<String>())
    } else {
        CatchPhase().fake::<String>()
    }
}

fn read_todo() -> String {
    fs::read_to_string("todo.txt").expect("Need a todo.txt file for our presentation.")
}
