use byte_unit::{Byte, UnitType};
use crossterm::{
    ExecutableCommand,
    cursor::{self, MoveTo},
    terminal::{Clear, ClearType},
};
use std::sync::Arc;
use std::{
    io::{Write, stdout},
    sync::atomic::Ordering,
    time::Duration,
};

pub enum OutputMode {
    Terminal { refresh_rate_ms: u64 },
    Normal { refresh_rate_ms: u64 },
}

impl Default for OutputMode {
    fn default() -> Self {
        OutputMode::Terminal {
            refresh_rate_ms: 200,
        }
    }
}

pub async fn terminal_output(
    method: reqwest::Method,
    shutdown_signal: tokio::sync::watch::Receiver<bool>,
    statistic: Arc<crate::statistic::Statistic>,
) -> anyhow::Result<()> {
    output_statistics(
        method,
        shutdown_signal,
        OutputMode::Terminal {
            refresh_rate_ms: 200,
        },
        statistic,
    )
    .await
}

pub async fn normal_output(
    method: reqwest::Method,
    shutdown_signal: tokio::sync::watch::Receiver<bool>,
    statistic: Arc<crate::statistic::Statistic>,
) -> anyhow::Result<()> {
    output_statistics(
        method,
        shutdown_signal,
        OutputMode::Normal {
            refresh_rate_ms: 2000,
        },
        statistic,
    )
    .await
}

async fn output_statistics(
    method: reqwest::Method,
    shutdown_signal: tokio::sync::watch::Receiver<bool>,
    mode: OutputMode,
    s: Arc<crate::statistic::Statistic>,
) -> anyhow::Result<()> {
    let counter = &s.request_counter;
    let sc = &s.status_counter;
    let network_traffics = &s.network_traffics;

    // Initial delay before starting output
    tokio::time::sleep(Duration::from_secs(4)).await;

    let mut stdout = stdout();
    writeln!(stdout)?;
    stdout.flush()?;

    // Only get cursor position when using terminal mode
    let cursor_y = match mode {
        OutputMode::Terminal { .. } => Some(cursor::position()?.1),
        OutputMode::Normal { .. } => None,
    };

    loop {
        if *shutdown_signal.borrow() {
            break Ok(());
        }

        // Clear screen if in terminal mode
        if let Some(y) = cursor_y {
            stdout
                .execute(MoveTo(0, y))?
                .execute(Clear(ClearType::FromCursorDown))?;
        }

        // Write request count
        write!(
            stdout,
            "The {} request has sent {} times",
            method,
            counter.load(Ordering::Relaxed)
        )?;

        // Write status counts
        write!(
            stdout,
            "\nrequest status counter's results: 2xx: {} 3xx: {} 4xx: {} 5xx: {} timeout: {}",
            sc.status_2xx.load(Ordering::Relaxed),
            sc.status_3xx.load(Ordering::Relaxed),
            sc.status_4xx.load(Ordering::Relaxed),
            sc.status_5xx.load(Ordering::Relaxed),
            sc.status_other.load(Ordering::Relaxed)
        )?;

        // Write network traffic
        let byte = Byte::from_u64(network_traffics.load(Ordering::Relaxed));
        write!(
            stdout,
            "\nnetwork traffics: {} bytes, Human readable: {}",
            byte,
            byte.get_appropriate_unit(UnitType::Decimal)
        )?;

        writeln!(stdout)?;
        stdout.flush()?;

        // Sleep according to the specified mode
        let sleep_duration = match mode {
            OutputMode::Terminal { refresh_rate_ms } => refresh_rate_ms,
            OutputMode::Normal { refresh_rate_ms } => refresh_rate_ms,
        };

        tokio::time::sleep(Duration::from_millis(sleep_duration)).await;
    }
}
