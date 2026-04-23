use gloo_timers::callback::Timeout;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use wasm_bindgen::prelude::*;
use web_sys::window;
use yew::prelude::*;

const TOKENS: &[&str] = &["1C", "55", "BD", "E9", "7A"];
const GRID_SIZE: usize = 5;
const MAX_BUFFER: usize = 7;

// --- 1. ЛОКАЛИЗАЦИЯ (i18n) ---
pub struct Dictionary {
    pub title: &'static str,
    pub buffer: &'static str,
    pub act_horiz: &'static str,
    pub act_vert: &'static str,
    pub demons: &'static str,
    pub level: &'static str,
    pub loaded: &'static str,
    pub waiting: &'static str,
    pub reward: &'static str,
    pub time_left: &'static str,
    pub success: &'static str,
    pub fail: &'static str,
}

fn get_dict(lang: &str) -> Dictionary {
    match lang {
        "en" => Dictionary {
            title: "Breach Protocol",
            buffer: "BUFFER:",
            act_horiz: "Activation: Horizontal",
            act_vert: "Activation: Vertical",
            demons: "Available Daemons",
            level: "Level",
            loaded: "UPLOADED",
            waiting: "STANDBY",
            reward: "Reward",
            time_left: "TIME LEFT:",
            success: "Breach successful. Disconnecting...",
            fail: "FAILURE: Time limit exceeded...",
        },
        _ => Dictionary {
            // Fallback на русский
            title: "Взлом протокола",
            buffer: "БУФЕР ОБМЕНА:",
            act_horiz: "Активация: Горизонталь",
            act_vert: "Активация: Вертикаль",
            demons: "Доступные Демоны",
            level: "Уровень",
            loaded: "ЗАГРУЖЕНО",
            waiting: "ОЖИДАНИЕ",
            reward: "Награда",
            time_left: "ОСТАЛОСЬ ВРЕМЕНИ:",
            success: "Взлом завершен. Разрыв соединения...",
            fail: "СБОЙ: Время истекло...",
        },
    }
}

// --- 2. КОНФИГУРАЦИЯ И ЦВЕТА ---
#[derive(Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ThemeConfig {
    pub primary: Option<String>,
    pub secondary: Option<String>,
    pub background: Option<String>,
    pub foreground: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct GameConfig {
    pub redirect_url: String,
    pub base_value: u32,
    pub time_limit: u32,
    pub locale: Option<String>,
    pub theme: Option<ThemeConfig>,
}

#[derive(Properties, PartialEq, Clone)]
pub struct GameProps {
    pub config: GameConfig,
}

#[derive(Serialize)]
struct GameResult {
    completed_targets: Vec<usize>,
    buffer: Vec<String>,
    total_coins: u32,
}

// --- 3. ИГРОВАЯ ЛОГИКА ---
fn generate_solvable_board() -> (Vec<Vec<String>>, Vec<Vec<String>>) {
    let mut rng = rand::thread_rng();
    let mut matrix = vec![vec![String::new(); GRID_SIZE]; GRID_SIZE];

    for r in 0..GRID_SIZE {
        for c in 0..GRID_SIZE {
            matrix[r][c] = TOKENS.choose(&mut rng).unwrap().to_string();
        }
    }

    let mut path = Vec::new();
    let mut used = HashSet::new();
    let mut is_row = true;
    let mut active_idx = 0;

    for _ in 0..MAX_BUFFER {
        let mut candidates = Vec::new();
        for i in 0..GRID_SIZE {
            let (r, c) = if is_row {
                (active_idx, i)
            } else {
                (i, active_idx)
            };
            if !used.contains(&(r, c)) {
                candidates.push((r, c));
            }
        }

        if candidates.is_empty() {
            break;
        }

        let &(r, c) = candidates.choose(&mut rng).unwrap();
        used.insert((r, c));
        path.push(matrix[r][c].clone());

        is_row = !is_row;
        active_idx = if is_row { r } else { c };
    }

    let t1 = if path.len() >= 2 {
        path[0..2].to_vec()
    } else {
        vec!["55".to_string(), "1C".to_string()]
    };
    let t2 = if path.len() >= 5 {
        path[2..5].to_vec()
    } else {
        vec!["BD".to_string(), "E9".to_string(), "1C".to_string()]
    };
    let t3 = if path.len() >= 7 {
        path[3..7].to_vec()
    } else {
        vec![
            "7A".to_string(),
            "BD".to_string(),
            "55".to_string(),
            "1C".to_string(),
        ]
    };

    (matrix, vec![t1, t2, t3])
}

