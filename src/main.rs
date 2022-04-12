use chrono::Local;
use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyModifiers},
    style::{self, Color, Stylize},
    terminal, ExecutableCommand, QueueableCommand, Result,
};
use fake::{faker::company::en::CatchPhase, Fake};
use image2ascii::string2ascii;
use rand::Rng;
use std::{fs, io::Stdout, process::exit};
use std::{
    io::{stdout, Write},
    thread, time,
};

const DRAW_CH: char = '*';
const MIN_ASCII_ART_HEIGHT: usize = 12;
const BOTTOM_OFFSET: u16 = 4;
const CONTENT_MARGIN: u16 = 4;

#[derive(Clone, Debug)]
enum Animate {
    On(u64),
}

#[derive(Clone, Debug)]
struct Line {
    y: u16,
    content: String,
    animate: Option<Animate>,
    color: Color,
}

fn main() -> Result<()> {
    terminal::enable_raw_mode()?;

    let (x_max, y_max_abs) = terminal::size()?;
    let y_max = y_max_abs - BOTTOM_OFFSET;

    let mut stdout = stdout();
    stdout.execute(cursor::Hide)?;
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    stdout
        .queue(cursor::MoveTo(CONTENT_MARGIN, y_max / 2))?
        .queue(style::Print("Ready to start\n"))?;
    stdout.flush()?;

    let slides_input = generate_buzzword_slides(x_max as usize, y_max as usize);
    let mut slides: Vec<&Vec<Line>> = slides_input.iter().rev().collect();
    let mut next_slide: Option<&Vec<Line>> = None;

    let mut slide_n = 0;
    let num_slides = slides.len();
    loop {
        // Clear terminal for next slide display
        match read()? {
            Event::Key(event) => {
                if event.code == KeyCode::Char(' ')
                    || event.code == KeyCode::Enter
                    || event.code == KeyCode::Char('n')
                {
                    next_slide = slides.pop();
                    if next_slide.is_some() {
                        slide_n += 1;
                    }
                } else if event.code == KeyCode::Char('q')
                    || event.code == KeyCode::Char('c') && event.modifiers == KeyModifiers::CONTROL
                {
                    stdout.execute(terminal::Clear(terminal::ClearType::All))?; // Maybe don't clear?
                    terminal::disable_raw_mode()?;
                    exit(0);
                }
            }
            _ => continue,
            // Event::Mouse(_event) => (),
            // Event::Resize(_width, _height) => (),
        }
        match next_slide {
            None => break,
            Some(slide) => {
                stdout.execute(terminal::Clear(terminal::ClearType::All))?;

                // Draw static elements first ... then the contents, which may animate
                draw_border(&mut stdout, x_max, y_max, slide)?;
                draw_footer(&mut stdout, x_max, y_max, slide_n, num_slides)?;
                draw_contents(&mut stdout, x_max, slide)?;
                // .. leave the cursor on the last row
                stdout.queue(cursor::MoveTo(0, y_max_abs))?;
                stdout.flush()?;
            }
        }
    }
    terminal::disable_raw_mode()?;
    Ok(())
}

fn draw_border(stdout: &mut Stdout, x_max: u16, y_max: u16, slide: &[Line]) -> Result<()> {
    let color = slide[0].color; // TODO: Taking first line color??
    for y in 0..y_max {
        for x in 0..x_max {
            if (y == 0 || y == y_max - 1) || [0, 1, x_max - 2, x_max - 1].contains(&x) {
                stdout
                    .queue(cursor::MoveTo(x, y))?
                    .queue(style::PrintStyledContent("█".with(color)))?;
            }
        }
    }
    Ok(())
}

fn draw_contents(stdout: &mut Stdout, x_max: u16, slide: &[Line]) -> Result<()> {
    let color = slide[0].color; // TODO: Taking first line color.
    for line in slide {
        let mut x = CONTENT_MARGIN;
        for ch in line.content.chars() {
            if let Some(Animate::On(rate)) = line.animate {
                stdout
                    .queue(cursor::MoveTo(x, line.y))?
                    .queue(style::PrintStyledContent("█".with(line.color)))?;
                stdout.flush()?;
                thread::sleep(time::Duration::from_millis(rate));
            }
            stdout
                .queue(cursor::MoveTo(x, line.y))?
                .queue(style::Print(ch))?;
            x += 1;
        }
        // ... finish of this line with a border element, then break for this line
        stdout
            .queue(cursor::MoveTo(x_max - 2, line.y))?
            .queue(style::PrintStyledContent("█".with(color)))?
            .queue(cursor::MoveTo(x_max - 1, line.y))?
            .queue(style::PrintStyledContent("█".with(color)))?;
    }
    Ok(())
}

