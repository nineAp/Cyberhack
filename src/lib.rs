use gloo_timers::callback::{Interval, Timeout};
use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashSet, rc::Rc};
use wasm_bindgen::prelude::*;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};
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
            title: "BREACH PROTOCOL",
            buffer: "BUFFER_ALLOCATION:",
            act_horiz: "SEQ_ACT: HORIZONTAL",
            act_vert: "SEQ_ACT: VERTICAL",
            demons: "DAEMON_UPLOADS",
            level: "LVL",
            loaded: "UPLOADED",
            waiting: "STANDBY",
            reward: "BOUNTY",
            time_left: "SYS_TIMER:",
            success: "BREACH SUCCESSFUL. DISCONNECTING...",
            fail: "CRITICAL FAILURE. TRACING...",
        },
        _ => Dictionary {
            title: "ВЗЛОМ ПРОТОКОЛА",
            buffer: "БУФЕР_ОБМЕНА:",
            act_horiz: "АКТИВ_СЕКВ: ГОРИЗОНТАЛЬ",
            act_vert: "АКТИВ_СЕКВ: ВЕРТИКАЛЬ",
            demons: "ДОСТУПНЫЕ_ДЕМОНЫ",
            level: "УР",
            loaded: "ЗАГРУЖЕНО",
            waiting: "ОЖИДАНИЕ",
            reward: "НАГРАДА",
            time_left: "ТАЙМЕР_СИСТЕМЫ:",
            success: "ВЗЛОМ ЗАВЕРШЕН. РАЗРЫВ СОЕДИНЕНИЯ...",
            fail: "КРИТИЧЕСКАЯ ОШИБКА. ИДЕТ ОТСЛЕДЖИВАНИЕ...",
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

#[derive(Properties, PartialEq)]
pub struct BgProps {
    pub theme: Option<ThemeConfig>,
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

#[function_component(MatrixBackground)]
fn matrix_background(props: &BgProps) -> Html {
    let canvas_ref = use_node_ref();

    let primary = props
        .theme
        .as_ref()
        .and_then(|t| t.primary.clone())
        .unwrap_or_else(|| "#8b3dff".into());
    let secondary = props
        .theme
        .as_ref()
        .and_then(|t| t.secondary.clone())
        .unwrap_or_else(|| "#00f0ff".into());
    let bg_color = props
        .theme
        .as_ref()
        .and_then(|t| t.background.clone())
        .unwrap_or_else(|| "#05020a".into());

    {
        let canvas_ref = canvas_ref.clone();
        use_effect_with(canvas_ref, move |canvas_ref| {
            let canvas = canvas_ref
                .cast::<HtmlCanvasElement>()
                .expect("Canvas not found");
            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();

            let window = web_sys::window().unwrap();
            let width = window.inner_width().unwrap().as_f64().unwrap();
            let height = window.inner_height().unwrap().as_f64().unwrap();

            canvas.set_width(width as u32);
            canvas.set_height(height as u32);

            let font_size = 16.0;
            let columns = (width / font_size) as usize;

            let drops = Rc::new(RefCell::new(vec![0.0; columns]));
            let mut rng = rand::thread_rng();
            for drop in drops.borrow_mut().iter_mut() {
                *drop = rng.gen_range(0.0..(height / font_size));
            }

            let chars: Vec<char> = "0123456789ABCDEF".chars().collect();

            // Крутим анимацию через Interval (~30 fps), это проще чем городить requestAnimationFrame в Rust
            let interval = Interval::new(33, move || {
                let mut rng = rand::thread_rng();

                ctx.set_global_alpha(0.15);
                ctx.set_fill_style(&JsValue::from_str(&bg_color));
                ctx.fill_rect(0.0, 0.0, width, height);

                ctx.set_font(&format!("{}px monospace", font_size));

                let mut drops_ref = drops.borrow_mut();
                for i in 0..drops_ref.len() {
                    let text = chars[rng.gen_range(0..chars.len())].to_string();
                    let x = i as f64 * font_size;
                    let y = drops_ref[i] * font_size;

                    if rng.gen_bool(0.02) {
                        ctx.set_global_alpha(0.9);
                        ctx.set_fill_style(&JsValue::from_str(&primary));
                    } else {
                        ctx.set_global_alpha(0.4);
                        ctx.set_fill_style(&JsValue::from_str(&secondary));
                    }

                    let _ = ctx.fill_text(&text, x, y);

                    if y > height && rng.gen_bool(0.025) {
                        drops_ref[i] = 0.0;
                    }
                    drops_ref[i] += 1.0;
                }
            });

            // Очистка при анмаунте компонента
            move || drop(interval)
        });
    }
    html! {
        <canvas
            ref={canvas_ref}
            class="opacity-80 pointer-events-none"
            // Жестко прибиваем инлайном, чтобы флекс-контейнер его игнорировал
            style="position: absolute; top: 0; left: 0; z-index: 0;"
        ></canvas>
    }
}

pub fn calculate_coins(base_value: u32, completed_targets: &[usize]) -> u32 {
    completed_targets
        .iter()
        .map(|&idx| base_value * (idx as u32 + 1))
        .sum()
}

// Генерация фейкового адреса памяти для декораций
fn generate_hex_address(index: usize) -> String {
    format!("0x{:04X}", 0x00A0 + (index * 0x14))
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
                    Timeout::new(2500, move || {
                        let _ = win.location().assign(&url);
                    })
                    .forget();
                }
            },
        )
    };

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

    let time_percentage = (*time_left as f32 / props.config.time_limit as f32) * 100.0;
    let is_critical_time = *time_left <= 5 && *is_timer_running;

    html! {
        // 1. Главный контейнер на весь экран. Добавили flex items-center justify-center для жесткого центрирования.
        <div style={custom_style} class="flex items-center justify-center min-h-screen w-full relative overflow-hidden bg-black">

            // 2. Фон, вырванный из потока (убедись, что в самом компоненте MatrixBackground стоит class="fixed inset-0 z-0...")
            <MatrixBackground theme={props.config.theme.clone()} />

            // 3. CRT оверлей поверх всего
            <div class="crt-overlay pointer-events-none absolute inset-0 z-50"></div>

            // 4. Обертка самой игры
            <div class="flex flex-col xl:flex-row gap-8 items-start justify-center p-4 md:p-8 text-foreground font-mono relative z-10 max-w-7xl mx-auto w-full">

                // --- ЛЕВАЯ ПАНЕЛЬ (Матрица и таймер) ---
                <div class="flex flex-col gap-6 w-full max-w-2xl relative">
                    // Декоративные углы терминала
                    <div class="absolute -top-2 -left-2 w-8 h-8 border-t-2 border-l-2 border-neon-cyan opacity-70"></div>
                    <div class="absolute -bottom-2 -left-2 w-8 h-8 border-b-2 border-l-2 border-neon-cyan opacity-70"></div>
                    <div class="absolute -top-2 -right-2 w-8 h-8 border-t-2 border-r-2 border-neon-cyan opacity-70"></div>
                    <div class="absolute -bottom-2 -right-2 w-8 h-8 border-b-2 border-r-2 border-neon-cyan opacity-70"></div>

                    <div class="backdrop-blur-md bg-[#0a0510]/80 p-6 md:p-8 border border-white/5 shadow-[0_0_50px_rgba(0,0,0,0.8)] relative overflow-hidden">
                        // Технический хедер
                        <div class="flex items-center gap-4 mb-6 border-b border-white/10 pb-4">
                            <div class="flex gap-1">
                                <div class="w-8 h-2 bg-neon-cyan"></div>
                                <div class="w-2 h-2 bg-neon-cyan"></div>
                                <div class="w-1 h-2 bg-white/50"></div>
                            </div>
                            <h1 class="text-3xl md:text-5xl font-bold text-white tracking-[0.2em] uppercase m-0 flex-1 drop-shadow-md">
                                { dict.title }
                            </h1>
                            <div class="text-[10px] text-white/30 text-right leading-tight hidden md:block">
                                <p>{"SYS_VER: 2.0.4b"}</p>
                                <p>{"AUTH_REQ: ROOT"}</p>
                            </div>
                        </div>

                        // Таймер и статус
                        <div class="flex justify-between items-end mb-4">
                            <div class="flex flex-col">
                                <div class="text-sm font-bold tracking-widest flex items-center gap-2">
                                    { if *is_row_turn {
                                        html! { <><div class="w-3 h-3 bg-neon-cyan animate-pulse"></div><span class="text-neon-cyan">{ dict.act_horiz }</span></> }
                                    } else {
                                        html! { <><div class="w-3 h-3 bg-neon-purple animate-pulse"></div><span class="text-neon-purple">{ dict.act_vert }</span></> }
                                    } }
                                </div>
                            </div>
                            <div class="flex flex-col items-end">
                                <div class="text-xs text-white/50 uppercase tracking-widest mb-1 flex items-center gap-2">
                                    <span class="animate-pulse opacity-50">{"[ REC ]"}</span>
                                    { dict.time_left }
                                </div>
                                <div class={classes!(
                                    "text-3xl", "font-bold", "font-mono", "tracking-wider", "transition-colors", "duration-300",
                                    if is_critical_time { vec!["text-red-500", "animate-pulse", "drop-shadow-[0_0_8px_red]"] } else { vec!["text-white", "drop-shadow-md"] }
                                )}>
                                    { format!("00:{:02}", *time_left) }
                                </div>
                            </div>
                        </div>

                        // PROGRESS BAR ТАЙМЕРА
                        <div class="w-full h-1 bg-white/5 overflow-hidden mb-8 relative">
                            <div class="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNCIgaGVpZ2h0PSI0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxyZWN0IHdpZHRoPSIyIiBoZWlnaHQ9IjQiIGZpbGw9InJnYmEoMCwwLDAsMC41KSIvPjwvc3ZnPg==')] z-10 pointer-events-none"></div>
                            <div
                                class={classes!(
                                    "h-full", "transition-all", "duration-1000", "ease-linear", "relative", "z-0",
                                    if is_critical_time { vec!["bg-red-500", "shadow-[0_0_15px_red]"] } else { vec!["bg-neon-cyan", "shadow-[0_0_15px_cyan]"] }
                                )}
                                style={format!("width: {}%", time_percentage)}
                            ></div>
                        </div>

                        // БУФЕР ОБМЕНА
                        <div class="bg-black/80 border-t border-b border-white/10 py-4 mb-8 relative">
                            <div class="text-xs text-neon-cyan/70 mb-2 tracking-widest px-4">{ dict.buffer }</div>
                            <div class="flex gap-2 px-4 items-center">
                                { for buffer.iter().map(|token| html! {
                                    <div class="w-12 h-12 flex items-center justify-center border-2 border-neon-cyan bg-neon-cyan/10 text-neon-cyan font-bold text-xl shadow-[0_0_10px_rgba(0,240,255,0.3)]">
                                        { token }
                                    </div>
                                }) }

                                // Пустые слоты и мигающий курсор
                                { for (buffer.len()..MAX_BUFFER).map(|i| html! {
                                    <div class="w-12 h-12 flex items-center justify-center border-2 border-white/5 bg-white/5 relative">
                                        { if i == buffer.len() && !*game_over {
                                            html! { <div class="w-4 h-1 bg-neon-cyan animate-pulse absolute bottom-2"></div> }
                                        } else { html! {} } }
                                    </div>
                                }) }
                            </div>
                        </div>

                        // ИГРОВАЯ МАТРИЦА
                        <div class="flex justify-center p-4 relative">
                            // Сетка матрицы (декорация)
                            <div class="absolute inset-0 bg-[linear-gradient(rgba(255,255,255,0.02)_1px,transparent_1px),linear-gradient(90deg,rgba(255,255,255,0.02)_1px,transparent_1px)] bg-[size:20px_20px] pointer-events-none"></div>

                            <div class="flex flex-col gap-3 relative z-10">
                                { for matrix.iter().enumerate().map(|(r, row)| html! {
                                    <div class="flex gap-3 relative">
                                        // Подсветка активной строки
                                        { if *is_row_turn && r == *active_index && !*game_over {
                                            html! { <div class="absolute -inset-y-1 -inset-x-2 bg-secondary/20 border-y border-secondary/40 shadow-[0_0_10px_rgba(0,240,255,0.1)] pointer-events-none"></div> }
                                        } else { html!{} } }

                                        { for row.iter().enumerate().map(|(c, token)| {
                                            let is_used = used_cells.contains(&(r, c));
                                            let is_active = if *is_row_turn { r == *active_index } else { c == *active_index };
                                            let can_click = is_active && !is_used && !*game_over;

                                            let mut cell_classes = vec!["w-12", "h-12", "md:w-16", "md:h-16", "flex", "items-center", "justify-center", "text-xl", "md:text-2xl", "font-bold", "border-2", "transition-all", "duration-100", "relative", "z-10", "tracking-wider"];

                                            if is_used {
                                                cell_classes.push("opacity-10");
                                                cell_classes.push("cursor-not-allowed");
                                                cell_classes.push("border-transparent");
                                            } else if can_click {
                                                cell_classes.push("cursor-pointer");
                                                cell_classes.push("hover-glitch");
                                                cell_classes.push("hover:scale-105");
                                                if *is_row_turn {
                                                    cell_classes.push("border-secondary");
                                                    cell_classes.push("text-neon-cyan");
                                                    cell_classes.push("bg-secondary/10");
                                                    cell_classes.push("hover:bg-secondary/30");
                                                    cell_classes.push("shadow-[0_0_15px_rgba(0,240,255,0.4)]");
                                                } else {
                                                    cell_classes.push("border-primary");
                                                    cell_classes.push("text-neon-purple");
                                                    cell_classes.push("bg-primary/10");
                                                    cell_classes.push("hover:bg-primary/30");
                                                    cell_classes.push("shadow-[0_0_15px_rgba(139,61,255,0.4)]");
                                                }
                                            } else {
                                                cell_classes.push("border-transparent");
                                                cell_classes.push("text-white/40");
                                                cell_classes.push("cursor-default");
                                            }

                                            let onclick = {
                                                let on_cell_click = on_cell_click.clone();
                                                Callback::from(move |_| if can_click { on_cell_click.emit((r, c)) })
                                            };

                                            html! {
                                                <div class={classes!(cell_classes)} onclick={onclick}>
                                                    { token }
                                                </div>
                                            }
                                        }) }
                                    </div>
                                }) }
                            </div>
                        </div>
                    </div>
                </div>

                // --- ПРАВАЯ ПАНЕЛЬ (ДЕМОНЫ) ---
                <div class="flex flex-col gap-4 w-full max-w-sm mt-4 xl:mt-0 relative">
                    <div class="backdrop-blur-md bg-[#0a0510]/80 p-6 border border-white/5 shadow-[0_0_30px_rgba(0,0,0,0.8)]">
                        <div class="text-lg font-bold border-b border-white/20 pb-2 mb-4 text-white uppercase tracking-widest flex items-center justify-between">
                            <div class="flex items-center gap-2">
                                <svg class="w-5 h-5 text-neon-cyan" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z"></path></svg>
                                { dict.demons }
                            </div>
                            <div class="text-[10px] text-white/30">{"MEM_DUMP"}</div>
                        </div>

                        <div class="flex flex-col gap-4">
                            { for targets.iter().enumerate().map(|(i, target)| {
                                let is_completed = completed_targets.contains(&i);
                                let reward = props.config.base_value * (i as u32 + 1);
                                let mem_address = generate_hex_address(i);

                                html! {
                                    <div class={classes!(
                                        "flex", "flex-col", "gap-2", "p-4", "border-l-4", "border-y", "border-r", "transition-all", "duration-500", "relative", "overflow-hidden",
                                        if is_completed { vec!["border-l-neon-cyan", "border-y-neon-cyan/20", "border-r-neon-cyan/20", "bg-neon-cyan/5"] } else { vec!["border-l-white/20", "border-y-white/5", "border-r-white/5", "bg-white/5"] }
                                    )}>
                                        // Декоративный фон для завершенного демона
                                        { if is_completed {
                                            html! { <div class="absolute inset-0 bg-[linear-gradient(45deg,transparent_25%,rgba(0,240,255,0.05)_50%,transparent_75%)] bg-[size:20px_20px] pointer-events-none"></div> }
                                        } else { html! {} } }

                                        <div class="flex justify-between text-[10px] uppercase tracking-wider relative z-10">
                                            <span class="text-white/40">{ mem_address }</span>
                                            <span class={if is_completed { "text-neon-cyan font-bold" } else { "text-white/30" }}>
                                                { if is_completed { dict.loaded } else { dict.waiting } }
                                            </span>
                                        </div>
                                        <div class="flex gap-3 relative z-10">
                                            { for target.iter().map(|token| html! {
                                                <span class={classes!(
                                                    "font-bold", "text-xl", "tracking-wider",
                                                    if is_completed { vec!["text-neon-cyan", "drop-shadow-[0_0_5px_cyan]"] } else { vec!["text-white/80"] }
                                                )}>{ token }</span>
                                            }) }
                                        </div>
                                        <div class="text-xs text-neon-purple mt-2 flex justify-between relative z-10 border-t border-white/10 pt-2">
                                            <span>{ dict.reward }</span>
                                            <span class="font-bold">{ format!("{} ₴", reward) }</span>
                                        </div>
                                    </div>
                                }
                            }) }
                        </div>
                    </div>

                    // ПЛАШКА ЗАВЕРШЕНИЯ
                    { if *game_over {
                        html! {
                            <div class="fixed inset-0 flex items-center justify-center z-[100]">
                                <div class="absolute inset-0 bg-black/60 backdrop-blur-sm"></div>
                                <div class={classes!(
                                    "p-6", "text-center", "font-bold", "uppercase", "tracking-widest", "border-2", "shadow-2xl", "relative", "z-10", "w-full", "max-w-md",
                                    if *time_left == 0 { vec!["bg-red-950/90", "text-red-500", "border-red-500", "shadow-[0_0_40px_red]"] }
                                    else { vec!["bg-purple-950/90", "text-neon-cyan", "border-neon-cyan", "shadow-[0_0_40px_cyan]"] }
                                )}>
                                    // Декоративные глитч-полосы на плашке
                                    <div class="absolute top-0 left-0 w-full h-1 bg-white/20 animate-pulse"></div>
                                    <div class="absolute bottom-0 left-0 w-full h-1 bg-white/20 animate-pulse"></div>

                                    <div class={classes!(
                                        "text-3xl", "mb-2", "drop-shadow-lg",
                                        if *time_left == 0 { "animate-pulse" } else { "" }
                                    )}>
                                        { if *time_left == 0 { "SYSTEM LOCKOUT" } else { "ACCESS GRANTED" } }
                                    </div>
                                    <div class="text-xs opacity-90 text-white/80">{ if *time_left == 0 { dict.fail } else { dict.success } }</div>

                                    { if *time_left > 0 {
                                        html! {
                                            <div class="mt-6 border-t border-white/20 pt-4 text-neon-purple">
                                                <div>{"TOTAL BOUNTY SECURED:"}</div>
                                                <div class="text-2xl mt-1">{ format!("{} ₴", calculate_coins(props.config.base_value, &completed_targets.iter().copied().collect::<Vec<usize>>())) }</div>
                                            </div>
                                        }
                                    } else { html! {} } }
                                </div>
                            </div>
                        }
                    } else { html! {} } }
                </div>
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
