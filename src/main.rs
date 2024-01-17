use std::sync::Arc;

use ollama_rs::{
    generation::completion::{request::GenerationRequest, GenerationContext},
    Ollama,
};
use slint::{Model, SharedString};
use tokio::runtime::Runtime;

slint::include_modules!();

fn main() {
    dotenv::dotenv().ok();
    let main_window = MainWindow::new().unwrap();
    let ollama_host =
        std::env::var("OLLAMA_HOST").expect("OLLAMA_HOST environment variable should be set");
    let ollama_port = std::env::var("OLLAMA_PORT")
        .expect("OLLAMA_PORT environment variable should be set")
        .parse::<u16>()
        .expect("OLLAMA_PORT should be a valid port number");
    let model = std::env::var("OLLAMA_MODEL").expect("MODEL environment variable should be set");

    let main_window_weak = std::rc::Rc::new(main_window.as_weak().unwrap());

    let ollama = Ollama::new(ollama_host, ollama_port);
    let context: Option<GenerationContext> = None;
    let rt = Runtime::new().unwrap();

    let model_arc = Arc::new(model);

    let model_clone = Arc::clone(&model_arc);

    main_window_weak
        .clone()
        .on_user_input_accepted(move |text| {
            let mut messages: Vec<Message> = main_window_weak.get_messages().iter().collect();

            messages.push(Message {
                text: text.clone(),
                author: Author::User,
            });

            let message_model = std::rc::Rc::new(slint::VecModel::from(messages.clone()));
            main_window_weak.set_messages(message_model.into());

            let mut request =
                GenerationRequest::new(model_clone.clone().to_string(), text.to_string());

            if let Some(context) = context.clone() {
                request = request.context(context);
            }

            let res = rt.block_on(ollama.generate(request));

            if let Ok(res) = res {
                messages.push(Message {
                    text: SharedString::from(res.response.trim()),
                    author: Author::Ollama,
                });
                let message_model = std::rc::Rc::new(slint::VecModel::from(messages));
                main_window_weak.set_messages(message_model.into());
            }
        });

    main_window.run().unwrap();
}
