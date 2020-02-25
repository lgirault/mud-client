#[derive(PartialEq, Copy, Clone)]
pub enum AppArea {
    Main,
    Input,
    CharacterSheet,
    Map,
    Chat,
}

//impl ToString for AppArea {
//    fn to_string(&self) -> String {
//        match self {
//            AppArea::Main => String::from(AppArea::MAIN),
//            AppArea::Input => String::from(AppArea::INPUT)
//        }
//    }
//
//}

//impl fmt::Display for AppArea {
//    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//        write!(f, "{:?}", self)
//        // or, alternatively:
//        // fmt::Debug::fmt(self, f)
//    }
//}

impl AppArea {
    pub const MAIN: &'static str = "Main";
    pub const INPUT: &'static str = "Input";
    pub const CHARACTER_SHEET: &'static str = "Character Sheet";
    pub const MAP: &'static str = "Map";
    pub const CHAT: &'static str = "Chat";

    pub fn name(&self) -> &'static str {
        match self {
            AppArea::Main => AppArea::MAIN,
            AppArea::Input => AppArea::INPUT,
            AppArea::CharacterSheet => AppArea::CHARACTER_SHEET,
            AppArea::Map => AppArea::MAP,
            AppArea::Chat => AppArea::CHAT,
        }
    }
}

pub struct App {
    pub focused_area: AppArea,
    /// Current value of the input box
    pub input: String,
    /// History of recorded messages
    pub messages: Vec<String>,
}

impl App {
    pub fn new() -> App {
        App {
            focused_area: AppArea::Input,
            input: String::new(),
            messages: Vec::new(),
        }
    }
}
