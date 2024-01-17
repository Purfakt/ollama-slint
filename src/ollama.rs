use super::{Author, MainWindow, Message};
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
        let (channel, r) = tokio::sync::mpsc::unbounded_channel();
        let worker_thread = std::thread::spawn({
            let handle_weak = ui.as_weak();
            move || {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(ollama_worker_loop(r, handle_weak))
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
    mut r: UnboundedReceiver<OllamaMessage>,
    handle: slint::Weak<MainWindow>,
) -> tokio::io::Result<()> {
    dotenv::dotenv().ok();
    let ollama_host =
        std::env::var("OLLAMA_HOST").expect("OLLAMA_HOST environment variable should be set");
    let ollama_port = std::env::var("OLLAMA_PORT")
        .expect("OLLAMA_PORT environment variable should be set")
        .parse::<u16>()
        .expect("OLLAMA_PORT should be a valid port number");
    let model = std::env::var("OLLAMA_MODEL").expect("MODEL environment variable should be set");

    let ollama = Ollama::new(ollama_host, ollama_port);
    let context: Option<GenerationContext> = None;

    loop {
        let message = match r.recv().await {
            None => return Ok(()),
            Some(m) => m,
        };

        match message {
            OllamaMessage::Quit => return Ok(()),
            OllamaMessage::Generate { text } => {
                let mut request = GenerationRequest::new(model.clone(), text.to_string());

                if let Some(context) = context.clone() {
                    request = request.context(context);
                };

                let generate = ollama.generate(request).await.unwrap();
                let response = generate.response.trim().to_owned();

                let weak = &handle.clone();
                let _ = weak.upgrade_in_event_loop(move |h| {
                    let mut messages: Vec<Message> = h.get_messages().iter().collect();
                    messages.push(Message {
                        text: SharedString::from(response),
                        author: Author::Ollama,
                    });
                    let message_model = std::rc::Rc::new(slint::VecModel::from(messages));
                    h.set_messages(message_model.into());
                });
            }
        }
    }
}
