use chrono::Local;
use crossterm::{
    cursor,
    event::{read, Event},
    style::{self, Color, Stylize},
    terminal, ExecutableCommand, QueueableCommand, Result,
};
use fake::{faker::company::en::CatchPhase, Fake};
use image2ascii::string2ascii;
use rand::Rng;
use std::{fs, io::Stdout};
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
    let (x_max, y_max) = terminal::size()?;
    let y_max = y_max - BOTTOM_OFFSET;

    let mut stdout = stdout();
    stdout.execute(cursor::Hide)?;

    let slides = generate_buzzword_slides(x_max as usize, y_max as usize);
    let mut slide_content = slides.iter();
    let mut slide_n = 0;
    loop {
        // Clear terminal for next slide display
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;
        match slide_content.next() {
            None => break,
            Some(slide) => {
                slide_n += 1;
                
                // Draw static elements first ... then the contents, which may animate
                draw_border(&mut stdout, x_max, y_max, slide)?;
                draw_footer(&mut stdout, x_max, y_max, slide_n, slides.len())?;
                draw_contents(&mut stdout, x_max, slide)?;

                stdout.queue(style::Print("\n"))?;
                stdout.flush()?;

                match read()? {
                    Event::Key(_event) => (),
                    Event::Mouse(_event) => (),
                    Event::Resize(_width, _height) => (),
                }
            }
        }
    }
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
    let todo_lines: Vec<String> = read_todo()
        .split('\n')
        .into_iter()
        .map(|l| l.to_string())
        .collect();
    let todo_lines_count = todo_lines.len();
    let mut lines: Vec<Line> = vec![];

    // .. generate TODO as ascii art
    let height = max_height - todo_lines_count - (CONTENT_MARGIN * 2) as usize;
    let c2d = string2ascii(header, height as f32, DRAW_CH, Option::None, None).unwrap();
    let todo_art = c2d.to_lines();
    let needed_width = todo_art
        .iter()
        .fold(std::usize::MIN, |x, line| x.max(line.len()));
    if height <= MIN_ASCII_ART_HEIGHT  || needed_width > max_width {
        lines.push(Line {
            y: CONTENT_MARGIN,
            content: header.to_string(),
            animate: None,
            color,
        });
    } else {
        // ... add the ascii art to the slide, starting at top
        lines.extend(gen_lines_from_ascii(
            CONTENT_MARGIN.into(),
            todo_art,
            false,
            color,
        ));
    }

    // ... compute the starting point and add the actual text
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
