#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    SplashScreen,
    MainMenu,
    InGame,
    Paused,
}
