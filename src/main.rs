use std::time::{Duration, Instant};

use druid::widget::prelude::*;
use druid::widget::{Align, Button, Flex, Label, ProgressBar, WidgetExt};
use druid::{AppLauncher, Data, Lens, LocalizedString, TimerToken, WindowDesc};

use rodio::Source;
use std::fs::File;
use std::io::BufReader;

static SECOND: Duration = Duration::from_millis(1000);
const BREAK_TIME: u32 = 5 * 60;
const WORK_TIME: u32 = 25 * 60;

struct TimerWidget {
    timer_id: TimerToken,
}

#[derive(Clone, Data, Lens)]
struct Pomodoro {
    seconds: u32,
    current_state: State,
    paused_state: State,
    progress: f64,
}

#[derive(Clone, Data, PartialEq, Debug, Copy)]
enum State {
    Working,
    Break,
    Paused,
}

impl Widget<Pomodoro> for TimerWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Pomodoro, _env: &Env) {
        match event {
            Event::WindowConnected => {
                // Start the timer when the application launches
                self.timer_id = ctx.request_timer(Instant::now() + SECOND);
            }
            Event::Timer(id) => {
                if *id == self.timer_id {
                    if data.current_state != State::Paused {
                        data.seconds = data.seconds - 1;
                    }
                    if data.seconds <= 0 {
                        blip();

                        if data.current_state == State::Working {
                            change_state(data, State::Break);
                        } else if data.current_state == State::Break {
                            change_state(data, State::Working);
                        }
                    }
                    let duration = time_from_state(data.current_state);
                    data.progress = (duration - data.seconds) as f64 / duration as f64;

                    self.timer_id = ctx.request_timer(Instant::now() + SECOND);
                    ctx.request_paint();
                }
            }
            _ => (),
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &Pomodoro,
        _env: &Env,
    ) {
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &Pomodoro, _data: &Pomodoro, _env: &Env) {
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &Pomodoro,
        _env: &Env,
    ) -> Size {
        bc.constrain((500.0, 500.0))
    }

    fn paint(&mut self, _ctx: &mut PaintCtx, _data: &Pomodoro, _env: &Env) {}
}

fn time_from_state(state: State) -> u32 {
    if state == State::Break {
        return BREAK_TIME;
    }

    WORK_TIME
}

fn change_state(data: &mut Pomodoro, state: State) {
    data.current_state = state;
    data.seconds = time_from_state(state);
}

fn blip() {
    let device = rodio::default_output_device().unwrap();

    let file = File::open("blip.wav").unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
    rodio::play_raw(&device, source.convert_samples());
}

fn ui_builder() -> impl Widget<Pomodoro> {
    let mut col = Flex::column();

    let timer = TimerWidget {
        timer_id: TimerToken::INVALID,
    };
    col.add_flex_child(Align::centered(timer), 1.0);

    let timer_label = Label::new(|data: &Pomodoro, _env: &_| {
        format!(
            "{minutes:02}:{seconds:02}",
            minutes = data.seconds / 60,
            seconds = data.seconds % 60
        )
    })
    .with_text_size(64.0);
    let state_label =
        Label::new(|data: &Pomodoro, _env: &_| format!("{:?}", data.current_state));

    col.add_flex_child(timer_label, 1.0);
    col.add_flex_spacer(1.0);
    col.add_flex_child(ProgressBar::new().lens(Pomodoro::progress).fix_width(150.0), 1.0);
    col.add_flex_spacer(1.0);

    col.add_flex_child(state_label, 1.0);
    col.add_flex_spacer(1.0);

    let start_button = Button::new("Start/Pause").on_click(|_ctx, data: &mut Pomodoro, _env| {
        if data.current_state == State::Paused {
            data.current_state = data.paused_state;
        } else {
            data.paused_state = data.current_state;
            data.current_state = State::Paused;
        }
    });
    col.add_flex_child(start_button, 1.0);

    col
}

pub fn main() {
    let window = WindowDesc::new(ui_builder)
        .window_size((500., 400.00))
        .with_min_size((300., 300.00))
        .title(LocalizedString::new("pomodoro-window-title").with_placeholder("Pomodoro"));

    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(Pomodoro {
            seconds: WORK_TIME,
            current_state: State::Paused,
            paused_state: State::Working,
            progress: 0.0,
        })
        .expect("launch failed");
}
