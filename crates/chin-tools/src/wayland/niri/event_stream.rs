use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use anyhow::anyhow;
use niri_ipc::Event;

use crate::{niri_msg, wrapper::anyhow::EResult};

pub fn handle_event_stream<F>(mut func: F) -> EResult
where
    F: FnMut(Event),
{
    let child = niri_msg!()
        .arg("event-stream")
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .ok_or_else(|| anyhow!("Could not capture standard output."))?;

    let reader = BufReader::new(stdout);

    tracing::info!("begin to read niri stream");
    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| {
            let event: Result<Event, serde_json::Error> = serde_json::from_str(&line);
            match event {
                Ok(e) => func(e),
                Err(e) => {
                    tracing::warn!("unable to convert {} to event, {}", line, e)
                }
            }
        });

    Ok(())
}

#[cfg(test)]
mod test {
    use super::handle_event_stream;

    #[test]
    fn read_events() {
        handle_event_stream(|e| {
            tracing::info!("{:?}", e);
        })
        .unwrap();
    }
}
