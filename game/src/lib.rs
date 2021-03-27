// scenes - these effectively act as separate games
pub mod awaiting_opponent;
pub mod pong; // pong game logic, ui, and rollback networking
pub mod title_screen; // title screen buttons and scene switching logic // screen that polls the server waiting for an opponent to join

// utility functions - these are more like libraries
pub mod imui;
pub mod scene; // scene API and scene struct/trait // immediate mode ui
