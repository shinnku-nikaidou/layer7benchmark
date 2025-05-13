use crossterm::{
    cursor::{self, MoveTo},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use std::{
    io::{stdout, Write},
    sync::atomic::Ordering,
    time::Duration,
};

use crate::statistic::STATISTIC;

pub async fn terminal_output(method: reqwest::Method) -> anyhow::Result<()> {
    let s = STATISTIC.get().unwrap().clone();
    let counter = &s.request_counter.clone();
    let sc = &s.status_counter.clone();

    tokio::time::sleep(Duration::from_secs(4)).await;
    let mut stdout = stdout();
    write!(stdout, "\n")?;
    stdout.flush()?;
    let (_, y) = cursor::position()?;
    loop {
        stdout
            .execute(MoveTo(0, y))?
            .execute(Clear(ClearType::FromCursorDown))?;
        write!(
            stdout,
            "The {} request has sent {} times",
            method,
            counter.load(Ordering::Relaxed)
        )?;

        write!(
            stdout,
            "\nrequest status counter's results: 2xx: {} 3xx: {} 4xx: {} 5xx: {} other: {}",
            sc.status_2xx.load(Ordering::Relaxed),
            sc.status_3xx.load(Ordering::Relaxed),
            sc.status_4xx.load(Ordering::Relaxed),
            sc.status_5xx.load(Ordering::Relaxed),
            sc.status_other.load(Ordering::Relaxed)
        )?;

        stdout.flush()?;

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
