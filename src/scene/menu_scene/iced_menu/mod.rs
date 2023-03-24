use std::{path::PathBuf, rc::Rc};

use iced_graphics::{
    alignment::{Horizontal, Vertical},
    Alignment,
};
use iced_native::{
    column as col,
    image::Handle as ImageHandle,
    row,
    widget::{self, button, checkbox, container, image, pick_list, slider, text, vertical_space, scrollable},
    Command, Length, Padding,
};

use crate::{
    config::PlayingSceneLayout,
    output_manager::OutputDescriptor,
    scene::menu_scene::neo_btn::neo_button,
    target::Target,
    ui::iced_state::{Element, Program},
    NeothesiaEvent,
};

mod theme;

type InputDescriptor = midi_io::MidiInputPort;

#[derive(Debug, Clone)]
pub enum Message {
    Tick,

    SelectOutput(OutputDescriptor),
    SelectInput(InputDescriptor),

    OpenMidiFilePicker,
    MidiFileLoaded(Option<lib_midi::Midi>),

    OpenSoundFontPicker,
    SoundFontFileLoaded(Option<PathBuf>),

    Play,

    WaitForNotesCheckbox(bool),
    GuideNotesCheckbox(bool),
    MuteDrumsCheckbox(bool),

    DrumsVolumeSlider(u8),
    MusicVolumeSlider(u8),
    MetronomeVolumeSlider(u8),

    SelectLayout(PlayingSceneLayout),

    GoToPage(Step),
    ExitApp,
}

struct Data {
    outputs: Vec<OutputDescriptor>,
    selected_output: Option<OutputDescriptor>,
    font_path: Option<PathBuf>,
    midi_file: Option<Rc<lib_midi::Midi>>,

    inputs: Vec<InputDescriptor>,
    selected_input: Option<InputDescriptor>,

    wait_for_notes: bool,
    guide_notes: bool,
    mute_drums: bool,
    is_loading: bool,

    drums_volume: u8,
    music_volume: u8,
    metronome_volume: u8,

    layouts: Vec<PlayingSceneLayout>,
    selected_layout: PlayingSceneLayout,

    logo_handle: ImageHandle,
}

pub struct AppUi {
    data: Data,
    current: Step,
}

impl AppUi {
    pub fn new(target: &mut Target) -> Self {
        Self {
            current: Step::Main,
            data: Data {
                outputs: Vec::new(),
                selected_output: None,
                font_path: target.config.soundfont_path.clone(),
                midi_file: target.midi_file.clone(),

                inputs: Vec::new(),
                selected_input: None,

                wait_for_notes: target.config.wait_for_notes,
                guide_notes: target.config.guide_notes,
                mute_drums: target.config.mute_drums,

                drums_volume: target.config.drums_volume,
                music_volume: target.config.music_volume,
                metronome_volume: (100.0 * target.config.metronome_volume) as u8,

                layouts: vec![PlayingSceneLayout::Horizontal, PlayingSceneLayout::Vertical],
                selected_layout: target.config.layout,
                is_loading: false,

                logo_handle: ImageHandle::from_memory(include_bytes!("../img/banner.png").to_vec()),
            },
        }
    }
}

impl Program for AppUi {
    type Message = Message;

