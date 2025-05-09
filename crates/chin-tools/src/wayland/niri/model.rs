pub use niri_ipc::Output as NiriOutput;
pub use niri_ipc::Window as NiriWindow;
pub use niri_ipc::Workspace as NiriWorkspace;

use std::{collections::HashMap, fmt::Debug, process::Command};

use serde::de::DeserializeOwned;

use crate::eanyhow;
use crate::{
    wayland::*,
    wrapper::anyhow::{AResult, EResult},
};

use super::NiriCompositor;

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
        return Err(anyhow::anyhow!(
            "command exited with {:?}",
            output.status.code()
        ));
    }
    let stdout = String::from_utf8_lossy(output.stdout.as_slice());

    Ok(serde_json::from_str(&stdout)?)
}

impl WLWindowBehaiver for NiriWindow {
    fn focus(&self) -> EResult {
        json_output(niri_action!().args(["focus-window", "--id", &format!("{}", self.id)]))
    }

    fn get_title(&self) -> Option<&str> {
        self.title.as_ref().map(|e| e.as_str())
    }

    fn get_app_id(&self) -> Option<&str> {
        self.app_id.as_ref().map(|e| e.as_str())
    }

    fn get_id(&self) -> crate::wayland::WLWindowId {
        self.id
    }

    fn is_focused(&self) -> bool {
        self.is_focused
    }

    fn get_workspace_id(&self) -> Option<crate::wayland::WLWorkspaceId> {
        self.workspace_id
    }
}

impl WLWorkspaceBehaiver for NiriWorkspace {
    fn is_active(&self) -> bool {
        self.is_active
    }

    fn is_focused(&self) -> bool {
        self.is_focused
    }

    fn focus(&self) -> EResult {
        json_output(niri_action!().args(["focus-workspace", &format!("{}", self.idx)]))
    }

    fn get_id(&self) -> WLWorkspaceId {
        self.id
    }

    fn get_name(&self) -> String {
        if self.name.is_some() {
            self.name.as_ref().unwrap().to_owned()
        } else {
            self.idx.to_string()
        }
    }

    fn get_monitor_id(&self) -> Option<WLMonitorId> {
        self.output.clone()
    }
}

impl WLOutputBehaiver for NiriOutput {
    fn focus(&self) -> EResult {
        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

impl WLCompositorBehavier for NiriCompositor {
    fn fetch_all_windows(&self) -> crate::AResult<Vec<NiriWindowWrapper>> {
        json_output(Command::new("niri").arg("msg").arg("--json").arg("windows"))
            .map(|v: Vec<NiriWindow>| v.into_iter().map(|e| e.into()).collect())
    }

    fn fetch_focused_window(&self) -> crate::AResult<NiriWindowWrapper> {
        json_output(niri_msg!().arg("focused-window")).map(|v: NiriWindow| v.into())
    }

    fn fetch_all_workspaces(&self) -> crate::AResult<Vec<NiriWorkspace>> {
        json_output(niri_msg!().arg("workspaces"))
            .map(|e: Vec<NiriWorkspace>| e.into_iter().map(|w| w.into()).collect())
    }

    fn fetch_focused_workspace(&self) -> crate::AResult<NiriWorkspace> {
        self.fetch_all_workspaces().and_then(|e| {
            e.into_iter()
                .find(|e| e.is_focused())
                .ok_or(crate::anyhow::aanyhow!("no focused workspace???"))
        })
    }

    fn fetch_all_outputs(&self) -> AResult<Vec<WLOutput>> {
        let ret: HashMap<String, WLOutput> = json_output(niri_msg!().arg("outputs"))?;
        Ok(ret.into_iter().map(|(_, o)| o.into()).collect())
    }

    fn new() -> AResult<Self>
    where
        Self: Sized,
    {
        if let Ok(_) = std::env::var("NIRI_SOCKET") {
            let mut instance = NiriCompositor::default();
            let windows = instance.fetch_all_windows()?;
            let awin = instance.fetch_focused_window().ok();
            let workspaces = instance.fetch_all_workspaces()?;

            instance.windows = windows.into_iter().map(|e| (e.get_id(), e)).collect();
            instance.workspaces = workspaces.into_iter().map(|e| (e.get_id(), e)).collect();
            instance.focused_winid = awin.map(|e| e.get_id());

            Ok(instance)
        } else {
            eanyhow!("unkonwn")
        }
    }
}

#[cfg(test)]
mod test {
    use crate::wayland::{
        niri::model::{NiriWindow, NiriWorkspace},
        WLWindowBehaiver, WLWorkspaceBehaiver,
    };
}
