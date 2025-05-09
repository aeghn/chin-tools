pub mod event_stream;
pub mod model;

use std::ops::Deref;

use hashbrown::{HashMap, HashSet};
use model::*;
use niri_ipc::Event;

use super::{
    InnerEquals, WLEvent, WLWindow, WLWindowBehaiver, WLWindowId, WLWorkspace, WLWorkspaceBehaiver,
    WLWorkspaceId,
};

#[derive(Clone, Debug)]
pub struct NiriWindowWrapper(NiriWindow);

#[derive(Default)]
pub struct NiriCompositor {
    inited: bool,
    focused_winid: Option<WLWindowId>,
    workspaces: HashMap<WLWorkspaceId, WLWorkspace>,
    windows: HashMap<WLWorkspaceId, WLWindow>,
}

impl Deref for NiriWindowWrapper {
    type Target = NiriWindow;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<NiriWindow> for NiriWindowWrapper {
    fn from(value: NiriWindow) -> Self {
        Self(value)
    }
}

impl PartialEq for NiriWindowWrapper {
    fn eq(&self, other: &Self) -> bool {
        let s = &self.0;
        let o = &other.0;
        s.equal(o)
    }
}

impl InnerEquals for NiriWindow {
    fn equal(&self, o: &Self) -> bool {
        let s = self;
        s.is_floating == o.is_floating
            && s.is_focused == o.is_focused
            && s.app_id == o.app_id
            && s.id == o.id
            && s.pid == o.pid
            && s.title == o.title
            && s.workspace_id == o.workspace_id
    }
}

impl NiriCompositor {
    fn apply_workspaces(&mut self, wss: Vec<WLWorkspace>) -> Vec<WLEvent> {
        let mut events = vec![];
        for new in wss {
            let old = self.workspaces.get(&new.get_id());
            if old.map(|e| *e != new).unwrap_or(true) {
                events.push(WLEvent::WorkspaceOverwrite(new.clone()));
                self.workspaces.insert(new.get_id(), new);
            }
        }

        events
    }

    fn get_old_focused_workspaces(&self) -> Vec<WLWorkspace> {
        let old_focused: Vec<WLWorkspace> = self
            .workspaces
            .values()
            .filter(|e| e.is_focused)
            .map(|e| WLWorkspace {
                is_focused: false,
                ..e.clone()
            })
            .collect();

        old_focused
    }

    fn change_focused_window(
        &mut self,
        new_focused: Option<WLWindow>,
        handle_new: bool,
    ) -> Vec<WLEvent> {
        let mut events: Vec<_> = vec![];

        let old_focused: Vec<WLWindow> = self
            .windows
            .values()
            .filter(|e| e.is_focused && Some(*e) != new_focused.as_ref())
            .map(|e| e.clone())
            .collect();

        for ele in old_focused {
            let ele = NiriWindowWrapper(NiriWindow {
                is_focused: false,
                ..ele.0
            });
            self.windows.insert(ele.get_id(), ele.clone());
            events.push(WLEvent::WindowOverwrite(ele));
        }

        if let Some(ws) = new_focused {
            if handle_new {
                let old = self.windows.insert(ws.get_id(), ws.clone());
                if old.as_ref() != Some(&ws) {
                    events.push(WLEvent::WindowOverwrite(ws));
                }
            }
        }

        events
    }

    pub fn handle_event(&mut self, event: Event) -> Option<Vec<WLEvent>> {
        let all_windows: &mut HashMap<WLWorkspaceId, WLWindow> = &mut self.windows;

        log::debug!("niri event {:?}", event);

        let mapped = match event {
            Event::WorkspacesChanged { workspaces } => {
                let mut events: Vec<_> = vec![];

                let new_set: HashSet<_> = workspaces.iter().map(|e| e.get_id()).collect();
                for (id, _) in &self.workspaces {
                    if !new_set.contains(id) {
                        events.push(WLEvent::WorkspaceDelete(id.clone()));
                    }
                }
                self.workspaces.retain(|e, _| new_set.contains(e));

                events.extend(self.apply_workspaces(workspaces));

                Some(events)
            }
            Event::WorkspaceActivated { id, focused } => {
                let mut wss = vec![];
                if focused {
                    wss.extend(self.get_old_focused_workspaces());
                } else {
                    log::debug!("workspace {} focused: {}", id, focused);
                }

                if let Some(ws) = self.workspaces.get(&id) {
                    let changed = NiriWorkspace {
                        is_focused: focused,
                        ..ws.clone()
                    };
                    wss.push(changed);
                } else {
                    log::warn!("new focused workspace is not existed {}", id);
                }

                Some(self.apply_workspaces(wss))
            }
            Event::WorkspaceActiveWindowChanged {
                workspace_id: _,
                active_window_id: _,
            } => None,
            Event::WindowsChanged { windows } => {
                let mut events: Vec<_> = vec![];

                for window in windows {
                    let window: NiriWindowWrapper = window.into();
                    let ow = self.windows.get(&window.get_id());
                    if ow.map_or(true, |e| !e.equal(&window)) {
                        self.windows.insert(window.get_id(), window.clone());
                        events.push(WLEvent::WindowOverwrite(window.clone()));
                    }
                    if window.is_focused {
                        self.change_focused_window(Some(window), false);
                    }
                }
                Some(events)
            }
            Event::WindowOpenedOrChanged { window } => {
                let events: Vec<WLEvent> = self.change_focused_window(Some(window.into()), true);
                Some(events)
            }
            Event::WindowClosed { id } => {
                all_windows.remove(&id);
                Some(vec![WLEvent::WindowDelete(id)])
            }
            Event::WindowFocusChanged { id } => {
                let win = id.and_then(|e| self.windows.remove(&e));
                let win = win.map(|mut e| {
                    e.0.is_focused = true;
                    e
                });
                let events = self.change_focused_window(win, true);

                Some(events)
            }
            Event::KeyboardLayoutsChanged {
                keyboard_layouts: _,
            } => None,
            Event::KeyboardLayoutSwitched { idx: _ } => None,
        };

        if self.inited {
            mapped
        } else {
            let mut result = vec![];
            for (_, w) in &self.workspaces {
                result.push(WLEvent::WorkspaceOverwrite(w.clone()));
            }
            for (_, w) in &self.windows {
                result.push(WLEvent::WindowOverwrite(w.clone()));
            }
            if let Some(es) = mapped {
                result.extend(es);
            }

            self.inited = true;

            Some(result)
        }
    }
}
