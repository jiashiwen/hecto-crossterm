use std::io::{stdout, Write};
use crossterm::{execute, Result, terminal::{ScrollUp, SetSize, size}};

fn main() -> Result<()> {
    let (cols, rows) = size()?;
    // Resize terminal and scroll up.
    execute!(
        stdout(),
        SetSize(10, 10),
        ScrollUp(5)
    )?;

    // Be a good citizen, cleanup
    execute!(stdout(), SetSize(cols, rows))?;
    Ok(())
}