pub fn calculate_coins(base_value: u32, completed_targets: &[usize]) -> u32 {
    completed_targets
        .iter()
        .map(|&idx| base_value * (idx as u32 + 1))
        .sum()
}

#[function_component(CyberHackGame)]
pub fn cyber_hack_game(props: &GameProps) -> Html {
    let board_state = use_state(|| generate_solvable_board());
    let matrix = &board_state.0;
    let targets = &board_state.1;

    let buffer = use_state(|| Vec::<String>::new());
    let is_row_turn = use_state(|| true);
    let active_index = use_state(|| 0usize);
    let used_cells = use_state(|| HashSet::<(usize, usize)>::new());
    let game_over = use_state(|| false);
    let completed_targets = use_state(|| HashSet::<usize>::new());

    // Инициализируем таймер значением из конфига (React props)
    let time_left = use_state(|| props.config.time_limit);
    let is_timer_running = use_state(|| false);

    let lang = props.config.locale.as_deref().unwrap_or("ru");
    let dict = get_dict(lang);

    let mut custom_style = String::new();
    if let Some(ref theme) = props.config.theme {
        if let Some(ref c) = theme.primary {
            custom_style.push_str(&format!("--primary: {}; ", c));
        }
        if let Some(ref c) = theme.secondary {
            custom_style.push_str(&format!("--secondary: {}; ", c));
        }
        if let Some(ref c) = theme.background {
            custom_style.push_str(&format!("--background: {}; ", c));
        }
        if let Some(ref c) = theme.foreground {
            custom_style.push_str(&format!("--foreground: {}; ", c));
        }
    }

    let end_game = {
        let game_over = game_over.clone();
        let redirect_url = props.config.redirect_url.clone();
        let base_value = props.config.base_value;

        Callback::from(
            move |(final_buffer, final_completed): (Vec<String>, HashSet<usize>)| {
                if *game_over {
                    return;
                }
                game_over.set(true);

                let completed_vec: Vec<usize> = final_completed.into_iter().collect();
                let total_coins = calculate_coins(base_value, &completed_vec);

                let result = GameResult {
                    completed_targets: completed_vec,
                    buffer: final_buffer,
                    total_coins,
                };

                if let Some(win) = window() {
                    if let Ok(json_str) = serde_json::to_string(&result) {
                        let _ = win.post_message(&JsValue::from_str(&json_str), "*");
                    }

                    let url = format!("{}?coins={}", redirect_url, total_coins);
                    Timeout::new(2000, move || {
                        let _ = win.location().assign(&url);
                    })
                    .forget();
                }
            },
        )
    };

    // Эффект обратного отсчета таймера
    {
        let time_left = time_left.clone();
        let is_timer_running = is_timer_running.clone();
        let game_over = game_over.clone();
        let end_game = end_game.clone();
        let buffer = buffer.clone();
        let completed_targets = completed_targets.clone();

        use_effect_with((*is_timer_running, *game_over, *time_left), move |deps| {
            let (running, over, current_time) = *deps;

            if running && !over && current_time > 0 {
                let time_left = time_left.clone();
                let timeout = Timeout::new(1000, move || {
                    time_left.set(current_time - 1);
                });
                Box::new(move || drop(timeout)) as Box<dyn FnOnce()>
            } else if running && !over && current_time == 0 {
                end_game.emit(((*buffer).clone(), (*completed_targets).clone()));
                Box::new(|| {}) as Box<dyn FnOnce()>
            } else {
                Box::new(|| {}) as Box<dyn FnOnce()>
            }
        });
    }

    let on_cell_click = {
        let buffer = buffer.clone();
        let is_row_turn = is_row_turn.clone();
        let active_index = active_index.clone();
        let used_cells = used_cells.clone();
        let game_over = game_over.clone();
        let completed_targets = completed_targets.clone();
        let is_timer_running = is_timer_running.clone();
        let end_game = end_game.clone();
        let matrix = matrix.clone();
        let targets = targets.clone();

        Callback::from(move |(r, c): (usize, usize)| {
            if *game_over {
                return;
            }
            // Запускаем таймер при первом клике
            if !*is_timer_running {
                is_timer_running.set(true);
            }

            let mut current_used = (*used_cells).clone();
            let mut current_buffer = (*buffer).clone();
            let mut current_completed = (*completed_targets).clone();

            if current_buffer.len() >= MAX_BUFFER || current_used.contains(&(r, c)) {
                return;
            }

            let is_valid_move = if *is_row_turn {
                r == *active_index
            } else {
                c == *active_index
            };

            if is_valid_move {
                current_used.insert((r, c));
                current_buffer.push(matrix[r][c].clone());

                let buffer_str = current_buffer.join("");
                for (i, target) in targets.iter().enumerate() {
                    let target_str = target.join("");
                    if buffer_str.contains(&target_str) {
                        current_completed.insert(i);
                    }
                }

                used_cells.set(current_used);
                buffer.set(current_buffer.clone());
                completed_targets.set(current_completed.clone());
                is_row_turn.set(!*is_row_turn);
                active_index.set(if *is_row_turn { c } else { r });

                if current_buffer.len() >= MAX_BUFFER || current_completed.len() == targets.len() {
                    end_game.emit((current_buffer, current_completed));
                }
            }
        })
    };

    html! {
        <div style={custom_style} class="flex flex-col md:flex-row gap-8 items-start justify-center p-8 bg-background text-foreground font-mono min-h-screen dark">
            <div class="flex flex-col gap-6 w-full max-w-lg">
                <div class="flex justify-between items-center mb-4">
                    <h1 class="text-3xl font-bold text-neon-purple animate-neon-flicker uppercase tracking-widest m-0">
                        { dict.title }
                    </h1>

                    <div class="flex flex-col items-end">
                        <div class="text-xs text-muted-foreground uppercase tracking-widest mb-1">
                            { dict.time_left }
                        </div>
                        <div class="w-24 h-10 flex items-center justify-center bg-card border-2 border-primary rounded shadow-md text-xl font-bold text-neon-cyan font-mono tracking-wider">
                            { format!("00:{:02}", *time_left) }
                        </div>
                    </div>
                </div>

                <div class="bg-card border border-border p-4 rounded-lg shadow-xl">
                    <div class="text-sm text-muted-foreground mb-2">{ dict.buffer }</div>
                    <div class="flex gap-2 mb-4">
                        { for buffer.iter().map(|token| html! {
                            <div class="w-10 h-10 flex items-center justify-center border-2 border-primary bg-primary text-primary-foreground font-bold">
                                { token }
                            </div>
                        }) }
                        { for (buffer.len()..MAX_BUFFER).map(|_| html! {
                            <div class="w-10 h-10 border-2 border-muted bg-transparent"></div>
                        }) }
                    </div>

                    <div class="text-sm font-bold tracking-widest uppercase">
                        { if *is_row_turn {
                            html! { <span class="text-neon-cyan">{ dict.act_horiz }</span> }
                        } else {
                            html! { <span class="text-neon-purple">{ dict.act_vert }</span> }
                        } }
                    </div>
                </div>

                <div class="flex flex-col gap-2 p-4 bg-popover rounded-lg border border-border">
                    { for matrix.iter().enumerate().map(|(r, row)| html! {
                        <div class="flex gap-2">
                            { for row.iter().enumerate().map(|(c, token)| {
                                let is_used = used_cells.contains(&(r, c));
                                let is_active = if *is_row_turn { r == *active_index } else { c == *active_index };
                                let can_click = is_active && !is_used && !*game_over;

                                let mut cell_classes = vec!["w-12", "h-12", "flex", "items-center", "justify-center", "text-lg", "font-bold", "border", "transition-colors", "cursor-pointer"];

                                if is_used {
                                    cell_classes.push("opacity-20");
                                    cell_classes.push("bg-muted");
                                    cell_classes.push("cursor-not-allowed");
                                } else if can_click {
                                    if *is_row_turn {
                                        cell_classes.push("border-secondary");
                                        cell_classes.push("text-neon-cyan");
                                        cell_classes.push("hover:bg-secondary");
                                    } else {
                                        cell_classes.push("border-primary");
                                        cell_classes.push("text-neon-purple");
                                        cell_classes.push("hover:bg-primary");
                                    }
                                } else {
                                    cell_classes.push("border-transparent");
                                    cell_classes.push("text-muted-foreground");
                                    cell_classes.push("opacity-50");
                                }

                                let onclick = {
                                    let on_cell_click = on_cell_click.clone();
                                    Callback::from(move |_| if can_click { on_cell_click.emit((r, c)) })
                                };

                                html! { <div class={classes!(cell_classes)} onclick={onclick}>{ token }</div> }
                            }) }
                        </div>
                    }) }
                </div>
            </div>

            <div class="flex flex-col gap-4 w-full max-w-sm mt-16">
                <div class="text-lg font-bold border-b border-border pb-2 mb-2 text-neon-cyan uppercase tracking-widest">
                    { dict.demons }
                </div>

                { for targets.iter().enumerate().map(|(i, target)| {
                    let is_completed = completed_targets.contains(&i);
                    let reward = props.config.base_value * (i as u32 + 1);

                    html! {
                        <div class={classes!(
                            "flex", "flex-col", "gap-2", "p-3", "border", "rounded",
                            if is_completed { vec!["border-neon-cyan", "bg-secondary/10"] } else { vec!["border-muted"] }
                        )}>
                            <div class="flex justify-between text-xs text-muted-foreground uppercase tracking-wider">
                                <span>{ format!("{} {}", dict.level, i + 1) }</span>
                                <span class={if is_completed { "text-neon-cyan font-bold" } else { "" }}>
                                    { if is_completed { dict.loaded } else { dict.waiting } }
                                </span>
                            </div>
                            <div class="flex gap-2">
                                { for target.iter().map(|token| html! {
                                    <span class={classes!(
                                        "font-bold",
                                        if is_completed { vec!["text-neon-cyan", "text-shadow"] } else { vec!["text-foreground"] }
                                    )}>{ token }</span>
                                }) }
                            </div>
                            <div class="text-xs text-neon-purple mt-1">{ format!("{}: {} ₴", dict.reward, reward) }</div>
                        </div>
                    }
                }) }

                { if *game_over {
                    html! {
                        <div class="mt-8 p-4 bg-primary text-primary-foreground font-bold text-center animate-pulse uppercase tracking-widest border border-neon-purple shadow-[0_0_15px_#8b3dff]">
                            { if *time_left == 0 { dict.fail } else { dict.success } }
                        </div>
                    }
                } else { html! {} } }
            </div>
        </div>
    }
}

