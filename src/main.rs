extern crate ggez;
use std::path;
use std::env;
use ggez::event;
use ggez::input;
use ggez::graphics::{self, Drawable, DrawParam};
use ggez::nalgebra;
use ggez::{Context, GameResult};
use ggez::timer;
use ggez::audio;
use ggez::audio::SoundSource;
use std::io::BufReader;
use std::fs::File;
use hound;

enum GameState
{
    Menu,
    Game
}

struct State 
{
    game_state: GameState,
    img_town: graphics::Image,
    img_beat: graphics::Image,
    img_paddle: graphics::Image,
    dtime: std::time::Duration,
    time: std::time::Duration,
    beats: Vec<Beat>,
    paddle_pos_x: f32,
    paddle_pos_y: f32,
    wav_reader: hound::WavReader<BufReader<File>>,
    beat_count: i16,
    missed_count: i16,
    hit_count: i16,
    music: audio::Source,
    stage: Option<MenuItem>,
    menu_items: Vec<MenuItem>,
}

struct Beat
{
    img: graphics::Image,
    pos_x: f32,
    pos_y: f32
}

#[derive(Clone)]
struct MenuItem
{
    text: String,
    pos_x: f32,
    pos_y: f32,
    width: f32,//for mouse collision
    height: f32,//for mouse collision
    colour: graphics::Color,
    file_location: String,
    bpm: i32
    
}

impl State {
    fn new(ctx: &mut Context) -> GameResult<State> 
    {
        let s = State 
        {
            game_state: GameState::Menu,
            img_town: graphics::Image::new(ctx, "/graphics/town.png")?,
            img_beat: graphics::Image::new(ctx, "/graphics/beat.png")?,
            img_paddle: graphics::Image::new(ctx, "/graphics/paddle.png")?,
            dtime: std::time::Duration::new(0, 0),
            time: std::time::Duration::new(0, 0),
            beats: vec![],
            paddle_pos_x: 0.0,
            paddle_pos_y: (graphics::drawable_size(ctx).1 - 130.0) as f32,
            wav_reader: hound::WavReader::open("resources/music/song1.wav").unwrap(),//default
            beat_count: 0,
            missed_count: 0,
            hit_count: 0,
            music: audio::Source::new(ctx, "/music/song1.wav")?,//default
            stage: None,
            menu_items: vec![
                MenuItem{text: "Song1".to_string(), pos_x: 200.0, pos_y: 200.0, width: 150.0, height: 40.0, colour: graphics::Color::new(0.2, 1.0, 0.2, 1.0), file_location: "/music/song1.wav".to_string(), bpm: 130},
                MenuItem{text: "Song2".to_string(), pos_x: 200.0, pos_y: 250.0, width: 150.0, height: 40.0, colour: graphics::Color::new(0.2, 1.0, 0.2, 1.0), file_location: "/music/song2.wav".to_string(), bpm: 145},
                MenuItem{text: "Song3".to_string(), pos_x: 200.0, pos_y: 300.0, width: 150.0, height: 40.0, colour: graphics::Color::new(0.2, 1.0, 0.2, 1.0), file_location: "/music/song3.wav".to_string(), bpm: 160}
            ]
        };
        Ok(s)
    }
}

fn rect_collision(rect_x: f32, rect_y: f32, rect_width: f32, rect_height: f32, point_x: f32, point_y: f32) -> bool {
    point_x > rect_x && point_x < rect_x + rect_width && point_y > rect_y && point_y < rect_y + rect_height//returns true/false
}

