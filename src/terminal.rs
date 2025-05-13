use crossterm::{
    cursor::{self, MoveTo},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use std::{
    io::{stdout, Write},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

pub async fn terminal_output(
    counter: Arc<AtomicU64>,
    method: reqwest::Method,
) -> anyhow::Result<()> {
    tokio::time::sleep(Duration::from_secs(4)).await;
    let mut stdout = stdout();
    write!(stdout, "\n")?;
    stdout.flush()?;
    let (_, y) = cursor::position()?;
    loop {
        stdout
            .execute(MoveTo(0, y))?
            .execute(Clear(ClearType::CurrentLine))?;
        write!(
            stdout,
            "The {} request has sent {} times",
            method,
            counter.load(Ordering::Relaxed)
        )?;

        stdout.flush()?;

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
