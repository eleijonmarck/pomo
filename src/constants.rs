use std::collections::HashMap;

// How to use the Pomodoro Timer?
// Set estimate pomodoros (1 = 25min of work) for each tasks
// Select a task to work on
// Start timer and focus on the task for 25 minutes
// Take a break for 5 minutes when the alarm ring
// Iterate 3-5 until you finish the tasks",
pub const DESCRIPTION: [&'static str; 6] = [
    "",
    "How to use the Pomodoro Timer?",
    "1 - Focus on a task for 25 minutes",
    "2 - Take a break for 5 minutes when the alarm rings",
    "3 - Iterate 3-5 times until you finish the task",
    "",
];

pub const PAUSE_MSG: &'static str = "-- Paused --";

lazy_static! {
    pub static ref GLYPH_DEFINITIONS: HashMap<char, [&'static str; 6]> = {
        let mut glyphs: HashMap<char, [&'static str; 6]> = HashMap::new();
        #[rustfmt::skip]
        glyphs.insert(
            ':',
            [
                "   ",
                "██╗",
                "╚═╝",
                "██╗",
                "╚═╝",
                "   ",
            ],
        );
        glyphs.insert(
            '0',
            [
                " ██████╗ ",
                "██╔═████╗",
                "██║██╔██║",
                "████╔╝██║",
                "╚██████╔╝",
                " ╚═════╝ ",
            ],
        );
        #[rustfmt::skip]
        glyphs.insert(
            '1',
            [
                " ██╗",
                "███║",
                "╚██║",
                " ██║",
                " ██║",
                " ╚═╝",
            ],
        );
        glyphs.insert(
            '2',
            [
                "██████╗ ",
                "╚════██╗",
                " █████╔╝",
                "██╔═══╝ ",
                "███████╗",
                "╚══════╝",
            ],
        );
        glyphs.insert(
            '3',
            [
                "██████╗ ",
                "╚════██╗",
                " █████╔╝",
                " ╚═══██╗",
                "██████╔╝",
                "╚═════╝ ",
            ],
        );
        glyphs.insert(
            '4',
            [
                "██╗  ██╗",
                "██║  ██║",
                "███████║",
                "╚════██║",
                "     ██║",
                "     ╚═╝",
            ],
        );
        glyphs.insert(
            '5',
            [
                "███████╗",
                "██╔════╝",
                "███████╗",
                "╚════██║",
                "███████║",
                "╚══════╝",
            ],
        );
        glyphs.insert(
            '6',
            [
                " ██████╗ ",
                "██╔════╝ ",
                "███████╗ ",
                "██╔═══██╗",
                "╚██████╔╝",
                " ╚═════╝ ",
            ],
        );
        glyphs.insert(
            '7',
            [
                "███████╗",
                "╚════██║",
                "    ██╔╝",
                "   ██╔╝ ",
                "   ██║  ",
                "   ╚═╝  ",
            ],
        );
        glyphs.insert(
            '8',
            [
                " █████╗ ",
                "██╔══██╗",
                "╚█████╔╝",
                "██╔══██╗",
                "╚█████╔╝",
                " ╚════╝ ",
            ],
        );
        glyphs.insert(
            '9',
            [
                " █████╗ ",
                "██╔══██╗",
                "╚██████║",
                " ╚═══██║",
                " █████╔╝",
                " ╚════╝ ",
            ],
        );
        glyphs
    };
}