#[wasm_bindgen(js_name = initCyberHack)]
pub fn init_cyber_hack(element_id: &str, config_json: &str) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let el = document
        .get_element_by_id(element_id)
        .expect("Не найден элемент для монтирования");

    let config: GameConfig = serde_json::from_str(config_json)
        .map_err(|e| JsValue::from_str(&format!("Ошибка парсинга конфига: {}", e)))?;

    let props = GameProps { config };

    yew::Renderer::<CyberHackGame>::with_root_and_props(el, props).render();
    Ok(())
}

#[wasm_bindgen(start)]
pub fn run_app() {
    let debug_config = GameConfig {
        redirect_url: "/debug".to_string(),
        base_value: 10,
        time_limit: 30,
        locale: Some("en".to_string()),
        theme: None,
    };

    yew::Renderer::<CyberHackGame>::with_props(GameProps {
        config: debug_config,
    })
    .render();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_coins() {
        let base = 100;
        // Демон 0 (множитель 1) + Демон 2 (множитель 3) = 100*1 + 100*3 = 400
        assert_eq!(calculate_coins(base, &[0, 2]), 400);
        // Все три демона = 100*1 + 100*2 + 100*3 = 600
        assert_eq!(calculate_coins(base, &[0, 1, 2]), 600);
        // Нет демонов = 0
        assert_eq!(calculate_coins(base, &[]), 0);
    }

    #[test]
    fn test_board_generation_size() {
        let (matrix, targets) = generate_solvable_board();

        // Проверяем размеры матрицы
        assert_eq!(matrix.len(), GRID_SIZE);
        for row in &matrix {
            assert_eq!(row.len(), GRID_SIZE);
        }

        // Проверяем генерацию 3 целей
        assert_eq!(targets.len(), 3);
        assert!(targets[0].len() <= 2);
        assert!(targets[1].len() <= 3);
        assert!(targets[2].len() <= 4);
    }

    #[test]
    fn test_board_contains_valid_tokens() {
        let (matrix, _) = generate_solvable_board();
        for row in matrix {
            for token in row {
                assert!(
                    TOKENS.contains(&token.as_str()),
                    "Найден неизвестный токен: {}",
                    token
                );
            }
        }
    }

    #[test]
    fn test_dict_fallback() {
        let dict_en = get_dict("en");
        assert_eq!(dict_en.title, "Breach Protocol");

        // Неизвестный язык должен отдавать русский
        let dict_fr = get_dict("fr");
        assert_eq!(dict_fr.title, "Взлом протокола");
    }
}