fn draw_footer(stdout: &mut Stdout, x_max: u16, y_max: u16, n: usize, total: usize) -> Result<()> {
    let footer = format!("{n} of {total}", n = n, total = total);
    stdout
        .queue(cursor::MoveTo(x_max - 12, y_max - 3))?
        .queue(style::Print(footer.as_str()))?;
    Ok(())
}

fn generate_buzzword_slides(max_width: usize, max_height: usize) -> Vec<Vec<Line>> {
    let mut rng = rand::thread_rng();
    let slide_count = rng.gen_range(3..=5);
    let mut slides = Vec::with_capacity(slide_count);

    // Random color generation
    // TODO: Can we get color value of terminal and have dark vs light mode options?
    let colors = [
        Color::Cyan,
        Color::DarkMagenta,
        Color::Green,
        Color::Red,
        Color::White,
        Color::Yellow,
    ];

    // Center the lines vertically ... Assume the max height is 20
    // ... if we have 2 lines of text, those lines will be at 9, 10
    // ... if we have 3 lines of text, those lines will be at 9, 10, 11 etc
    // ... TODO: Currently not setting transition for first 2 slides
    for i in 0..slide_count {
        let mut rng = rand::thread_rng();
        let num_lines = rng.gen_range(2..=4);
        let mut y = (max_height / 2) - (num_lines / 2);
        let mut lines = Vec::with_capacity(num_lines);
        let color = colors[i % colors.len()];
        for j in 0..=num_lines {
            let with_bullet = j != 0;
            lines.push(Line {
                y: y as u16,
                content: generate_buzzword_phrase(with_bullet),
                animate: if i > 2 { Some(Animate::On(8)) } else { None },
                color,
            });
            y += 1;
        }
        slides.push(lines);
    }

    // /////////////////////////////////////////////////////////////////////////////////////////////////////
    // Add a `The End` slide
    let height = (max_height as f32 / 2.5).round();
    let now = Local::now();
    let message = format!("{}", now.format("%H:%M"));
    let c2d = string2ascii(message.as_str(), height, DRAW_CH, Option::None, None).unwrap();
    let time_art = c2d.to_lines();
    let num_lines = time_art.len();
    let y = (max_height / 2) - (num_lines / 2);
    let mut lines = Vec::with_capacity(num_lines + 1);

    let color = colors[slides.len() % colors.len()];
    lines.push(Line {
        y: (y - 1) as u16,
        content: "The end".to_string(),
        animate: None,
        color,
    });
    lines.extend(gen_lines_from_ascii(y + 2, time_art, true, color));
    slides.push(lines);

    // /////////////////////////////////////////////////////////////////////////////////////////////////////
    // Add ToDo slide
    // ... bullet points et al, compute height first, even though they're displayed last
    let color = colors[slides.len() % colors.len()];
    let header = "TODO!";
    let mut avoid_ascii_art = false;
    let todo_lines: Vec<String> = read_todo()
        .split('\n')
        .into_iter()
        .map(|l| l.to_string())
        .collect();
    let needed_space = todo_lines.len() + (CONTENT_MARGIN as usize * 2);
    let mut lines: Vec<Line> = vec![];

    // Draw ascii art for the header, if we have height & width
    match max_height.checked_sub(needed_space) {
        Some(height) => {
            let c2d = string2ascii(header, height as f32, DRAW_CH, Option::None, None).unwrap();
            let todo_art = c2d.to_lines();
            let needed_width = todo_art
                .iter()
                .fold(std::usize::MIN, |x, line| x.max(line.len()));
            // We need to build the ascii art to get it's true height, check that against a sane minimum
            if height <= MIN_ASCII_ART_HEIGHT || needed_width > max_width - CONTENT_MARGIN as usize
            {
                avoid_ascii_art = true;
            } else {
                // ... add the ascii art to the slide, starting at top
                lines.extend(gen_lines_from_ascii(
                    CONTENT_MARGIN.into(),
                    todo_art,
                    false,
                    color,
                ));
            }
        }
        // Underflow, we really don't have space
        None => avoid_ascii_art = true,
    };

    // If we don't have space for ascii art, just render it as its own line TODO: Add bold/underline/???
    if avoid_ascii_art {
        lines.push(Line {
            y: CONTENT_MARGIN,
            content: header.to_string(),
            animate: None,
            color,
        });
    }

    // Dump the contents of this slide
    // TODO: Handle text overflow (it currently just scrolls off screen ...)
    let mut y = lines.len() as u16 + CONTENT_MARGIN + 2;
    for line in todo_lines {
        y += 1;
        lines.push(Line {
            y,
            content: line,
            animate: None,
            color,
        });
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

fn gen_lines_from_ascii(
    mut y: usize,
    ascii_lines: Vec<String>,
    animate: bool,
    color: Color,
) -> Vec<Line> {
    let mut lines = vec![];
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
            animate: if animate { Some(Animate::On(1)) } else { None },
            color,
        });
        y += 1;
    }
    lines
}
