
use std::sync::Arc;
use std::collections::HashMap;

use super::gtk;
use super::gtk::prelude::*;
use super::gtk::{Button, Window, WindowType, Orientation, Label};

use crossbeam::sync::MsQueue;

use renderer::{self, Command, Event};

#[derive(Clone)]
pub struct Renderer {
    pub commands: Arc<MsQueue<Command>>,
    pub events: Arc<MsQueue<Event>>,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            commands: Arc::new(MsQueue::new()),
            events: Arc::new(MsQueue::new()),
        }
    }
}

impl renderer::Renderer for Renderer {
    fn send(&self, cmd: Command) {
        self.commands.push(cmd)
    }

    fn recv(&self) -> Event {
        self.events.pop()
    }
}

enum Widget {
    Button(Button),
    Label(Label),
    Field(gtk::Entry),
}

pub fn run(renderer: Renderer) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_title("** cedar **");
    window.set_default_size(500, 500);

    window.connect_delete_event(|_, _| {
        // println!("Quit!");

        gtk::main_quit();
        Inhibit(false)
    });

    let mut widgets = HashMap::new();

    let container = gtk::Box::new(Orientation::Vertical, 5);
    window.add(&container);

    gtk::timeout_add(16, move || {
        if let Some(command) = renderer.commands.try_pop() {
            // println!("Command: {:?}", command);

            match command {
                Command::Create {
                    id,
                    kind,
                    attributes,
                } => {
                    match kind.as_str() {
                        "Button" => {
                            let button = Button::new_with_label(&attributes["Text"]);
                            container.add(&button);

                            {
                                let id = id.clone();
                                let events = renderer.events.clone();
                                button.connect_clicked(
                                    move |_| events.push(Event::Click { id: id.clone() }),
                                );
                            }

                            widgets.insert(id, Widget::Button(button));
                        }

                        "Label" => {
                            let label = Label::new(Some(attributes["Text"].as_str()));
                            container.add(&label);

                            widgets.insert(id, Widget::Label(label));
                        }

                        "Field" => {
                            let field = gtk::Entry::new();
                            container.add(&field);

                            if let Some(ref placeholder) = attributes.get("Placeholder") {
                                field.set_placeholder_text(Some(placeholder.as_str()))
                            }

                            {
                                let id = id.clone();
                                let events = renderer.events.clone();
                                field.connect_event(move |field, _| {
                                    if let Some(ref text) = field.get_text() {
                                        events.push(Event::Change {
                                            id: id.clone(),
                                            value: text.clone(),
                                        });
                                    }

                                    gtk::Inhibit(false)
                                });
                            }

                            widgets.insert(id, Widget::Field(field));
                        }

                        _ => unimplemented!(),
                    }
                }

                Command::Update(id, attribute, value) => {
                    let ref widget = widgets[&id];
                    match widget {
                        &Widget::Label(ref label) if attribute == "Text" => label.set_label(&value),

                        &Widget::Field(ref field) if attribute == "Placeholder" => {
                            field.set_placeholder_text(Some(value.as_str()))
                        }

                        _ => unimplemented!(),
                    }
                }

                _ => unimplemented!(),
            }

            window.show_all();
        }

        gtk::Continue(true)
    });

    gtk::main();
}
