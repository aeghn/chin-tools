pub mod event_stream;
pub mod model;

use std::ops::Deref;

use hashbrown::HashMap;
use model::*;
use niri_ipc::Event;

use super::{
    InnerEquals, WLEvent, WLWindow, WLWindowBehaiver, WLWindowId, WLWorkspace, WLWorkspaceBehaiver,
    WLWorkspaceId,
};

#[derive(Clone, Debug)]
pub struct NiriWindowWrapper(NiriWindow);

#[derive(Default)]
pub struct NiriInstance {
    inited: bool,
    focused_wsid: WLWorkspaceId,
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

impl NiriInstance {
    fn change_focused_workspace(&mut self, new_focused: Option<&WLWorkspace>) -> Vec<WLEvent> {
        let mut events: Vec<_> = vec![];

        if let Some(ws) = new_focused {
            let old_focused: Vec<&mut WLWorkspace> = self
                .workspaces
                .values_mut()
                .filter(|e| e.is_focused && *e != ws)
                .collect();

            if !old_focused.is_empty() {
                for ele in old_focused {
                    ele.is_focused = false;
                    events.push(WLEvent::WorkspaceOverwrite(ele.clone()));
                }
            }
            self.workspaces.insert(ws.get_id(), ws.clone());
            events.push(WLEvent::WorkspaceOverwrite(ws.clone()));

            self.focused_wsid = ws.get_id();
        }

        events
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
        let all_workspaces: &mut HashMap<WLWorkspaceId, WLWorkspace> = &mut self.workspaces;
        let all_windows: &mut HashMap<WLWorkspaceId, WLWindow> = &mut self.windows;
        let mapped = match event {
            Event::WorkspacesChanged { workspaces } => {
                let mut events: Vec<_> = vec![];
                let mut new_focused = None;
                for ws in workspaces.into_iter() {
                    if ws.is_focused {
                        new_focused.replace(ws.clone());
                    }
                    let ows = self.workspaces.get(&ws.get_id());
                    if Some(&ws) != ows || ows.is_none() {
                        self.workspaces.insert(ws.get_id(), ws.clone());
                        events.push(WLEvent::WorkspaceOverwrite(ws));
                    }
                }
                events.extend(self.change_focused_workspace(new_focused.as_ref()));

                Some(events)
            }
            Event::WorkspaceActivated { id, focused } => {
                let ows = all_workspaces.remove(&id);

                let mut events: Vec<_> = vec![];

                if let Some(ws) = ows {
                    let ws1 = NiriWorkspace {
                        is_focused: focused,
                        ..ws.clone()
                    };
                    if ws1 != ws {
                        events.push(WLEvent::WorkspaceOverwrite(ws1.clone()));
                        all_workspaces.insert(id, ws1);
                    }
                }

                Some(events)
            }
            Event::WorkspaceActiveWindowChanged {
                workspace_id,
                active_window_id,
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
