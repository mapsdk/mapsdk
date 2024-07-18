use tokio::sync::mpsc;

use crate::{
    env::Env,
    layer::{Event, Layer, LayerType},
    render::Renderer,
};

pub struct ImageTiledLayer {
    event_sender: Option<mpsc::UnboundedSender<Event>>,
}

impl Layer for ImageTiledLayer {
    fn r#type(&self) -> LayerType {
        LayerType::ImageTiledLayer
    }

    fn set_event_sender(&mut self, event_sender: mpsc::UnboundedSender<Event>) {
        self.event_sender = Some(event_sender);
    }

    fn unset_event_sender(&mut self) {
        self.event_sender = None;
    }

    fn update(&mut self, _env: &Env, _renderer: &mut Renderer) {
        todo!()
    }
}
