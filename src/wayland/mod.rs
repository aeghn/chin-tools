use hashbrown::HashMap;
use std::collections::HashSet;

use crate::wrapper::anyhow::AResult;

#[cfg(feature = "wayland-niri")]
pub mod niri;

use anyhow::anyhow;
use niri::model::Output as NiriOutput;
use niri::model::Window as NiriWindow;
use niri::model::Workspace as NiriWorkspace;

pub enum WLCompositor {
    Niri,
}

impl WLCompositor {
    pub fn current() -> AResult<Self> {
        if let Ok(_) = std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
            Err(anyhow!("hyprland not implemented"))
        } else if let Ok(_) = std::env::var("NIRI_SOCKET") {
            return Ok(Self::Niri);
        } else {
            Err(anyhow!("unknown not implemented"))
        }
    }

    pub fn get_all_windows(&self) -> AResult<Vec<WLWindow>> {
        match self {
            WLCompositor::Niri => {
                NiriWindow::get_all().map(|e| e.into_iter().map(|e| WLWindow::Niri(e)).collect())
            }
        }
    }
    pub fn get_focused_window(&self) -> Option<WLWindow> {
        match self {
            WLCompositor::Niri => NiriWindow::get_focused().map(|e| WLWindow::Niri(e)).ok(),
        }
    }
    pub fn get_all_workspaces(&self) -> AResult<Vec<WLWorkspace>> {
        match self {
            WLCompositor::Niri => NiriWorkspace::get_all()
                .map(|e| e.into_iter().map(|e| WLWorkspace::Niri(e)).collect()),
        }
    }
    pub fn get_all_outputs(&self) -> AResult<Vec<WLOutput>> {
        match self {
            WLCompositor::Niri => {
                NiriOutput::get_all().map(|e| e.into_iter().map(|e| WLOutput::Niri(e)).collect())
            }
        }
    }

    pub fn get_focused_workspace(&self) -> AResult<WLWorkspace> {
        match self {
            WLCompositor::Niri => NiriWorkspace::get_focused().map(|e| WLWorkspace::Niri(e)),
        }
    }

    pub fn get_focused_output(&self) -> AResult<WLOutput> {
        match self {
            WLCompositor::Niri => {
                let ws = NiriWorkspace::get_focused()?;

                NiriOutput::get_all()?
                    .into_iter()
                    .find(|e| Some(&e.name) == ws.output.as_ref())
                    .map(|e| WLOutput::Niri(e))
                    .ok_or(anyhow!("Unable to read this output"))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum WLWindow {
    Niri(NiriWindow),
}

impl WLWindow {
    pub fn focus(&self) -> AResult<()> {
        match self {
            WLWindow::Niri(window) => {
                window.focus()?;
            }
        }

        Ok(())
    }

    pub fn get_title(&self) -> Option<String> {
        match self {
            WLWindow::Niri(window) => window.title.clone(),
        }
    }

    pub fn get_app_id(&self) -> Option<String> {
        match self {
            WLWindow::Niri(window) => window.app_id.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum WLWorkspace {
    Niri(NiriWorkspace),
}

impl WLWorkspace {
    pub fn focus(&self) -> AResult<()> {
        Ok(())
    }

    pub fn get_id(&self) -> u64 {
        match self {
            WLWorkspace::Niri(workspace) => workspace.id,
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            WLWorkspace::Niri(workspace) => workspace
                .name
                .clone()
                .unwrap_or_else(|| workspace.idx.to_string()),
        }
    }

    pub fn is_active(&self) -> bool {
        match self {
            WLWorkspace::Niri(workspace) => workspace.is_active,
        }
    }

    pub fn get_output_name(&self) -> Option<String> {
        match self {
            WLWorkspace::Niri(workspace) => workspace.output.clone(),
        }
    }

    pub fn is_focused(&self) -> bool {
        match self {
            WLWorkspace::Niri(workspace) => workspace.is_focused,
        }
    }
}

#[derive(Debug, Clone)]
pub enum WLOutput {
    Niri(NiriOutput),
}

impl WLOutput {
    pub fn focus(&self) -> AResult<()> {
        Ok(())
    }

    pub fn get_name(&self) -> &str {
        match self {
            WLOutput::Niri(output) => &output.name,
        }
    }
}

#[derive(Clone, Debug)]
pub enum WLEvent {
    WorkspaceDeleted(WLWorkspace),
    WorkspaceAdded(WLWorkspace),
    WorkspaceChanged(WLWorkspace),
    WorkspaceFocused(WLWorkspace),
    WindowFocused(Option<WLWindow>),
    MonitorFocused(WLOutput),
}

pub fn into_wl_event(
    event: niri::model::Event,
    all_workspaces: &mut HashMap<u64, NiriWorkspace>,
    all_windows: &mut HashMap<u64, NiriWindow>,
) -> Option<Vec<WLEvent>> {
    match event {
        niri::model::Event::WorkspacesChanged { workspaces } => {
            let mut events = vec![];
            let new_set: HashSet<u64> = workspaces.iter().map(|w| w.id).collect();

            let mut old = HashMap::new();
            for (k, v) in all_workspaces.drain() {
                if new_set.contains(&k) {
                    old.insert(k, v);
                } else {
                    events.push(WLEvent::WorkspaceDeleted(WLWorkspace::Niri(v)));
                }
            }
            *all_workspaces = old;

            for ws in workspaces.into_iter() {
                let old = { all_workspaces.get(&ws.id) };
                if ws.is_focused {
                    events.push(WLEvent::WorkspaceFocused(WLWorkspace::Niri(ws.clone())));
                }
                if old.is_some() && old != Some(&ws) {
                    all_workspaces.insert(ws.id, ws.clone());
                    events.push(WLEvent::WorkspaceChanged(WLWorkspace::Niri(ws)));
                } else if old.is_none() {
                    all_workspaces.insert(ws.id, ws.clone());
                    events.push(WLEvent::WorkspaceAdded(WLWorkspace::Niri(ws)));
                }
            }
            Some(events)
        }
        niri::model::Event::WorkspaceActivated { id, focused } => {
            if focused {
                tracing::error!("all workspace ids: {:?}", all_workspaces.keys());
                all_workspaces
                    .get(&id)
                    .map(|e| vec![WLEvent::WorkspaceFocused(WLWorkspace::Niri(e.clone()))])
            } else {
                None
            }
        }
        niri::model::Event::WorkspaceActiveWindowChanged {
            workspace_id: _,
            active_window_id,
        } => {
            if let Some(wid) = active_window_id {
                let items = WLEvent::WindowFocused(
                    all_windows.get(&wid).map(|e| WLWindow::Niri(e.clone())),
                );
                Some(vec![items])
            } else {
                None
            }
        }
        niri::model::Event::WindowsChanged { windows } => {
            for win in windows {
                all_windows.insert(win.id, win);
            }
            None
        }
        niri::model::Event::WindowOpenedOrChanged { window } => {
            all_windows.insert(window.id, window);
            None
        }
        niri::model::Event::WindowClosed { id } => {
            all_windows.remove(&id);
            None
        }
        niri::model::Event::WindowFocusChanged { id } => {
            if let Some(id) = id {
                if let Some(win) = all_windows.get(&id) {
                    let win = WLEvent::WindowFocused(Some(WLWindow::Niri(win.clone())));
                    Some(vec![win])
                } else {
                    Some(vec![WLEvent::WindowFocused(None)])
                }
            } else {
                Some(vec![WLEvent::WindowFocused(None)])
            }
        }
        niri::model::Event::KeyboardLayoutsChanged {
            keyboard_layouts: _,
        } => None,
        niri::model::Event::KeyboardLayoutSwitched { idx: _ } => None,
    }
}

#[derive(Debug)]
pub struct CurrentStatus {
    pub workspace: WLWorkspace,
    pub output: WLOutput,
    pub window: Option<WLWindow>,
}

impl CurrentStatus {
    pub fn new(com: WLCompositor) -> AResult<Self> {
        Ok(Self {
            workspace: com.get_focused_workspace()?,
            output: com.get_focused_output()?,
            window: com.get_focused_window(),
        })
    }
}
