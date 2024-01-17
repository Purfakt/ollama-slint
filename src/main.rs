mod generated_code {
    slint::include_modules!();
}
pub use generated_code::*;
mod ollama;

use ollama::OllamaMessage;
use slint::{Model, SharedString};

fn main() {
    let main_window = MainWindow::new().unwrap();
    let ollama_worker = ollama::OllamaWorker::new(&main_window);

    let main_window_weak = main_window.as_weak();

    main_window_weak.unwrap().on_user_input_accepted({
        let ollama_channel = ollama_worker.channel.clone();
        move |text| {
            handle_user_input(&main_window_weak, text.clone());
            ollama_channel
                .send(OllamaMessage::Generate {
                    text: text.to_string(),
                })
                .unwrap();
        }
    });

    main_window.run().unwrap();
    ollama_worker.join().unwrap();
}

fn handle_user_input(handle: &slint::Weak<MainWindow>, text: SharedString) {
    let h = &handle.clone().unwrap();
    let mut messages: Vec<Message> = h.get_messages().iter().collect();

    messages.push(Message {
        text: text.clone(),
        author: Author::User,
    });

    let message_model = std::rc::Rc::new(slint::VecModel::from(messages.clone()));
    h.set_messages(message_model.into());
}