impl event::EventHandler for State {
    
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if let GameState::Menu = self.game_state {
            for item in &mut self.menu_items {
                if rect_collision(item.pos_x, item.pos_y, item.width, item.height, input::mouse::position(ctx).x, input::mouse::position(ctx).y) {
                    item.colour = graphics::Color::new(1.0, 1.0, 1.0, 1.0);
                    if input::mouse::button_pressed(ctx, ggez::input::mouse::MouseButton::Left) {
                        self.game_state = GameState::Game;
                        self.stage = Some(item.clone());
                        self.music = audio::Source::new(ctx, &item.file_location)?;
                        self.music.set_volume(0.05);
                        self.music.play()?;
                        let path = format!("resources{}", item.file_location);
                        self.wav_reader = hound::WavReader::open(path).unwrap();
                        self.beat_count = 0;
                        self.missed_count = 0;
                        self.hit_count = 0;
                        self.time = std::time::Duration::new(0, 0);
                        self.paddle_pos_y = (graphics::drawable_size(ctx).1 - 130.0) as f32;              
                    }
                }
                else if item.colour == graphics::Color::new(1.0, 1.0, 1.0, 1.0)
                {
                    item.colour = graphics::Color::new(0.2, 1.0, 0.2, 1.0);
                }
            }
        }
        else if let GameState::Game = self.game_state
        {
            self.paddle_pos_x = input::mouse::position(ctx).x;
            let beat_length = 60.0/self.stage.clone().unwrap().bpm as f32;//in seconds
            let required_beats = (self.time.as_millis() as f32)/(beat_length*1000.0);
            let seek_loc = (beat_length * (self.beat_count as f32) * self.wav_reader.spec().sample_rate as f32) as u32;
            if (self.beat_count as f32) < required_beats {
                //create new beat
                self.beat_count += 1;                        
                if seek_loc < self.wav_reader.duration() {//there are samples left
                    self.wav_reader.seek(seek_loc)?;
                }
                let mut beat_amp = self.wav_reader.samples::<i16>().next().unwrap().unwrap() as i32;
                beat_amp = beat_amp.abs();
                let screen_width = graphics::drawable_size(ctx).0;
                let start_x = (beat_amp * 100) % screen_width as i32;
                if seek_loc < self.wav_reader.duration() {//there are samples left
                    let new_beat = Beat {
                        img: self.img_beat.clone(),
                        pos_x: start_x as f32,
                        pos_y: 0.0,
                    };
                    self.beats.push(new_beat);
                };
            }
            
            let mut remove_beats = vec![];
            for (i, entity) in self.beats.iter_mut().enumerate() {
                let screen_height = graphics::drawable_size(ctx).1;
                if entity.pos_y > screen_height as f32 {
                    remove_beats.push(i);
                    self.missed_count += 1;
                }
                else {
                    //paddle collision  
                    let paddle_width = 110.0;
                    let collision_points = 4;
                    for seg in 0..collision_points {
                        let step = (seg as f32)*paddle_width/((collision_points-1) as f32) ;
                        if rect_collision(entity.pos_x, entity.pos_y, entity.img.width() as f32, entity.img.height() as f32, self.paddle_pos_x + step, self.paddle_pos_y) {
                            remove_beats.push(i);
                            self.hit_count += 1;
                            break;
                        }
                    }
                    entity.pos_y += 7.0;
                }
            }

            remove_beats.reverse(); 
            for entity_number in remove_beats {
                self.beats.remove(entity_number);
            }
            if self.beats.len() == 0 && seek_loc > self.wav_reader.duration() {
                self.game_state = GameState::Menu
            }
            self.dtime = timer::delta(ctx);
            self.time = self.time.checked_add(self.dtime).unwrap();
        }        
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let samples_left = self.wav_reader.samples::<i16>().len() as f32;
        let samples_total = self.wav_reader.len() as f32;
        let screen_width = graphics::drawable_size(ctx).0;
        let marker_pos = ((samples_total-samples_left)/samples_total)*screen_width;
        graphics::Mesh::new_line(
            ctx,
            &[nalgebra::Point2::new(marker_pos, 0.0), nalgebra::Point2::new(marker_pos, 100.0)],
            1.0,
            graphics::Color::new(0.2, 1.0, 0.2, 1.0)
        )?
        .draw(ctx, DrawParam::default())?;

        if let GameState::Menu = self.game_state {
            for item in &self.menu_items {
                graphics::Text::new(item.text.clone())
                    .draw(ctx, DrawParam::default()
                    .dest(nalgebra::Point2::new(item.pos_x, item.pos_y)).color(item.colour))?;
                graphics::Text::new(format!("BPM: {}",item.bpm))
                    .draw(ctx, DrawParam::default()
                    .dest(nalgebra::Point2::new(item.pos_x, item.pos_y+20.0)).color(item.colour))?;
            }
        }
        graphics::Text::new(format!("Missed: {}",self.missed_count.to_string()))
            .draw(ctx, DrawParam::default().dest(nalgebra::Point2::new(100.0, 60.0)))?;
        
        graphics::Text::new(format!("Blocked: {}",self.hit_count.to_string()))
            .draw(ctx, DrawParam::default().dest(nalgebra::Point2::new(100.0, 80.0)))?;

        graphics::Text::new(format!("Total: {}", (self.hit_count+self.missed_count).to_string()))
            .draw(ctx, DrawParam::default().dest(nalgebra::Point2::new(100.0, 100.0)))?;

        self.img_town.draw(ctx, DrawParam::default()
        .dest(nalgebra::Point2::new(0.0, graphics::drawable_size(ctx).1 as f32))
        .offset(nalgebra::Point2::new(0.0, 1.0)))?;

        self.img_paddle.draw(ctx, DrawParam::default()
        .dest(nalgebra::Point2::new(self.paddle_pos_x, self.paddle_pos_y)))?;

        for entity in &self.beats {
            entity.img.draw(ctx, DrawParam::default()
            .dest(nalgebra::Point2::new(entity.pos_x, entity.pos_y)))?
        }

        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("super_simple", "ggez").add_resource_path(resource_dir);
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut State::new(ctx)?;
    graphics::set_fullscreen(ctx, ggez::conf::FullscreenType::True)?;
    //graphics::set_drawable_size(ctx, 1920.0, 1080.0)?;
    let rect = graphics::Rect::new(0.0, 0.0, 1920.0, 1080.0);
    graphics::set_screen_coordinates(ctx, rect)?;
    event::run(ctx, event_loop, state)
}