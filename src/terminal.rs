use crossterm::{
    cursor::{MoveTo, MoveUp},
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

pub async fn terminal_output(counter: Arc<AtomicU64>, method: String) -> anyhow::Result<()> {
    tokio::time::sleep(Duration::from_secs(4)).await;
    let mut stdout = stdout();
    loop {
        stdout
            .execute(MoveUp(2))?
            .execute(MoveTo(0, 0))?
            .execute(Clear(ClearType::CurrentLine))?;
        write!(
            stdout,
            "The {method} request has sent {} times",
            counter.load(Ordering::Relaxed)
        )?;

        stdout.flush()?;

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
