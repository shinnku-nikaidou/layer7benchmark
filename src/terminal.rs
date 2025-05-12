use std::{
    io::{stdout, Write},
    sync::atomic::Ordering,
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    cursor::{MoveTo, MoveUp},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use http::Method;
use tokio::sync::watch;

use crate::COUNTER;

pub async fn terminal_output(method: Method, mut shutdown: watch::Receiver<bool>) -> Result<()> {
    tokio::time::sleep(Duration::from_secs(6)).await;

    let mut stdout = stdout();

    loop {
        stdout
            .execute(MoveUp(2))?
            .execute(MoveTo(0, 0))?
            .execute(Clear(ClearType::CurrentLine))?;

        write!(
            stdout,
            "The {method} request has sent {} times",
            COUNTER.load(Ordering::Acquire)
        )?;

        stdout.flush()?;

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(200)) => { }
            _ = shutdown.changed() => {
                let shutdown = shutdown.borrow_and_update();

                if *shutdown {
                    break;
                }
            }
        }
    }

    Ok(())
}
