use super::{config, Author, MainWindow, Message};
use ollama_rs::{
    generation::completion::{request::GenerationRequest, GenerationContext},
    Ollama,
};
use slint::ComponentHandle;
use slint::{Model, SharedString};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Debug)]
pub enum OllamaMessage {
    Quit,
    Generate { text: String },
}

pub struct OllamaWorker {
    pub channel: UnboundedSender<OllamaMessage>,
    worker_thread: std::thread::JoinHandle<()>,
}

impl OllamaWorker {
    pub fn new(ui: &MainWindow) -> Self {
        let (channel, receiver) = tokio::sync::mpsc::unbounded_channel();
        let worker_thread = std::thread::spawn({
            let handle = ui.as_weak();
            move || {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(ollama_worker_loop(receiver, handle))
                    .unwrap()
            }
        });
        Self {
            channel,
            worker_thread,
        }
    }

    pub fn join(self) -> std::thread::Result<()> {
        let _ = self.channel.send(OllamaMessage::Quit);
        self.worker_thread.join()
    }
}

async fn ollama_worker_loop(
    mut receiver: UnboundedReceiver<OllamaMessage>,
    handle: slint::Weak<MainWindow>,
) -> tokio::io::Result<()> {
    let config = config::config();
    let ollama = Ollama::new(config.ollama_host().to_owned(), config.ollama_port());
    let mut context: Option<GenerationContext> = None;

    loop {
        let message = match receiver.recv().await {
            None => return Ok(()),
            Some(m) => m,
        };

        match message {
            OllamaMessage::Quit => return Ok(()),
            OllamaMessage::Generate { text } => {
                let mut request =
                    GenerationRequest::new(config.ollama_model().to_owned(), text.to_string());

                if let Some(context) = context.clone() {
                    request = request.context(context);
                };

                let response = ollama.generate(request).await.unwrap();
                context = response.final_data.and_then(|x| Some(x.context));
                let response_text = response.response;

                let weak = &handle.clone();
                let _ = weak.upgrade_in_event_loop(move |h| {
                    push_message(h, response_text.trim().to_owned())
                });
            }
        }
    }
}

fn push_message(window: MainWindow, response: String) {
    let mut messages: Vec<Message> = window.get_messages().iter().collect();
    messages.push(Message {
        text: SharedString::from(response),
        author: Author::Ollama,
    });
    let message_model = std::rc::Rc::new(slint::VecModel::from(messages));
    window.set_messages(message_model.into());
}
