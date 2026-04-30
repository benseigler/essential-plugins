use std::{
    path::PathBuf,
    sync::{Arc, atomic::Ordering},
};

use crossbeam_channel::{Receiver, Sender};
use nih_plug::prelude::*;
use nih_plug_vizia::{ViziaState, ViziaTheming, create_vizia_editor, vizia::prelude::*};

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (800, 600))
}

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
#[derive(Clone, Lens)]
pub struct UiData {
    params: Arc<PluginParams>,
    export_path: PathBuf,
    send_from_ui: Sender<FromEditorMessage>,
    recv_to_ui: Receiver<ToEditorMessage>,
    exporting: bool,
}

impl UiData {
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
}
impl Model for UiData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        self.update_with_messages();
        event.map(|event, _| match event {
            UiEvent::SetExportPath => pick_target(self),
            UiEvent::ExportScene => {
                if self.exporting {
                    self.exporting = false;
                    return self.send_message(FromEditorMessage::ExportUnarm);
                }
                self.exporting = true;
                return self.send_message(FromEditorMessage::ExportArm);
            }
            UiEvent::SetSceneStart => {
                self.send_message(FromEditorMessage::SetSceneStart);
            }
            UiEvent::SetSceneEnd => {
                self.send_message(FromEditorMessage::SetSceneEnd);
            }
            UiEvent::Tick => {}
        })
    }
}

enum UiEvent {
    SetSceneStart,
    SetSceneEnd,
    SetExportPath,
    ExportScene,
    Tick,
}

fn pick_target(data: &mut UiData) {
    let sender = data.send_from_ui.clone();
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

pub fn create(
    params: Arc<PluginParams>,
    send_from_ui: Sender<FromEditorMessage>,
    recv_to_ui: Receiver<ToEditorMessage>,
    exporting: bool,
    vizia_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(vizia_state, ViziaTheming::Builtin, move |cx, _| {
        let roboto = include_bytes!("../../../assets/font/Roboto-VariableFont_wdth,wght.ttf");
        cx.add_font_mem(roboto);
        cx.set_default_font(&["Roboto"]);
        let data = UiData {
            params: params.clone(),
            export_path: params.export_path.lock().unwrap().clone(),
            // scene_start: params.scene_start.load(Ordering::Acquire),
            // scene_end: params.scene_end.load(Ordering::Acquire),
            send_from_ui: send_from_ui.clone(),
            recv_to_ui: recv_to_ui.clone(),
            exporting,
        };
        data.build(cx);
        Button::new(
            cx,
            |cx| cx.emit(UiEvent::SetExportPath),
            |cx| Label::new(cx, "Set Export Path"),
        );
        Label::new(
            cx,
            UiData::export_path.map(|path| path.to_string_lossy().into_owned()),
        );
        Button::new(
            cx,
            |cx| cx.emit(UiEvent::ExportScene),
            |cx| Label::new(cx, "Export"),
        );
        Label::new(
            cx,
            UiData::exporting.map(|exporting| {
                match exporting {
                    true => return "Export armed",
                    false => return "Not exporting.",
                };
            }),
        );
        Button::new(
            cx,
            |cx| cx.emit(UiEvent::SetSceneStart),
            |cx| Label::new(cx, "Set Scene Start"),
        );
        Label::new(
            cx,
            UiData::params.map(|params| params.scene_start.load(Ordering::Acquire)),
        );
        Button::new(
            cx,
            |cx| cx.emit(UiEvent::SetSceneEnd),
            |cx| Label::new(cx, "Set Scene End"),
        );
        Label::new(
            cx,
            UiData::params.map(|params| params.scene_end.load(Ordering::Acquire)),
        );
        let timer = cx.add_timer(REFRESH_RATE, None, |ctx, _action| {
            ctx.emit(UiEvent::Tick);
        });
        cx.start_timer(timer);
    })
}

const fn get_refresh_rate_tick(refresh_rate: u64) -> Duration {
    let nanoseconds = 1_000_000_000 / refresh_rate;
    Duration::from_nanos(nanoseconds)
}

const REFRESH_RATE: Duration = get_refresh_rate_tick(120);
