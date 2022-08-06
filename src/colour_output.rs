// Provides simple colour output functions based on termcolor, to stdout and stderr
use anyhow::Result;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct ColourOutput {
    stream: StandardStream,
    chunks: Vec<Chunk>,
}

pub enum StreamKind {
    Stderr,
    Stdout,
}

impl ColourOutput {
    pub fn new(dest: StreamKind) -> Self {
        let writer = match dest {
            StreamKind::Stderr => StandardStream::stderr(ColorChoice::Always),
            StreamKind::Stdout => StandardStream::stdout(ColorChoice::Always),
        };
        ColourOutput {
            stream: writer,
            chunks: Vec::new(),
        }
    }

    pub fn append<I: Into<String>>(mut self, content: I, kind: Style) -> Self {
        self.chunks.push(Chunk::new(content, kind));
        self
    }
}
struct Chunk {
    content: String,
    kind: Style,
}

impl Chunk {
    fn new<I: Into<String>>(content: I, kind: Style) -> Self {
        Chunk {
            content: content.into(),
            kind,
        }
    }
}

impl Chunk {
    fn colour(&self) -> Color {
        match self.kind {
            Style::Normal => Color::White,
            Style::Error => Color::Red,
            Style::TaskContent => Color::Green,
            Style::ListName => Color::Cyan,
            Style::Warning => Color::Rgb(144, 40, 159),
            Style::Link => Color::Blue,
        }
    }
    fn is_bold(&self) -> bool {
        matches!(
            self.kind,
            Style::TaskContent | Style::Error | Style::ListName | Style::Warning
        )
    }
}
pub enum Style {
    Normal,
    Error,
    TaskContent,
    ListName,
    Warning,
    Link,
}

impl ColourOutput {
    pub fn println(mut self) -> Result<()> {
        for chunk in self.chunks {
            self.stream.set_color(
                ColorSpec::new()
                    .set_fg(Some(chunk.colour()))
                    .set_bold(chunk.is_bold()),
            )?;
            write!(self.stream, "{}", chunk.content)?;
        }
        // reset styles
        self.stream
            .set_color(ColorSpec::new().set_fg(Some(Color::White)).set_bold(false))?;
        writeln!(self.stream)?;
        Ok(())
    }
}
