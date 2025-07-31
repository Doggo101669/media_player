use raylib::prelude::*;
use simple_logger::*;
use std::fs;
use std::path::Path;
use std::process::exit;
use rand::prelude::*;

fn display_loading(rl: &mut RaylibHandle, thread: &RaylibThread, progress: i32, max_progress: i32) { /* This is for when it's loading to display it. */
    // begin new frame
    let mut d_temp = rl.begin_drawing(thread);
    // draw to screen Loading... <current stage>/<last stage>
    d_temp.clear_background(Color::RAYWHITE);
    d_temp.draw_text(format!("Loading... {}/{}", progress, max_progress).as_str(), 12, 12, 20, Color::BLACK);
}

fn color_from_hex(s: &str) -> Color { // chatgpt
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
    // Initialize simpleLogger
    SimpleLogger::new().env().init().unwrap();
    log::info!("Initializing RaylibHandle...");
    // initialize raylib with size(800, 600), log_level(warnings) so I can log myself, title("Media Player")
    let (mut rl, thread) = raylib::init()
        .size(800, 600)
        .log_level(TraceLogLevel::LOG_WARNING)
        .title("Media Player")
        .build();

    rl.set_target_fps(240);

    // for display_loading()
    let (mut progress, max_progress) = (0, 5);

    display_loading(&mut rl, &thread, progress, max_progress);

    log::info!("Initializing Audio Device...");
    // initialize the audio device and display the progress
    progress = 1;
    display_loading(&mut rl, &thread, progress, max_progress);
    let mut audio_device = RaylibAudio::init_audio_device().unwrap();

    // load all images and such
    log::info!("Loading Textures...");
    progress = 2;
    display_loading(&mut rl, &thread, progress, max_progress);
    let paused_texture = rl.load_texture(&thread, "res/paused.png").unwrap();
    let unpaused_texture = rl.load_texture(&thread, "res/unpaused.png").unwrap();

    log::info!("Loading all songs...");
    progress = 3;
    display_loading(&mut rl, &thread, progress, max_progress);
    // new list of song paths and display names 1. path, 2. name
    let mut songs: Vec<(String, String)> = Vec::new();

    // path is root of git
    let path = Path::new("..");

    // self explanitory
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_file() {
                        log::info!("Found file: {:?}", path);
                        if path.extension().and_then(|e| e.to_str()) == Some("mp3") {
                            log::info!("File is an mp3, adding file to songs list...");
                            let song_path_str = path.to_str().unwrap().to_string();
                            songs.push((song_path_str.clone(), song_path_str
                                .chars()
                                .clone()
                                .take(song_path_str.len() - 17)
                                .skip(3)
                                .collect()
                            ));
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
        exit(-1);
    }

    if songs.len() == 0 {
        log::warn!("No mp3s found. Consider adding some or adding youtube links to links.txt.");
        log::warn!("If you are sure you have links, have you ran download.bash?");
    }

    log::info!("Entering main thread...");
    progress = 4;
    display_loading(&mut rl, &thread, progress, max_progress);

    let mut song_id: usize = 0;

    // for detecting if song has changed. we set it to anything but song_id to that it thinks it changes and loads the song
    let mut last_song_id: usize = 1;
    // this is for if the song has started playing yet
    let mut song_played: bool = false;
    let mut rng = rand::thread_rng();

    // colors for theme
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

    // camera for scrolling list
    let mut list_camera: Camera2D = Camera2D {
        offset: Vector2 { x: 100.0, y: 75.0 },
        target: Vector2 { x: 0.0, y: 0.0 },
        rotation: 0.0,
        zoom: 1.0
    };

    // smoothing
    let mut scroll_smooth: f32 = 0.0;
    let mut scroll_down_limit: f32 = 0.0;

    let mut paused: bool = true;
    // should it shuffle, or play in order?
    let mut shuffle: bool = false;

    // current playing music
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
                unsafe {ffi::UnloadMusicStream(music.unwrap().unwrap());} // without this it will memory leak
            }
            log::info!("Loading Song ID: {}, Song {}", song_id, songs.get(song_id).unwrap().0);
            last_song_id = song_id;
            // get the song that is supposed to be loaded and load it
            music = Some(audio_device.new_music(&songs.get(song_id).unwrap().0).unwrap());
        }

        if let Some(m) = music.as_mut() {
            // disable looping so we can detect when the music stops
            m.looping = false;

            // if it's stopped and it's not justp paused, play it if it hasn't started already or if it has then go to next song
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
            /* :=-- CONTROLLS --=: */
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
                m.seek_stream(song_length); // go to end to simulate skipping (does this so it works with shuffle)
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
            log::error!("music is None! song_id: {}, last_song_id {}, Option<song name>: {:?}", song_id, last_song_id, songs.get(song_id));
            exit(-1);
        }
        // scrolling
        scroll_smooth += rl.get_mouse_wheel_move() * 5.0;
        list_camera.offset.y += scroll_smooth;
        scroll_smooth *= 0.9;
        // scroll limiting so you cant scroll past the list
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
                c.draw_text(format!("{}: {}", i + 1, song.1).as_str(), 24, y_pos + 7, 5, color_light0);
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
        d.draw_text(text, ((800 / 2) - 212) - d.get_font_default().measure_text(text, 20.0, 2.0).x as i32, 565, 20, color_light0);
        d.draw_text(format!("{}", song_length as u32).as_str(), (800 / 2) + 212, 565, 20, color_light0);
        d.draw_rectangle((800 / 2) - 200, 564, 400, 20, color_dark2);
        d.draw_rectangle((800 / 2) - 196, 568, ((song_progress / song_length) * 396.0) as i32, 12, color_light0);
        let shuffle_text = match shuffle {
            true => "SHUFFLE",
            false => "ORDER",
        };
        d.draw_text(shuffle_text, ((800 / 2) + 200) - (d.get_font_default().measure_text(shuffle_text, 20.0, 2.0).x as i32), 542, 20, color_light2);
        d.draw_text(format!("{}/{}", song_id + 1, songs.len()).as_str(), (800 / 2) - 200, 542, 20, color_light0);
        let (x_pos, y_pos) = ((800 / 2) - 8, 542);
        match paused {
            true => d.draw_texture(&paused_texture, x_pos, y_pos, Color::WHITE),
            false => d.draw_texture(&unpaused_texture, x_pos, y_pos, Color::WHITE),
        }
        // info
        d.draw_text(format!("FPS: {}", d.get_fps()).as_str(), 6, 576, 5, color_green);
        let song_name_text = songs.get(song_id).unwrap().1.as_str();
        d.draw_text(song_name_text, 6, 588, 5, color_light2);
    }

    // unload resources
    log::info!("Unloading Textures...");
    unsafe {
        ffi::UnloadTexture(paused_texture.unwrap());
        ffi::UnloadTexture(unpaused_texture.unwrap());
    } // without this it will memory leak
}
