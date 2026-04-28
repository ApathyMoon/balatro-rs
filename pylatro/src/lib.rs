use balatro_rs::action::Action;
use balatro_rs::card::Card;
use balatro_rs::config::Config;
use balatro_rs::error::GameError;
use balatro_rs::game::Game;
use balatro_rs::joker::Jokers;
use balatro_rs::stage::{End, Stage};
use pyo3::prelude::*;
use serde_json;

#[pyclass(module = "pylatro")]
struct GameEngine {
    game: Game,
}

#[pymethods]
impl GameEngine {
    pub fn __getstate__(&self) -> pyo3::PyResult<Vec<u8>> {
        serde_json::to_vec(&self.game)
            .map_err(|e| pyo3::exceptions::PyException::new_err(e.to_string()))
    }

    pub fn __setstate__(&mut self, state: Vec<u8>) -> pyo3::PyResult<()> {
        self.game = serde_json::from_slice(&state)
            .map_err(|e| pyo3::exceptions::PyException::new_err(e.to_string()))?;
        self.game.rehydrate_effects(); // rebuild Arc<Mutex<fn>> closures serde skipped
        Ok(())
    }


    #[new]
    #[pyo3(signature = (config=None))]
    fn new(config: Option<Config>) -> Self {
        GameEngine {
            game: Game::new(config.unwrap_or(Config::default())),
        }
    }

    fn clone_game(&self) -> Self {
        let mut cloned = GameEngine {
            game: self.game.clone(),
        };
        cloned.game.rehydrate_effects();
        cloned
    }

    pub fn clone(&self) -> Self {
        self.clone_game()
    }

    pub fn copy(&self) -> Self {
        self.clone_game()
    }

    // Also fix __copy__ and __deepcopy__ so Python's copy.deepcopy works too
    pub fn __copy__(&self) -> Self {
        self.clone()
    }

    pub fn __deepcopy__(&self, _memo: pyo3::Bound<'_, pyo3::types::PyDict>) -> Self {
        self.clone_game()
    }

    fn gen_actions(&self) -> Vec<Action> {
        return self.game.gen_actions().collect();
    }

    fn gen_action_space(&self) -> Vec<usize> {
        return self.game.gen_action_space().to_vec();
    }

    fn handle_action(&mut self, action: Action) -> Result<(), GameError> {
        return self.game.handle_action(action);
    }

    fn handle_action_index(&mut self, index: usize) -> Result<(), GameError> {
        return self.game.handle_action_index(index);
    }

    fn index_to_action(&self, index: usize) -> PyResult<Action> {
        let space = self.game.gen_action_space();
        space.to_action(index, &self.game)
            .map_err(|e| pyo3::exceptions::PyException::new_err(format!("{:?}", e)))
    }

    #[getter]
    fn state(&self) -> GameState {
        return GameState {
            game: self.game.clone(),
        };
    }
    #[getter]
    fn is_over(&self) -> bool {
        return self.game.is_over();
    }
    #[getter]
    fn is_win(&self) -> bool {
        if let Some(end) = self.game.result() {
            if end == End::Win {
                return true;
            }
        }
        return false;
    }
}

#[pyclass]
struct GameState {
    game: Game,
}

#[pymethods]
impl GameState {
    #[getter]
    fn stage(&self) -> Stage {
        return self.game.stage;
    }
    #[getter]
    fn round(&self) -> usize {
        return self.game.round;
    }
    #[getter]
    fn action_history(&self) -> Vec<Action> {
        return self.game.action_history.clone();
    }
    #[getter]
    fn deck(&self) -> Vec<Card> {
        return self.game.deck.cards();
    }
    #[getter]
    fn selected(&self) -> Vec<Card> {
        return self.game.available.selected();
    }
    #[getter]
    fn available(&self) -> Vec<Card> {
        return self.game.available.cards();
    }
    #[getter]
    fn discarded(&self) -> Vec<Card> {
        return self.game.discarded.clone();
    }
    #[getter]
    fn plays(&self) -> usize {
        return self.game.plays;
    }
    #[getter]
    fn discards(&self) -> usize {
        return self.game.discards;
    }

    #[getter]
    fn score(&self) -> usize {
        return self.game.score;
    }
    #[getter]
    fn required_score(&self) -> usize {
        return self.game.required_score();
    }
    #[getter]
    fn jokers(&self) -> Vec<Jokers> {
        return self.game.jokers.clone();
    }
    #[getter]
    fn money(&self) -> usize {
        return self.game.money;
    }

    fn __repr__(&self) -> String {
        format!("GameState:\n{}", self.game)
    }
}

#[pymodule]
fn pylatro(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Config>()?;
    m.add_class::<GameEngine>()?;
    m.add_class::<GameState>()?;
    m.add_class::<Stage>()?;
    Ok(())
}
