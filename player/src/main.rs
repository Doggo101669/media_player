use raylib::ffi::UnloadMusicStream;
use raylib::prelude::*;
use simple_logger::*;
use std::fs;
use std::path::Path;
use std::process::exit;
use rand::prelude::*;

fn display_loading(rl: &mut RaylibHandle, thread: &RaylibThread, progress: i32, max_progress: i32) {
    let mut d_temp = rl.begin_drawing(thread);
    d_temp.clear_background(Color::RAYWHITE);
    d_temp.draw_text(format!("Loading... {}/{}", progress, max_progress).as_str(), 12, 12, 20, Color::BLACK);
}

fn color_from_hex(s: &str) -> Color {
    let hex = s.trim_start_matches('#');
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
            Color::new(r, g, b, 255)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
            let a = u8::from_str_radix(&hex[6..8], 16).unwrap();
            Color::new(r, g, b, a)
        }
        _ => panic!("Invalid hex string"),
    }
}

fn main() {
    SimpleLogger::new().env().init().unwrap();
    log::info!("Initializing RaylibHandle...");
    let (mut rl, thread) = raylib::init()
        .size(800, 600)
        .log_level(TraceLogLevel::LOG_WARNING)
        .title("Media Player")
        .build();

    rl.set_target_fps(240);

    let (mut progress, max_progress) = (0, 4);

    display_loading(&mut rl, &thread, progress, max_progress);

    log::info!("Initializing Audio Device...");
    progress = 1;
    display_loading(&mut rl, &thread, progress, max_progress);
    let mut audio_device = RaylibAudio::init_audio_device().unwrap();

    log::info!("Loading all songs...");
    progress = 2;
    display_loading(&mut rl, &thread, progress, max_progress);
    let mut songs: Vec<String> = Vec::new();

    let path = Path::new("..");

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_file() {
                        log::info!("Found file: {:?}", path);
                        if path.extension().and_then(|e| e.to_str()) == Some("mp3") {
                            log::info!("File is an mp3, adding file to songs list...");
                            songs.push(path.to_str().unwrap().to_string());
                        }
                    } else if path.is_dir() {
                        log::info!("Found directory: {:?}", path);
                    }
                }
                Err(e) => {log::error!("Error reading entry: {}", e);exit(-1)},
            }
        }
    } else {
        log::error!("Could not open directory");
    }

    if songs.len() == 0 {
        log::warn!("No mp3s found. Consider adding some or adding youtube links to links.txt.");
        log::warn!("If you are sure you have links, have you ran download.bash?");
    }

    log::info!("Entering main thread...");
    progress = 3;
    display_loading(&mut rl, &thread, progress, max_progress);

    let mut song_id: usize = 0;

    let mut last_song_id: usize = 1;
    let mut song_played: bool = false;
    let mut rng = rand::thread_rng();

    let color_dark0 = color_from_hex("#2e3440");
    let color_dark1 = color_from_hex("#3b4252");
    let color_dark2 = color_from_hex("#434c5e");
    let color_dark3 = color_from_hex("#4c566a");
    let color_light0 = color_from_hex("#eceff4");
    let color_light1 = color_from_hex("#e5e9f0");
    let color_light2 = color_from_hex("#d8dee9");
    let color_green = color_from_hex("#a3be8c");
    let color_frost0 = color_from_hex("#8fbcbb");
    let color_frost1 = color_from_hex("#88c0d0");

    let mut list_camera: Camera2D = Camera2D {
        offset: Vector2 { x: 100.0, y: 75.0 },
        target: Vector2 { x: 0.0, y: 0.0 },
        rotation: 0.0,
        zoom: 1.0
    };

    let mut scroll_smooth: f32 = 0.0;
    let mut scroll_down_limit: f32 = 0.0;

    let mut paused: bool = true;
    let mut shuffle: bool = false;

    let mut music: Option<Music> = None;

    while !rl.window_should_close() {
        /* :=-- PROCESSING --=: */
        let mut song_progress: f32 = 0.0;
        let mut song_length: f32 = 0.0;
        if last_song_id != song_id {
            if Option::is_none(&music) {
                log::info!("Skipping deallocation, music is None.");
                log::warn!("If this is not the first time loading a song,");
                log::warn!("this could mean either your mp3 is corrupt or something happened.");
            } else {
                log::info!("Deallocating music...");
                unsafe {ffi::UnloadMusicStream(music.unwrap().unwrap());}
            }
            log::info!("Loading Song ID: {}, Song {}", song_id, songs.get(song_id).unwrap());
            last_song_id = song_id;
            music = Some(audio_device.new_music(songs.get(song_id).unwrap()).unwrap());
        }

        if let Some(m) = music.as_mut() {
            m.looping = false;

            if !m.is_stream_playing() && !paused {
                if !song_played {
                    m.play_stream();
                    song_played = true;
                } else {
                    if shuffle {
                        let mut random_usize: usize = song_id;
                        while random_usize == song_id {
                            random_usize = rng.random_range(0..(songs.len()-1));
                        }
                        song_id = random_usize;
                    } else {
                        song_id = (song_id + 1) % songs.len();
                    }
                    song_played = false;
                }
            }

            m.update_stream();
            song_progress = m.get_time_played();
            song_length = m.get_time_length();
            if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
                if song_progress > song_length - 5.0 {
                    m.seek_stream(song_length);
                } else {
                    m.seek_stream(song_progress + 5.0);
                }
            }
            if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
                if song_progress < 5.0 {
                    m.seek_stream(0.0);
                } else {
                    m.seek_stream(song_progress - 5.0);
                }
            }
            if rl.is_key_pressed(KeyboardKey::KEY_UP) && rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) {
                m.seek_stream(song_length);
            }
            if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) && rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) {
                song_id = (song_id + 1) % songs.len();
                song_played = false;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_LEFT) && rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) {
                song_id = (song_id - 1) % songs.len();
                song_played = false;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                if paused {
                    paused = false;
                    m.resume_stream();
                } else {
                    paused = true;
                    m.pause_stream();
                }
            }
            if rl.is_key_pressed(KeyboardKey::KEY_S) {
                shuffle = !shuffle;
            }
        } else {
            // music is None, handle accordingly (maybe load music first)
            log::error!("music is None! song_id: {}, last_song_id {}, Option<song name>: {:?}", song_id, last_song_id, songs.get(song_id));
            exit(-1);
        }
        scroll_smooth += rl.get_mouse_wheel_move() * 5.0;
        list_camera.offset.y += scroll_smooth;
        scroll_smooth *= 0.9;
        if list_camera.offset.y > 75.0 {
            list_camera.offset.y = 75.0;
        }
        if list_camera.offset.y < -scroll_down_limit {
            list_camera.offset.y = -scroll_down_limit;
        }
        /* :=-- DRAWING --=: */
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(color_dark1);
        d.draw_rectangle(100, 75, 600, 450, color_dark0);
        // TODO: list
        let mut i: usize = 0;
        let mut c = d.begin_mode2D(list_camera);
        for song in &songs {
            let y_pos = 6 * (i as i32 + 1 + (i as i32 * 4));
            let world_y = c.get_world_to_screen2D(Vector2 {x: 0.0, y: y_pos as f32}, list_camera).y;
            if world_y < 525.0 && world_y > 30.0 {
                let back_color = match (song_id == i) {
                    true => color_frost0,
                    false => color_dark2,
                };
                c.draw_rectangle(6, y_pos, 588, 24, back_color);
                c.draw_text(format!("{}: {}", i + 1, song).as_str(), 24, y_pos + 7, 5, color_light0);
            }
            if i == songs.len() - 1 {
                scroll_down_limit = y_pos as f32 - 494.0;
            }
            // after all
            i += 1;
        }
        drop(c);
        d.draw_rectangle(100, 0, 600, 75, color_dark1);
        d.draw_rectangle(100, 525, 600, 75, color_dark1);
        // progress bar
        let text = format!("{}", song_progress as u32);
        let text = text.as_str();
        d.draw_text(text, ((800 / 2) - 212) - d.get_font_default().measure_text(text, 20.0, 1.0).x as i32, 565, 20, color_light0);
        d.draw_text(format!("{}", song_length as u32).as_str(), (800 / 2) + 212, 565, 20, color_light0);
        d.draw_rectangle((800 / 2) - 200, 564, 400, 20, color_dark2);
        d.draw_rectangle((800 / 2) - 196, 568, ((song_progress / song_length) * 396.0) as i32, 12, color_light0);
        // info
        d.draw_text(format!("FPS: {}", d.get_fps()).as_str(), 12, 48, 20, color_green);
        d.draw_text(format!("NOW PLAYING SONG ID: {}/{}", song_id + 1, songs.len()).as_str(), 12, 12, 20, color_light0);
        d.draw_text(format!("SONG NAME: {}", songs.get(song_id).unwrap()).as_str(), 12, 28, 5, color_light2);
        let shuffle_text = match shuffle {
            true => "SHUFFLE",
            false => "ORDER",
        };
        d.draw_text(shuffle_text, 12, 36, 5, color_light2);
    }
}
