use gpui_kit::component::button::*;
use gpui_kit::*;
use mvvm_core::{Command, State};

struct TodoViewModel {
    items: State<Vec<String>>,
    add_sample: Command<()>,
}

impl TodoViewModel {
    fn new() -> Self {
        let items = State::new(vec!["Read the MVVM overview".to_owned()]);
        let items_for_command = items.clone();
        let add_sample = Command::always(move |_| {
            items_for_command.update(|items| items.push("Explore the GPUI adapter".to_owned()));
            Ok(())
        });
        Self { items, add_sample }
    }
}

struct TodoApp {
    vm: TodoViewModel,
}

impl Render for TodoApp {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let items = self.vm.items.get();
        let count = items.len();
        div()
            .flex()
            .flex_col()
            .size_full()
            .p_6()
            .gap_3()
            .child(div().text_2xl().child("Rust MVVM · Todo"))
            .child(format!("{count} item(s)"))
            .children(items.into_iter().map(|item| div().child(format!("• {item}"))))
            .child(
                Button::new("add-todo")
                    .primary()
                    .label("Add sample item")
                    .on_click(cx.listener(|view, _, _, cx| {
                        let _ = view.vm.add_sample.execute(());
                        cx.notify();
                    })),
            )
    }
}

fn main() {
    application().run(|cx| {
        init(cx);
        open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| TodoApp { vm: TodoViewModel::new() })
        })
        .expect("failed to open todo window");
    });
}
