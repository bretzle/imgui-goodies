use imgoodies::{notify::*, Framework};
use imgui::{Condition, TreeNodeFlags, Ui};

struct State {
    notifications: Notifications,
    title: String,
    content: String,
    duration: i32,
    typ: ToastType,
    enable_title: bool,
    enable_content: bool,
}

fn main() {
    Framework::new(
        "Notification demo",
        State {
            notifications: Notifications::default(),
            title: "A wonderful quote!".to_string(),
            content: String::new(),
            duration: 5000,
            typ: ToastType::Success,
            enable_title: true,
            enable_content: true,
        },
    )
    .run(draw);
}

fn draw(ui: &mut Ui, state: &mut State) {
    ui.window("Hello World!")
        .size([550.0, 550.0], Condition::FirstUseEver)
        .position([50.0, 50.0], Condition::FirstUseEver)
        .build(|| {
            if ui.collapsing_header("Examples without title", TreeNodeFlags::DEFAULT_OPEN) {
                if ui.button("Success") {
                    state.notifications.push(Toast::new(ToastType::Success, 3000).content("Hello World! This is a success!"));
                }
                ui.same_line();
                if ui.button("Warning") {
					state.notifications.push(Toast::new(ToastType::Warning, 3000).content("Hello World! This is a warning!"));
				}
                ui.same_line();
                if ui.button("Error") {
					state.notifications.push(Toast::new(ToastType::Error, 3000).content("Hello World! This is a error!"));
				}
                ui.same_line();
                if ui.button("Info") {
					state.notifications.push(Toast::new(ToastType::Info, 3000).content("Hello World! This is a info!"));
				}
                ui.same_line();
                if ui.button("Info long") {
					state.notifications.push(Toast::new(ToastType::Info, 3000).content("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation"));
				}
            }

            if ui.collapsing_header("Do it yourself", TreeNodeFlags::DEFAULT_OPEN) {
                ui.input_text_multiline("Title", &mut state.title, [0.0, 0.0]).build();
                ui.input_text_multiline("Content", &mut state.content, [0.0, 0.0])
                    .build();
                ui.input_int("Duration (ms)", &mut state.duration).step(100).build();

                // dont allow negative
                if state.duration < 0 {
                    state.duration = 0;
                }

                let typ_str = ["None", "Success", "Warning", "Error", "Info"];

                ui.combo_simple_string(
                    "Type",
                    unsafe { &mut *(&mut state.typ as *mut _ as *mut usize) },
                    &typ_str,
                );

                ui.checkbox("Enable title", &mut state.enable_title);
                ui.same_line();
                ui.checkbox("Enable content", &mut state.enable_content);

                if ui.button("Show") {
                    let mut toast = Toast::new(state.typ, state.duration as usize);

                    if state.enable_title {
                        toast = toast.title(state.title.clone());
                    }

                    if state.enable_content {
                        toast = toast.content(state.content.clone());
                    }

                    state.notifications.push(toast);
                }
            }
        });

    state.notifications.render(ui);
}
