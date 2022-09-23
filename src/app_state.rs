#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    SplashScreen,
    TitleScreen,
    MainGame,
    Paused,
}
