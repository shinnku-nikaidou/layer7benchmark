use byte_unit::{Byte, UnitType};
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

pub async fn terminal_output(
    method: reqwest::Method,
    shutdown_signal: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<()> {
    let s = STATISTIC.get().unwrap();
    let counter = &s.request_counter;
    let sc = &s.status_counter;
    let network_traffics = &s.network_traffics;

    tokio::time::sleep(Duration::from_secs(4)).await;
    let mut stdout = stdout();
    write!(stdout, "\n")?;
    stdout.flush()?;
    let (_, y) = cursor::position()?;
    loop {
        if *shutdown_signal.borrow() {
            break Ok(());
        }

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

        let byte = Byte::from_u64(network_traffics.load(Ordering::Relaxed));

        write!(
            stdout,
            "\nnetwork traffics: {} bytes, Human readable: {}",
            byte,
            byte.get_appropriate_unit(UnitType::Decimal)
        )?;

        stdout.flush()?;

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