    fn update(&mut self, target: &mut Target, message: Message) -> Command<Self::Message> {
        match message {
            Message::GoToPage(page) => {
                self.current = page;
            }
            Message::Play => {
                if self.data.midi_file.is_some() {
                    target.midi_file = self.data.midi_file.take();

                    if let Some(out) = self.data.selected_output.clone() {
                        let out = match out {
                            #[cfg(feature = "synth")]
                            OutputDescriptor::Synth(_) => {
                                OutputDescriptor::Synth(self.data.font_path.clone())
                            }
                            o => o,
                        };

                        target.output_manager.borrow_mut().connect(out);
                    }

                    if let Some(port) = self.data.selected_input.clone() {
                        target.input_manager.connect_input(port);
                    }

                    target
                        .proxy
                        .send_event(NeothesiaEvent::MainMenu(super::Event::Play));
                }
            }
            Message::OpenMidiFilePicker => {
                self.data.is_loading = true;
                return open_midi_file_picker(Message::MidiFileLoaded);
            }
            Message::MidiFileLoaded(midi) => {
                if let Some(midi) = midi {
                    self.data.midi_file = Some(Rc::new(midi));
                }
                self.data.is_loading = false;
            }
            Message::OpenSoundFontPicker => {
                self.data.is_loading = true;
                return open_sound_font_picker(Message::SoundFontFileLoaded);
            }
            Message::SoundFontFileLoaded(font) => {
                if let Some(font) = font {
                    target.config.soundfont_path = Some(font.clone());
                    self.data.font_path = Some(font);
                }
                self.data.is_loading = false;
            }
            Message::SelectOutput(output) => {
                target.config.set_output(&output);
                self.data.selected_output = Some(output);
            }
            Message::SelectInput(input) => {
                target.config.set_input(Some(&input));
                self.data.selected_input = Some(input);
            }
            Message::WaitForNotesCheckbox(v) => {
                target.config.wait_for_notes = v;
                self.data.wait_for_notes = v;
            }
            Message::GuideNotesCheckbox(v) => {
                target.config.guide_notes = v;
                self.data.guide_notes = v;
            }
            Message::MuteDrumsCheckbox(v) => {
                target.config.mute_drums = v;
                self.data.mute_drums = v;
            }
            Message::DrumsVolumeSlider(v) => {
                target.config.drums_volume = v;
                self.data.drums_volume = v;
            }
            Message::MusicVolumeSlider(v) => {
                target.config.music_volume = v;
                self.data.music_volume = v;
            }
            Message::MetronomeVolumeSlider(v) => {
                target.config.metronome_volume = v as f32 / 100.0;
                self.data.metronome_volume = v;
            }
            Message::SelectLayout(v) => {
                target.config.layout = v;
                self.data.selected_layout = v;
            }
            Message::Tick => {
                self.data.outputs = target.output_manager.borrow().outputs();
                self.data.inputs = target.input_manager.inputs();

                if self.data.selected_output.is_none() {
                    if let Some(out) = self
                        .data
                        .outputs
                        .iter()
                        .find(|output| Some(output.to_string()) == target.config.output)
                    {
                        self.data.selected_output = Some(out.clone());
                    } else {
                        self.data.selected_output = self.data.outputs.first().cloned();
                    }
                }

                if self.data.selected_input.is_none() {
                    if let Some(input) = self
                        .data
                        .inputs
                        .iter()
                        .find(|input| Some(input.to_string()) == target.config.input)
                    {
                        self.data.selected_input = Some(input.clone());
                    } else {
                        self.data.selected_input = self.data.inputs.first().cloned();
                    }
                }
            }
            Message::ExitApp => {
                target.proxy.send_event(NeothesiaEvent::GoBack);
            }
        }

        Command::none()
    }

