use std::{
    path::PathBuf,
    sync::{Arc, atomic::Ordering},
};

use crossbeam_channel::{Receiver, Sender};
use nice_plug_iced::{
    EditorState, NiceGuiContext,
    iced::{
        Theme,
        widget::{Column, button, column, text},
    },
};

#[derive(Debug, Clone)]
pub enum FromEditorMessage {
    SetSceneStart,
    SetSceneEnd,
    ExportArm,
    ExportUnarm,
    SetExportPath(PathBuf),
}

#[derive(Debug, Clone)]
pub enum ToEditorMessage {
    ExportUnarm,
    SetExportPath(PathBuf),
}

use crate::PluginParams;
#[derive(Clone)]
pub struct State {
    pub params: Arc<PluginParams>,
    pub export_path: PathBuf,
    pub send_from_ui: Sender<FromEditorMessage>,
    pub recv_to_ui: Receiver<ToEditorMessage>,
    pub exporting: bool,
}

pub struct Ui {
    editor_state: EditorState<State>,
    #[allow(unused)]
    nice_ctx: NiceGuiContext,
}
impl Ui {
    pub fn new(editor_state: EditorState<State>, nice_ctx: NiceGuiContext) -> Self {
        Self {
            editor_state,
            nice_ctx,
        }
    }
    pub fn update(&mut self, event: UiMessage) {
        self.editor_state.update(event);
    }
    pub fn view(&self) -> Column<'_, UiMessage> {
        self.editor_state.view()
    }
    pub fn theme(&self) -> Option<Theme> {
        Some(Theme::Dark)
    }
}

impl State {
    pub fn send_message(&mut self, msg: FromEditorMessage) {
        let _ = self.send_from_ui.send(msg);
    }
    pub fn update_with_messages(&mut self) {
        while let Ok(msg) = self.recv_to_ui.try_recv() {
            match msg {
                ToEditorMessage::ExportUnarm => self.exporting = false,
                ToEditorMessage::SetExportPath(path) => self.export_path = path,
            }
        }
    }
    fn update(&mut self, event: UiMessage) {
        match event {
            UiMessage::SetExportPath => pick_target(self),
            UiMessage::ExportScene => {
                if self.exporting {
                    self.exporting = false;
                    return self.send_message(FromEditorMessage::ExportUnarm);
                }
                self.exporting = true;
                return self.send_message(FromEditorMessage::ExportArm);
            }
            UiMessage::SetSceneStart => {
                self.send_message(FromEditorMessage::SetSceneStart);
            }
            UiMessage::SetSceneEnd => {
                self.send_message(FromEditorMessage::SetSceneEnd);
            }
            UiMessage::Poll => {
                self.update_with_messages();
            }
        }
    }
    fn view(&self) -> Column<'_, UiMessage> {
        column![
            button("Set Export Path").on_press(UiMessage::SetExportPath),
            text(self.export_path.to_string_lossy()),
            button("Export").on_press(UiMessage::ExportScene),
            text(match self.exporting {
                true => "Export armed",
                false => "Export unarmed",
            }),
            button("Set scene start").on_press(UiMessage::SetSceneStart),
            text(self.params.scene_start.load(Ordering::Acquire)),
            button("Set scene end").on_press(UiMessage::SetSceneEnd),
            text(self.params.scene_end.load(Ordering::Acquire)),
        ]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UiMessage {
    SetSceneStart,
    SetSceneEnd,
    SetExportPath,
    ExportScene,
    Poll,
}

fn pick_target(state: &mut State) {
    let sender = state.send_from_ui.clone();
    std::thread::spawn(move || {
        let file = rfd::FileDialog::new()
            .add_filter("xpans Spatial Record", &["xsr"])
            .set_title("Set export")
            .set_can_create_directories(true)
            .save_file();
        if let Some(file) = file {
            let path_buf = file.to_path_buf();
            let _ = sender.send(FromEditorMessage::SetExportPath(path_buf.clone()));
        }
    });
}
