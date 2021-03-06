use amethyst::{
    ecs::{System, WriteStorage, ReadExpect},
    ui::{UiText},
};
use crate::states::load::UiElements;
use crate::systems::actions::actions::HasRunNow;
use amethyst::core::ecs::RunNow;


pub struct UpdateUi {
    pub(crate) text: &'static str,
}

impl HasRunNow for UpdateUi {
    fn get_run_now<'a>(&self) -> Box<dyn RunNow<'a>> {
        Box::new(UpdateUi {text : self.text})
    }
}

impl<'a> System<'a> for UpdateUi {
    type SystemData = (
        WriteStorage<'a, UiText>,
        ReadExpect<'a, UiElements>,
    );

    fn run(&mut self, (mut ui_text, ui_elements): Self::SystemData){
        if let Some(text) = ui_text.get_mut(ui_elements.current_state_text) {
            text.text = self.text.parse().unwrap();
        }
    }
}