    fn keyboard_input(&self, event: &iced_native::keyboard::Event) -> Option<Message> {
        use iced_native::keyboard::{Event, KeyCode};

        if let Event::KeyPressed { key_code, .. } = event {
            match key_code {
                KeyCode::Tab => match self.current {
                    Step::Main => Some(Message::OpenMidiFilePicker),
                    Step::Settings => Some(Message::OpenSoundFontPicker),
                    _ => None,
                },
                KeyCode::S => match self.current {
                    Step::Main => Some(Message::GoToPage(Step::Settings)),
                    _ => None,
                },
                KeyCode::A => match self.current {
                    Step::Main => Some(Message::WaitForNotesCheckbox(!self.data.guide_notes)),
                    _ => None,
                },
                KeyCode::D => match self.current {
                    Step::Main => Some(Message::GuideNotesCheckbox(!self.data.guide_notes)),
                    _ => None,
                },
                KeyCode::Enter => match self.current {
                    Step::Exit => Some(Message::ExitApp),
                    Step::Main => Some(Message::Play),
                    _ => None,
                },
                KeyCode::Escape => Some(match self.current {
                    Step::Exit => Message::GoToPage(Step::Main),
                    Step::Main => Message::GoToPage(Step::Exit),
                    Step::Settings => Message::GoToPage(Step::Main),
                }),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(&self) -> Element<Message> {
        self.current.view(&self.data)
    }
}

#[derive(Debug, Clone)]
pub enum Step {
    Exit,
    Main,
    Settings,
}

impl<'a> Step {
    fn view(&'a self, data: &'a Data) -> Element<Message> {
        if data.is_loading {
            return Self::loading(data);
        }

        match self {
            Self::Exit => Self::exit(),
            Self::Main => Self::main(data),
            Self::Settings => Self::settings(data),
        }
    }

    fn loading(data: &'a Data) -> Element<'a, Message> {
        let column = col![image(data.logo_handle.clone()), text("Loading...").size(50)]
            .spacing(30)
            .align_items(Alignment::Center);

        center_x(top_padded(column)).into()
    }

    fn exit() -> Element<'a, Message> {
        let output = centered_text("Do you want to exit?").size(30);

        let select_row = row![
            neo_button("No")
                .width(Length::Fill)
                .on_press(Message::GoToPage(Step::Main)),
            neo_button("Yes")
                .width(Length::Fill)
                .on_press(Message::ExitApp),
        ]
        .spacing(5)
        .height(Length::Units(50));

        let controls = col![output, select_row]
            .align_items(Alignment::Center)
            .width(Length::Units(650))
            .spacing(30);

        center_x(controls).center_y().into()
    }

    fn main(data: &'a Data) -> Element<'a, Message> {
        let buttons = col![
            neo_button("Select File")
                .on_press(Message::OpenMidiFilePicker)
                .width(Length::Fill)
                .height(Length::Units(80)),
            neo_button("Settings")
                .on_press(Message::GoToPage(Step::Settings))
                .width(Length::Fill)
                .height(Length::Units(80)),
            neo_button("Exit")
                .on_press(Message::GoToPage(Step::Exit))
                .width(Length::Fill)
                .height(Length::Units(80))
        ]
        .width(Length::Units(450))
        .spacing(10);

        let column = col![image(data.logo_handle.clone()), buttons]
            .spacing(40)
            .align_items(Alignment::Center);

        let mut content = top_padded(column);

        if data.midi_file.is_some() {
            let guide_notes =
                checkbox("Guide Notes", data.guide_notes, Message::GuideNotesCheckbox)
                    .style(theme::checkbox());

            let mute_drums = checkbox("Mute Drums", data.mute_drums, Message::MuteDrumsCheckbox)
                .style(theme::checkbox());

            let wait_for_notes = checkbox(
                "Wait For Notes",
                data.wait_for_notes,
                Message::WaitForNotesCheckbox,
            )
            .style(theme::checkbox());

            let play = neo_button("Play")
                .height(Length::Units(60))
                .min_width(80)
                .on_press(Message::Play);

            let row = row![guide_notes, mute_drums, wait_for_notes, play]
                .spacing(20)
                .align_items(Alignment::Center);

            let container = container(row)
                .width(Length::Fill)
                .align_x(Horizontal::Right)
                .padding(Padding {
                    top: 0,
                    right: 10,
                    bottom: 10,
                    left: 0,
                });

            content = content.push(container);
        }

        center_x(content).into()
    }

    fn settings(data: &'a Data) -> Element<'a, Message> {
        let outputs = &data.outputs;
        let selected_output = data.selected_output.clone();

        let is_synth = matches!(selected_output, Some(OutputDescriptor::Synth(_)));

        let output_list = pick_list(outputs, selected_output, Message::SelectOutput)
            .width(Length::Fill)
            .style(theme::pick_list());

        let output_title = text("Output:")
            .vertical_alignment(Vertical::Center)
            .height(Length::Units(30));

        let output_list = if is_synth {
            let btn = button(centered_text("SoundFont"))
                .width(Length::Units(50))
                .on_press(Message::OpenSoundFontPicker)
                .style(theme::button());

            row![
                output_title.width(Length::Units(60)),
                output_list.width(Length::FillPortion(3)),
                btn.width(Length::FillPortion(1))
            ]
        } else {
            row![output_title, output_list]
        }
        .spacing(10);

        let inputs = &data.inputs;
        let selected_input = data.selected_input.clone();

        let input_list = pick_list(inputs, selected_input, Message::SelectInput)
            .width(Length::Fill)
            .style(theme::pick_list());

        let input_title = text("Input:")
            .vertical_alignment(Vertical::Center)
            .height(Length::Units(30));

        let input_list = row![
            input_title.width(Length::Units(60)),
            input_list.width(Length::FillPortion(3)),
        ]
        .spacing(10);

        let selected_layout = Some(data.selected_layout);
        let layout_list = pick_list(&data.layouts, selected_layout, Message::SelectLayout)
            .width(Length::Fill)
            .style(theme::pick_list());

        let layout_title = text("Layout:")
            .vertical_alignment(Vertical::Center)
            .height(Length::Units(30));

        let layout_list = row![
            layout_title.width(Length::Units(60)),
            layout_list.width(Length::FillPortion(3)),
        ]
        .spacing(10);

        let metronome_volume_title = text("Metronome:")
            .vertical_alignment(Vertical::Center)
            .height(Length::Units(30));

        let metronome_volume =
            slider(0..=100, data.metronome_volume, Message::MetronomeVolumeSlider)
            .width(Length::Fill)
            .style(theme::slider());

        let metronome_volume_list = row![
            metronome_volume_title.width(Length::Units(120)),
            metronome_volume.width(Length::FillPortion(3))
        ]
        .spacing(10);

        let drums_volume_title = text("Drums:")
            .vertical_alignment(Vertical::Center)
            .height(Length::Units(30));

        let drums_volume =
            slider(0..=127, data.drums_volume, Message::DrumsVolumeSlider)
            .width(Length::Fill)
            .style(theme::slider());

        let drums_volume_list = row![
            drums_volume_title.width(Length::Units(120)),
            drums_volume.width(Length::FillPortion(3))
        ]
        .spacing(10);

        let music_volume_title = text("Music:")
            .vertical_alignment(Vertical::Center)
            .height(Length::Units(30));

        let music_volume = slider(0..=127, data.music_volume, Message::MusicVolumeSlider)
            .width(Length::Fill)
            .style(theme::slider());

        let music_volume_list = row![
            music_volume_title.width(Length::Units(120)),
            music_volume.width(Length::FillPortion(3))
        ]
        .spacing(10);

        let buttons = row![neo_button("Back")
            .on_press(Message::GoToPage(Step::Main))
            .width(Length::Fill),]
        .width(Length::Shrink)
        .height(Length::Units(50));

        let column = col![
            image(data.logo_handle.clone()),
            col![
                output_list,
                input_list,
                layout_list,
                drums_volume_list,
                music_volume_list,
                metronome_volume_list,
            ]
            .spacing(10),
            buttons
        ]
        .spacing(40)
        .align_items(Alignment::Center);

        center_x(top_padded(column)).into()
    }
}

fn centered_text<'a>(label: impl ToString) -> widget::Text<'a, iced_wgpu::Renderer> {
    text(label)
        .horizontal_alignment(Horizontal::Center)
        .vertical_alignment(Vertical::Center)
}

fn top_padded<'a, MSG: 'a>(
    content: impl Into<Element<'a, MSG>>,
) -> widget::Column<'a, MSG, iced_wgpu::Renderer> {
    let spacer = vertical_space(Length::FillPortion(1));
    let content = scrollable(content)
        .height(Length::FillPortion(7));

    col![spacer, content]
        .width(Length::Fill)
        .height(Length::Fill)
        .align_items(Alignment::Center)
}

fn center_x<'a, MSG: 'a>(
    content: impl Into<Element<'a, MSG>>,
) -> widget::Container<'a, MSG, iced_wgpu::Renderer> {
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
}

fn open_midi_file_picker(
    f: impl FnOnce(Option<lib_midi::Midi>) -> Message + 'static + Send,
) -> Command<Message> where
{
    Command::perform(
        async {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("midi", &["mid", "midi"])
                .pick_file()
                .await;

            if let Some(file) = file {
                log::info!("File path = {:?}", file.path());

                let thread = async_thread::Builder::new()
                    .name("midi-loader".into())
                    .spawn(move || {
                        let midi = lib_midi::Midi::new(file.path());

                        if let Err(e) = &midi {
                            log::error!("{}", e);
                        }

                        midi.ok()
                    });

                if let Ok(thread) = thread {
                    thread.join().await.ok().flatten()
                } else {
                    None
                }
            } else {
                log::info!("User canceled dialog");
                None
            }
        },
        f,
    )
}

fn open_sound_font_picker(
    f: impl FnOnce(Option<PathBuf>) -> Message + 'static + Send,
) -> Command<Message> where
{
    Command::perform(
        async {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("SoundFont2", &["sf2"])
                .pick_file()
                .await;

            if let Some(file) = file.as_ref() {
                log::info!("Font path = {:?}", file.path());
            } else {
                log::info!("User canceled dialog");
            }

            file.map(|f| f.path().to_owned())
        },
        f,
    )
}
