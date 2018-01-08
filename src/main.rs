// The gtk crate provides GTK+ widgets used to draw the user interface
extern crate gtk;

// The relm crate provides the Relm functional async event resolution system
#[macro_use] extern crate relm;
// The relm_derive crate provides custom derives that save boilerplate for Relm message enums
#[macro_use] extern crate relm_derive;

// The rfyl crate provides dice notation functionality
extern crate rfyl;
// The futures crate provides functionality for creating asyncronous functions
extern crate futures;

// GUI imports
use gtk::*;
use relm::{Relm, Widget, Update};

// Logic imports
mod roll;
use roll::{lazy_roll, RollOutcome};

/// The model keeps track of all the state of the program
struct Model {
    /// The async event resolution system
    pub relm: Relm<Win>,
    /// The current content of the text entry box
    pub textentry_content: String,
    /// All the rolls the program has computed this session
    pub rolls: Vec<RollOutcome>,
}

/// All the actions available to the program
#[derive(Msg, Debug)]
enum Message {
    /// Fired every time the input is changed
    ChangeInput,
    /// Fired when a roll is triggered - either by the "activate" event
    /// or a click on the button
    StartRoll,
    /// Fired when the async future for rolling an expression completes
    FinishRoll(RollOutcome),
    /// Fired when the application is closed/quit
    Quit
}

/// Stores references to the retained state of the GUI.
/// Because of how GTK+ works, it looks like we own the memory,
/// but all the GTK+ widgets are actually owned by GTK+.
struct Win {
    /// The application state
    model: Model,
    /// The window containing the application's GUI
    window: Window,
    /// The input into which dice expressions can be entered
    input: Entry,
    /// Data for the treeview that reports the result of dice rolls
    rolls_store: ListStore,
}

/// The Update trait allows the Relm API to work with the app
impl Update for Win {
    type Model = Model;
    type ModelParam = ();
    type Msg = Message;

    // Create the inital model - the inital state of the application
    fn model(relm: &Relm<Self>, _: Self::ModelParam) -> Self::Model {
        Model {
            relm: relm.clone(),
            rolls: Vec::new(),
            textentry_content: String::new(),
        }
    }

    // Update the model when a message is received
    fn update(&mut self, event: Self::Msg) {
        // These are set to true if these parts of the UI need to be refreshed.
        let mut input_invalid = false;
        let mut output_invalid = false;

        match event {
            // When the Quit event fires, just end the program.
            Message::Quit => gtk::main_quit(),
            // When the ChangeInput event fires, record the new value.
            Message::ChangeInput => {
                self.model.textentry_content = self.input.get_text().unwrap().clone();
                input_invalid = true;
            }
            // When the StartRoll event fires, spin off a future to do the rolling.
            Message::StartRoll => {
                // Get the spec from the current model.
                let spec = self.model.textentry_content.clone();
                // Start a future for the roll computation.
                let future = lazy_roll(spec);
                // Tell Relm to fire a FinishRoll event when the future is finished
                self.model.relm.connect_exec_ignore_err(future, Message::FinishRoll);
                // Clear the text entry.
                self.model.textentry_content = String::new();
                input_invalid = true;
            },
            // When the FinishRoll event fires, record the result.
            Message::FinishRoll(outcome) => {
                self.model.rolls.push(outcome);
                output_invalid = true;
            }
        };

        // Set the input text to the recorded value from the model.
        if input_invalid {
            self.input.set_text(&self.model.textentry_content);
        }

        // Set the rolls store's content to that of the model's roll list.
        if output_invalid {
            self.rolls_store.clear();
            for roll in self.model.rolls.iter() {
                // Insert the new value at the beginning
                let i = self.rolls_store.prepend();
                self.rolls_store.set(&i, 
                    &[0,1],  // Insert into rows 0 and 1
                    &[&roll.descriptor, &format!("{}", roll.outcome)] // Insert the descriptor and the outcome
                    );
            }
        }
    }
}


impl Widget for Win {
    type Root = Window;

    // Return the root widget
    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    // Create view widgets
    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        // Set up the top level window
        let window = Window::new(WindowType::Toplevel);
        window.set_title("d20roll - Rust Dice Roller");
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(300, 400);

        // This box organizes the entry and the output in vertical order
        let vbox = Box::new(Orientation::Vertical, 0);

        // This box organizes the input entry and the Roll button
        let hbox = Box::new(Orientation::Horizontal, 0);
        // It needs to fill all the available space.
        hbox.set_hexpand(true);

        // This input accepts user text input
        let input = Entry::new();
        // It needs to push the button to the minimum possible size
        input.set_hexpand(true);
        hbox.add(&input);

        // This button submits the user input
        let button = Button::new_with_label("Roll");
        hbox.add(&button);

        vbox.add(&hbox);

        // This store holds all the rolls to be displayed on the UI
        let rolls_store = ListStore::new(&[Type::String, Type::String]);
        // This view displays the rolls so far
        let rolls_view = TreeView::new_with_model(&rolls_store);
        // The view needs to fill the whole UI
        rolls_view.set_hexpand(true);
        rolls_view.set_vexpand(true);
        // The headers need to be visible so it's clear what each column is
        rolls_view.set_headers_visible(true);

        // This column displays the rolls specifications
        let spec_column = TreeViewColumn::new();
        let cell = CellRendererText::new();
        spec_column.set_title("Specification");
        spec_column.set_visible(true);
        spec_column.pack_start(&cell, true);
        // Associate this column with column 0 of the model
        spec_column.add_attribute(&cell, "text", 0);
        rolls_view.append_column(&spec_column);

        // This column displays the rolls results
        let result_column = TreeViewColumn::new();
        let cell = CellRendererText::new();
        result_column.set_title("Result");
        result_column.set_visible(true);
        result_column.pack_start(&cell, true);
        // Associate this column with column 0 of the model
        result_column.add_attribute(&cell, "text", 1);
        rolls_view.append_column(&result_column);

        // This wrapper enables scrolling of the list
        let label_container_scroll = ScrolledWindow::new(None, None);
        label_container_scroll.set_hexpand(true);
        label_container_scroll.set_vexpand(true);

        label_container_scroll.add(&rolls_view);
        vbox.add(&label_container_scroll);

        window.add(&vbox);

        window.show_all();

        // The delete event should quit the app
        connect!(relm, window, connect_delete_event(_, _), return (Some(Message::Quit), Inhibit(false)));
        // Whenever the input is changed, the model needs to be updated
        connect!(relm, input, connect_changed(_), Message::ChangeInput);
        // Whenever the Roll button is clicked, a roll needs to start
        connect!(relm, button, connect_clicked(_), Message::StartRoll);
        // Whenever the user hits "enter" or submits the input in another way, a roll needs to start
        connect!(relm, input, connect_activate(_), Message::StartRoll);

        Win {
            model,
            window,
            input,
            rolls_store
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}
