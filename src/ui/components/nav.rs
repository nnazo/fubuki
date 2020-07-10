use crate::ui::style;
// use crate::ui::style;
// use crate::ui::components::*;
use iced::{
    Element, Row, button, Button, Text, Length
};

#[derive(Debug, Clone)]
pub enum Message {
    // AnimeListPress,
    // MangaListPress,
    CurrentMediaPress { selected: bool },
    SettingsPress { selected: bool },
}

#[derive(Debug, Default, Clone)]
pub struct Nav {
    // anime_state: button::State,
    // manga_state: button::State,
    media_state: button::State,
    settings_state: button::State,
    media_selected: bool,
    settings_selected: bool,
    // content: Page,
}

impl Nav { // seems like this will need to be both the nav and the content pane as i cant bubble messages out of the update function when a button is pressed --- wait im not sure... these guys were wrapping those messages a lot
    pub fn new() -> Self {
        Nav {
            media_selected: true,
            ..Nav::default()
        }
    }

    pub fn update(&mut self, message: Message) {
        // i dont think the nav will really ever need to update aside from
        // changing style of the selected button -- or maybe even not? dunno since the framework should do that for me
        match message {
            Message::CurrentMediaPress { selected: _ } => {
                self.media_selected = true;
                self.settings_selected = false;
            },
            Message::SettingsPress { selected: _ } => {
                self.media_selected = false;
                self.settings_selected = true;
            },
        }
        // if i have nav hold the container then i could do something like container.update(container::message::something)
    }

    pub fn view(&mut self) -> Element<Message> {
        let media = Button::new(&mut self.media_state, Text::new("Current Media"))
            // .width(40)
            // .height(Length::Units(25))
            .padding(18)
            .width(Length::Fill)
            .style(style::Button::Nav { selected: self.media_selected })
            .on_press(Message::CurrentMediaPress { selected: self.media_selected });
        
        let settings = Button::new(&mut self.settings_state, Text::new("Settings"))
            // .height(Length::Units(25))
            .padding(18 )
            .width(Length::Fill)
            .style(style::Button::Nav { selected: self.settings_selected })
            .on_press(Message::SettingsPress { selected: self.settings_selected });
        
        Row::new()
            .push(media)
            .push(settings)
            .spacing(0)
            .into()
    }
}

