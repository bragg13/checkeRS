#[derive(Debug)]
pub enum Scene {
    Menu(MainMenuScene),
    Game(GameScene),
}

impl Scene {
    pub fn handle_input(&mut self, key_event: KeyEvent) -> Option<ClientEvent> {
        match self {
            Scene::Menu(menu) => menu.handle_input(key_event),
            Scene::Game(game_scene) => game_scene.handle_input(key_event),
        }
    }
    pub fn handle_event(&mut self, game_event: GameEvent) -> Option<ClientEvent> {
        match self {
            Scene::Menu(menu) => menu.handle_server_events(game_event),
            Scene::Game(game_scene) => game_scene.handle_server_events(game_event),
        }
    }
    pub fn handle_render(&self, area: Rect, buf: &mut Buffer) {
        match self {
            Scene::Menu(main_menu_scene) => main_menu_scene.render(area, buf),
            Scene::Game(game_scene) => game_scene.render(area, buf),
        }
    }
}
