use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyModifiers},
    style::{self, Color, Stylize},
    terminal, ExecutableCommand, QueueableCommand, Result,
};
use fake::{faker::company::en::CatchPhase, Fake};
use image2ascii::string2ascii;
use nix::{sys::signal, unistd::getpid};
use rand::Rng;
use std::{fs, io::Stdout, process::exit};
use std::{
    io::{stdout, Write},
    path::PathBuf,
    thread, time,
};

const DRAW_CH: char = '*';
const MIN_ASCII_ART_HEIGHT: usize = 12;
const BOTTOM_OFFSET: u16 = 4;
const CONTENT_MARGIN: u16 = 4;

#[derive(Clone, Debug)]
struct Animate {
    rate: u64,
}

#[derive(Clone, Debug)]
struct Line {
    y: u16,
    content: String,
    animate: Option<Animate>,
    color: Color,
}

fn main() -> Result<()> {
    // Terminal init
    let (x_max, y_max_abs) = terminal::size()?;
    let y_max = y_max_abs - BOTTOM_OFFSET;
    let mut stdout = stdout();

    // Clear terminal, hide cursor, enable raw mode
    take_terminal(&mut stdout)?;
    let mut curr_y = y_max / 2;

    // Display start screen (not a slide)
    stdout
        .queue(cursor::MoveTo(CONTENT_MARGIN, curr_y))?
        .queue(style::Print("Ready to start"))?;
    curr_y += 2;
    stdout
        .queue(cursor::MoveTo(CONTENT_MARGIN, curr_y))?
        .queue(style::Print(
            "`n` or `space` or `enter` to move to next slide",
        ))?;
    curr_y += 1;
    stdout
        .queue(cursor::MoveTo(CONTENT_MARGIN, curr_y))?
        .queue(style::Print("`p` move to previous slide"))?;
    curr_y += 1;
    stdout
        .queue(cursor::MoveTo(CONTENT_MARGIN, curr_y))?
        .queue(style::Print("`q` to quit presentation"))?;
    stdout.flush()?;

    // Slides init
    let slides_input = generate_buzzword_slides(x_max as usize, y_max as usize);
    let mut slides: Vec<&Vec<Line>> = slides_input.iter().rev().collect();
    let mut prev_slides: Vec<&Vec<Line>> = vec![];
    let mut prev_slide: Option<&Vec<Line>> = None;
    let mut slide_n = 0;
    let num_slides = slides.len();

    // Timer init
    let start_ts: time::Instant = time::Instant::now();
    let mut display_elapsed_time = false;

    // Event handler
    loop {
        if poll(time::Duration::from_millis(500))? {
            match read()? {
                Event::Key(event) => {
                    // ... advance to next slide (<space>, <enter>, n)
                    // ... TODO: next slide & prev slide cases are awfully similar
                    if event.code == KeyCode::Enter
                        || event.code == KeyCode::Char('n')
                        || event.code == KeyCode::Char(' ')
                    {
                        match slides.pop() {
                            None => continue, // if there's no more slides, just stop on last slide
                            Some(slide) => {
                                take_terminal(&mut stdout)?;
                                slide_n += 1;
                                // Draw static elements first ... then the contents, which may animate
                                draw_border(&mut stdout, x_max, y_max, slide)?;
                                draw_footer(&mut stdout, x_max, y_max, slide_n, num_slides)?;
                                draw_contents(&mut stdout, x_max, slide)?;
                                stdout.flush()?;

                                // Push the previous slide onto the stack for navigating backward.
                                // Note, this isn't the current slide because we would then navigate
                                // back to the current slide before getting back to the actual previous slide
                                if let Some(s) = prev_slide {
                                    prev_slides.push(s);
                                }
                                prev_slide = Some(slide);
                            }
                        }
                    // ... go back one slide (p)
                    // ... TODO: next slide & prev slide cases are awfully similar
                    } else if event.code == KeyCode::Char('p') {
                        if let Some(slide) = prev_slides.pop() {
                            take_terminal(&mut stdout)?;
                            slide_n -= 1;
                            // Draw static elements first ... then the contents, which may animate
                            draw_border(&mut stdout, x_max, y_max, slide)?;
                            draw_footer(&mut stdout, x_max, y_max, slide_n, num_slides)?;
                            draw_contents(&mut stdout, x_max, slide)?;
                            stdout.flush()?;

                            // Push the previous slide onto the stack for navigating backward.
                            // Note, this isn't the current slide because we would then navigate
                            // back to the current slide before getting back to the actual previous slide
                            if let Some(s) = prev_slide {
                                slides.push(s);
                            }
                            prev_slide = Some(slide);
                        }
                    // ... toggle the timer (t)
                    } else if event.code == KeyCode::Char('t') {
                        display_elapsed_time = !display_elapsed_time;
                    // ... suspend (ctrl-z) ... we have to handle "standard" commands in raw mode
                    } else if event.code == KeyCode::Char('z')
                        && event.modifiers == KeyModifiers::CONTROL
                    {
                        release_terminal(&mut stdout)?;
                        signal::kill(getpid(), signal::SIGSTOP).unwrap();
                    // ... quit (q, ctrl-c)
                    } else if event.code == KeyCode::Char('q')
                        || event.code == KeyCode::Char('c')
                            && event.modifiers == KeyModifiers::CONTROL
                    {
                        release_terminal(&mut stdout)?;
                        exit(0);
                    }
                }
                _ => continue,
                // Event::Mouse(_event) => (),
                // Event::Resize(_width, _height) => (),
            }
        }
        let curr_ts = time::Instant::now();
        let elapsed = (curr_ts - start_ts).as_secs();
        let mut curr_ts_str = format!("Elapsed time: {}:{:02}", elapsed / 60, elapsed % 60);
        if !display_elapsed_time {
            // overwrite previously displayed elapsed time with space chars
            let elapsed_len = curr_ts_str.len();
            curr_ts_str = (0..elapsed_len).map(|_| " ").collect::<String>();
        }
        stdout
            .queue(cursor::MoveTo(
                x_max - curr_ts_str.len() as u16 - CONTENT_MARGIN - 2,
                y_max + 1,
            ))?
            .queue(style::Print(curr_ts_str))?;
        stdout.flush()?;
    }
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
            if let Some(Animate { rate }) = line.animate {
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

fn take_terminal(stdout: &mut Stdout) -> Result<()> {
    // Take control of terminal, useful if we come back in via SIGCONT
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    stdout.execute(cursor::Hide)?;
    terminal::enable_raw_mode()?;
    Ok(())
}

fn release_terminal(stdout: &mut Stdout) -> Result<()> {
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn generate_buzzword_slides(max_width: usize, max_height: usize) -> Vec<Vec<Line>> {
    let mut rng = rand::thread_rng();
    let slide_count = rng.gen_range(3..=7);
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
                animate: if i > 2 {
                    Some(Animate { rate: 8 })
                } else {
                    None
                },
                color,
            });
            y += 1;
        }
        slides.push(lines);
    }

    // /////////////////////////////////////////////////////////////////////////////////////////////////////
    // Add Text slides
    // TODO: Install glob crate and read all slide*txt files.
    //       Better yet, read one file and split on slides.

    let diagram = PathBuf::from("Diagram.txt");
    let todo = PathBuf::from("TODO.txt");
    let the_end = PathBuf::from("The_End.txt");
    for path in vec![diagram, todo, the_end] {
        let color = colors[slides.len() % colors.len()];
        let header = path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .replace('_', " ");
        let mut avoid_ascii_art = false;
        let text_lines: Vec<String> = fs::read_to_string(&path)
            .unwrap() // TODO: Do proper error handling
            .split('\n')
            .into_iter()
            .map(|l| l.to_string())
            .collect();
        let needed_space = text_lines.len() + (CONTENT_MARGIN as usize * 2);
        let mut lines: Vec<Line> = vec![];

        // Draw ascii art for the header, if we have height & width
        match max_height.checked_sub(needed_space) {
            Some(mut height) => {
                if height >= max_height / 2 {
                    height = (max_height as f32 / 2.5) as usize;
                }
                let c2d =
                    string2ascii(&header, height as f32, DRAW_CH, Option::None, None).unwrap();
                let text_art = c2d.to_lines();
                // Find the widest part of our text art
                let needed_width = text_art
                    .iter()
                    .fold(std::usize::MIN, |x, line| x.max(line.len()));
                // We need to build the ascii art to get it's true height, check that against a sane minimum
                if height <= MIN_ASCII_ART_HEIGHT
                    || needed_width > max_width - CONTENT_MARGIN as usize
                {
                    avoid_ascii_art = true;
                } else {
                    // ... add the ascii art to the slide, starting at top
                    lines.extend(gen_lines_from_ascii(
                        CONTENT_MARGIN.into(),
                        text_art,
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
        for line in text_lines {
            y += 1;
            lines.push(Line {
                y,
                content: line,
                animate: None,
                color,
            });
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
            animate: if animate {
                Some(Animate { rate: 1 })
            } else {
                None
            },
            color,
        });
        y += 1;
    }
    lines
}
