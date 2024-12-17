use std::{collections::HashMap, fmt::Debug, process::Command};

use anyhow::anyhow;
use model::{Output, Window, Workspace};
use serde::de::DeserializeOwned;

use crate::wrapper::anyhow::AResult;

pub mod event_stream;
pub mod model;

#[macro_export]
macro_rules! niri_msg {
    () => {
        Command::new("niri").arg("msg").arg("-j")
    };
}

#[macro_export]
macro_rules! niri_action {
    () => {
        niri_msg!().arg("action")
    };
}

fn json_output<T>(cmd: &mut Command) -> AResult<T>
where
    T: DeserializeOwned + Debug,
{
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(anyhow!("command exited with {:?}", output.status.code()));
    }
    let stdout = String::from_utf8_lossy(output.stdout.as_slice());

    Ok(serde_json::from_str(&stdout)?)
}

impl Window {
    pub fn get_all() -> AResult<Vec<Window>> {
        json_output(Command::new("niri").arg("msg").arg("--json").arg("windows"))
    }

    pub fn get_focused() -> AResult<Window> {
        json_output(niri_msg!().arg("focused-window"))
    }

    pub fn focus(&self) -> AResult<()> {
        
        json_output(niri_action!().args(["focus-window", "--id", &format!("{}", self.id)]))
            
    }
}


impl Workspace {
    pub fn get_all() -> AResult<Vec<Self>> {
        json_output(niri_msg!().arg("workspaces"))
    }

    pub fn focus(&self) -> AResult<()> {
        json_output(niri_action!().args(["focus-workspace", &format!("{}", self.id)]))
    }

    pub fn get_focused() -> AResult<Workspace> {
        Workspace::get_all().and_then(|e| {
            e.into_iter()
                .find(|e| e.is_focused)
                .ok_or(anyhow!("no focused workspace???"))
        })
    }
}



impl Output {
    pub fn get_all() -> AResult<Vec<Output>> {
        let ret: HashMap<String, Output> = json_output(niri_msg!().arg("outputs"))?;
        Ok(ret.values().into_iter().map(|e| e.clone()).collect())
    }
}

#[cfg(test)]
mod test {
    use crate::wayland::niri::model::Window;

    #[test]
    fn get_all() {
        println!("{:?}", Window::get_all())
    }

    #[test]
    fn get_focused() {
        println!("{:?}", Window::get_focused())
    }

    use crate::wayland::niri::model::Workspace;

    #[test]
    fn get_all_wss() {
        println!("{:?}", Workspace::get_all())
    }
}
