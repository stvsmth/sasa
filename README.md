# Sasa

Command-line presentation tool. VERY alpha.

Sasa is Swahili for "to present."

![Demo screen cast](https://f004.backblazeb2.com/file/sasa-files/demo.gif)

## Known issues / Upcoming work

* Only displays random catch-phrases. While humorous, this isn't very useful. Next
  step is to parse Markdown or some other kind of markup language.
* Resizing the window will create a mess. (Shouldn't be too hard to fix.)
* Math for handling ASCII-art headers needs much more attention.
* Content may overflow current terminal size in unpleasant ways.

## Motivation

Initially, this just seemed like a fun way to play around in Rust. But there
might be value in a CLI presentation tool when your presentation involves moving
around in the terminal as well.

But mostly it's a tool for exploring Rust first and displaying presentations second.

## Useful links

* Cross-platform, idiomatic Rust terminal interface
  * `https://docs.rs/crossterm/latest/crossterm/`
  * `https://github.com/crossterm-rs/crossterm/tree/master/examples`
* Markdown parsing, using pull-down technology
  * `https://docs.rs/pulldown-cmark/0.9.1/pulldown_cmark`
* Rendering ASCII art
  * `https://github.com/kizitorashiro/image2ascii`
* Generating ASCII diagrams
  * `https://monodraw.helftone.com` This program is reason enough to create
    an command-line presentation tool. It's a Mac only tool, but I'm sure there
    are others.
