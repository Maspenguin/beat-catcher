extern crate ggez;
use std::path;
use std::env;
extern crate rand;
//use rand::Rng;
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
use rustfft::algorithm::DFT;
use rustfft::FFT;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
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
    img_coolbeans: graphics::Image,
    img_coolbeans_missed: graphics::Image,
    dtime: std::time::Duration,
    time: std::time::Duration,
    beats: Vec<Beat>,
    missed_count: i32,
    paddle_pos_x: f32,
    paddle_pos_y: f32,
    wav_reader: hound::WavReader<BufReader<File>>,
    beat_count: i16,
    music: audio::Source,
    stage: Option<MenuItem>,
    menu_items: Vec<MenuItem>,
    points: Vec<nalgebra::Point2<f32>>,//diagram plots
    samples_per_point: usize
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
    width: f32,
    height: f32,
    colour: graphics::Color,
    track_name: String,
    bpm: i32
    
}

impl State {
    fn new(ctx: &mut Context) -> GameResult<State> 
    {
        // let mut reader = hound::WavReader::open("resources/WalkStabWalk.wav").unwrap();
        // println!("Spec: {:?}", reader.spec().sample_rate);
        // println!("Duration: {}", reader.duration());
        // for i in 0..10000
        // {
        //     //reader.seek(i * reader.spec().sample_rate)?;//.unwrap() ?
        //     println!("Seek: {}", reader.samples::<i16>().next().unwrap().unwrap());
        // }

        //let mut sqr_sum = 0.0;
        // for s in reader.samples::<i16>() 
        // {
        //     let sample = s.unwrap() as f64;
        //     sqr_sum += sample * sample;
        //     println!("Sample: {}", sample);
        // }
        //println!("RMS is {}", (sqr_sum / reader.len() as f64).sqrt());
        let s = State 
        {
            game_state: GameState::Menu,
            img_town: graphics::Image::new(ctx, "/graphics/town.png")?,
            img_beat: graphics::Image::new(ctx, "/graphics/beat.png")?,
            img_paddle: graphics::Image::new(ctx, "/graphics/paddle.png")?,
            img_coolbeans: graphics::Image::new(ctx, "/graphics/coolbeans.png")?,
            img_coolbeans_missed: graphics::Image::new(ctx, "/graphics/coolbeans_missed.png")?,
            //music: audio::Source::new(ctx, "/music.mp3")?, //145 bpm  // 60/145 seconds between each beat?
            dtime: std::time::Duration::new(0, 0),
            time: std::time::Duration::new(0, 0),
            beats: vec![],//CursedEntity {img: graphics::Image::new(ctx, "/wednesday.png")?, pos_x: 50.0, pos_y: 0.0}
            missed_count: 0,
            paddle_pos_x: 0.0,
            paddle_pos_y: (graphics::drawable_size(ctx).1 - 130.0) as f32,
            wav_reader: hound::WavReader::open("resources/music/test.wav").unwrap(),//DanceOfTheDecorous(Jazz).wav//WalkStabWalk.wav
            beat_count: 0,
            music: audio::Source::new(ctx, "/music/DanceOfTheProfane.mp3")?,
            stage: None,
            menu_items: vec![
                MenuItem{text: "Dance of the Profane".to_string(), pos_x: 200.0, pos_y: 200.0, width: 200.0, height: 40.0, colour: graphics::Color::new(0.2, 1.0, 0.2, 1.0), track_name: "/music/DanceOfTheProfane.mp3".to_string(), bpm: 145},
                MenuItem{text: "Walk-Stab-Walk".to_string(),       pos_x: 200.0, pos_y: 250.0, width: 200.0, height: 40.0, colour: graphics::Color::new(0.2, 1.0, 0.2, 1.0), track_name: "/music/test.wav".to_string(), bpm: 158},
                MenuItem{text: "Dance of the Decorous (Jazz)".to_string(), pos_x: 200.0, pos_y: 300.0, width: 200.0, height: 40.0, colour: graphics::Color::new(0.2, 1.0, 0.2, 1.0), track_name: "/music/DanceOfTheDecorous(Jazz).wav".to_string(), bpm: 145}
            ],
            points: vec![],
            samples_per_point: 370//2000//370//499//starts from 0/40//
        };
        Ok(s)
    }
}

fn rect_collision(rect_x: f32, rect_y: f32, rect_width: f32, rect_height: f32, point_x: f32, point_y: f32) -> bool
{
    point_x > rect_x && point_x < rect_x + rect_width && point_y > rect_y && point_y < rect_y + rect_height//returns true/false
}

impl event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult 
    {
        if let GameState::Menu = self.game_state
        {
            for item in &mut self.menu_items
            {
                if rect_collision(item.pos_x, item.pos_y, item.width, item.height, input::mouse::position(ctx).x, input::mouse::position(ctx).y)
                {
                    item.colour = graphics::Color::new(1.0, 1.0, 1.0, 1.0);
                    if input::mouse::button_pressed(ctx, ggez::input::mouse::MouseButton::Left)
                    {
                        self.game_state = GameState::Game;
                        self.stage = Some(item.clone());
                        self.music = audio::Source::new(ctx, &item.track_name)?;
                        self.paddle_pos_y = (graphics::drawable_size(ctx).1 - 130.0) as f32;
                        
                        let mut i = 0;
                        
                        while i < 2000 //next.is_some()
                        {
                            let next = self.wav_reader.samples::<i16>().nth(self.samples_per_point);
                            println!("Len: {}", self.wav_reader.samples::<i16>().len());
                            println!("Len2: {}", self.wav_reader.len());
                            
                            self.points.push(nalgebra::Point2::new(i as f32, (500 + (next.unwrap().unwrap()/200)) as f32));//40
                            i += 1;
                        }
                        let dft = DFT::new(2000, false);
                        let mut input: Vec<Complex<f32>> = vec![];
                        for point in &self.points
                        {
                            input.push(Complex::new(point.y,0.0));
                        }
                        // input[1] = Complex::new(100.0,0.0);
                        // input[5] = Complex::new(300.0,0.0);
                        // input[20] = Complex::new(10.0,0.0);
                        //Zero::zero(); 100
                        let mut output: Vec<Complex<f32>> = vec![Complex::new(0.0,0.0); 2000];
                        dft.process(&mut input, &mut output);
                        //println!("input: {:?}", input);
                        //println!("output: {:?}", output);
                        //println!("output: {}", output.len());
                        // println!("input: {:?}", self.points);
                        i = 0;
                        while i < self.points.len()
                        {
                            self.points[i] = nalgebra::Point2::new(i as f32, output[i].re+500.0);
                            i += 1;
                        }
                        // println!("output: {:?}", self.points);
                    
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
            // match self.stage {
            //     Some(ref p) => beat_length = 60.0/p.bpm as f32,//in seconds
            //     None => println!("Blah")
            // }
            
            let required_beats = (self.time.as_millis() as f32)/(beat_length*1000.0);
            //println!("Beat: {}", self.wav_reader.samples::<i16>().next().unwrap().unwrap());
            
            if (self.beat_count as f32) < required_beats
            {
                //create new beat
                self.beat_count += 1;
                // self.wav_reader.seek(((60.0/158.0) * (self.beat_count as f32)) as u32 * self.wav_reader.spec().sample_rate)?;
                
                
                println!("Beatlength: {}", beat_length);
                println!("Time: {}", self.time.as_millis() as f32/1000.0 );
                let seek_loc = (beat_length * (self.beat_count as f32) * self.wav_reader.spec().sample_rate as f32) as u32;//todo: this is a bunch slower than the music plays
                //let seek_loc = ((60*3+29) * (self.wav_reader.spec().sample_rate)) as u32;
                self.wav_reader.seek(seek_loc)?;
                println!("Duration: {}", (self.wav_reader.duration()));
                println!("SeekLoc: {}", seek_loc);
                // let mut beat_value = self.wav_reader.samples::<i16>().next().unwrap().unwrap();
                // if beat_value < 0
                // {
                //     beat_value = beat_value *-1;
                // }

                let mut frequency = 0;
                //let mut last;
                //let mut next;
                frequency = self.wav_reader.samples::<i16>().next().unwrap().unwrap() as i32;
                frequency = frequency.abs();
                // if self.wav_reader.samples::<i16>().next().unwrap().unwrap() as i32 > 0
                // {
                //     last = 1;
                //     while last == 1
                //     {
                //         next = self.wav_reader.samples::<i16>().next().unwrap().unwrap() as i32;
                //         //println!("next: {}", next);
                //         if next > 0
                //         {
                //             frequency += 1;
                //             //println!("Freq: {}", frequency);
                //         }
                //         else
                //         {
                //             last = -1;
                //         }
                //     }                 
                // }
                // else
                // {
                //     last = -1;
                //     while last == -1
                //     {
                //         next = self.wav_reader.samples::<i16>().next().unwrap().unwrap() as i32;
                //         if next < 0
                //         {
                //             frequency += 1;
                //         }
                //         else
                //         {
                //             last = 1;
                //         }
                //     }      
                // }

                
                //println!("Beat: {}", beat_value);
                //let ce_type = rand::thread_rng().gen_range(0,1);

                // println!("Startx: {}", frequency);
                let start_x = (frequency * 100) % 1900;//1900 is "screenwidth"


                //self.points.push(nalgebra::Point2::new((5*self.beat_count) as f32, (100 + frequency/12) as f32));//for diagram
                
                
                //let start_x = rand::thread_rng().gen_range(200, graphics::drawable_size(ctx).0 as i32 - 200); //star_x old
                
                // println!("Startx: {}", (beat_value/1000)%1900);
                //let start_x = (beat_value/5)%1900;//1900 is "screenwidth"
                
 
                let new_ce = Beat
                {
                    img: self.img_beat.clone(),
                    pos_x: start_x as f32,
                    pos_y: 0.0,
                };
                self.beats.push(new_ce);


                //println!(" : {}", self.cursed_entities.len());
            }
            
            let mut remove_entities = vec![];
            for (i, entity) in self.beats.iter_mut().enumerate()
            {
                if self.music.stopped() //&& entity.pos_y > self.paddle_pos_y - 100.0 //todo: need a system to sync up the first beat
                {
                    self.music.play()?;//playtime
                }
                if entity.pos_y > graphics::drawable_size(ctx).1 as f32
                {
                    remove_entities.push(i);
                    self.missed_count += 1;
                }
                else
                {        
                    if rect_collision(entity.pos_x, entity.pos_y, entity.img.width() as f32 , entity.img.height() as f32, self.paddle_pos_x, self.paddle_pos_y) //todo loop  to make this nicer
                    || rect_collision(entity.pos_x, entity.pos_y, entity.img.width() as f32, entity.img.height() as f32, self.paddle_pos_x + 37.0, self.paddle_pos_y)
                    || rect_collision(entity.pos_x, entity.pos_y, entity.img.width() as f32, entity.img.height() as f32, self.paddle_pos_x + 73.0, self.paddle_pos_y)
                    || rect_collision(entity.pos_x, entity.pos_y, entity.img.width() as f32, entity.img.height() as f32, self.paddle_pos_x + 110.0, self.paddle_pos_y)
                    {
                        remove_entities.push(i);
                    }
                    // if entity.pos_y > (graphics::drawable_size(ctx).1 - 130.0) as f32 && entity.pos_y < (graphics::drawable_size(ctx).1 - 100.0) as f32
                    // {
                    //     if self.paddle_pos_x - 55.0 < entity.pos_x && entity.pos_x < self.paddle_pos_x + 55.0
                    //     {
                    //         remove_entities.push(i);
                    //     }
                    // }

                    entity.pos_y += 7.0;
                }
            }

            remove_entities.reverse(); 
            for entity_number in remove_entities {
                self.beats.remove(entity_number);
            }

            self.dtime = timer::delta(ctx);
            //println!("DTime: {}", self.dtime.as_millis());
            if self.dtime.as_millis() < 30//todo this is a hack
            {
                self.time = self.time.checked_add(self.dtime).unwrap();
            }
        }
        //println!(" : {}", self.dtime.subsec_micros());
        
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult
    {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());


        // for i in 0..500
        // {
        //     self.wav_reader.seek(((60.0/158.0)/64.0 * (i as f32) * self.wav_reader.spec().sample_rate as f32) as u32)?;
        //     println!("Seek: {}", (i as u32));
        //     let mut beat_value = self.wav_reader.samples::<i16>().next().unwrap().unwrap();
        //     if beat_value < 0
        //     {
        //         beat_value = beat_value*-1;
        //     }
        //     // for _i in 0 .. 1000
        //     // {
        //     //     let mut next = self.wav_reader.samples::<i16>().next().unwrap().unwrap() as i32;
        //     //     if next < 0
        //     //     {
        //     //         next  = -1 * next;
        //     //     }
        //     //     beat_value += next;

        //     // }
        //     println!("Beattt: {}", (beat_value / 1000));
        //     let _pixel = graphics::Mesh::new_rectangle(
        //         ctx,
        //         graphics::DrawMode::Fill,
        //         graphics::Rect::new(200.0 + (i*2) as f32, 400.0 + (beat_value as f32/100.0), 1.0, 1.0),
        //         graphics::Color::new(1.0, 1.0, 1.0, 1.0)
        //     )?.draw(ctx, DrawParam::default())?;
        // }

        // let vec = vec![5, 9, -2, -5];
        // let slice = &vec[0];
        // println!(" : {}", slice);



        // graphics::Mesh::new_circle
        // (
        //     ctx,
        //     graphics::DrawMode::fill(),
        //     nalgebra::Point2::new(300.0, 300.0),
        //     20.0,
        //     1.0,
        //     graphics::Color::new(0.2, 1.0, 0.2, 1.0)
        // )?
        
        // self.points.push(nalgebra::Point2::new(0.0, 500.0));
        // self.points.push(nalgebra::Point2::new(1000.0, 800.0));
        if self.points.len() >= 2 
        {
            graphics::Mesh::new_polyline
            (
                ctx,
                graphics::DrawMode::stroke(1.0),
                &self.points,
                graphics::Color::new(0.2, 1.0, 0.2, 1.0)
            )?
            .draw(ctx, DrawParam::default())?;
        }

        //let marker_points = vec![nalgebra::Point2::new(1.0, 700.0), nalgebra::Point2::new(2.0, 1000.0)];
        
        let samples_left = self.wav_reader.samples::<i16>().len() as f32;
        let samples_total = self.wav_reader.len() as f32;
        let marker_pos = (samples_total-samples_left)/(self.samples_per_point * 1920) as f32 * 1920.0;
        
        graphics::Mesh::new_line
        (
            ctx,
            &[nalgebra::Point2::new(marker_pos, 700.0), nalgebra::Point2::new(marker_pos, 1000.0)],
            1.0,
            graphics::Color::new(0.2, 1.0, 0.2, 1.0)
        )?
        .draw(ctx, DrawParam::default())?;
        if let GameState::Menu = self.game_state
        {
            for item in &self.menu_items
            {
                graphics::Text::new(item.text.clone()).draw(ctx, DrawParam::default()
                .dest(nalgebra::Point2::new(item.pos_x, item.pos_y))
                .color(item.colour))?;
            }
        }
        //let dst = cgmath::Point2::new(20.0, 20.0);
        //graphics::draw(ctx, &self.digs_img, DrawParam::default().dest(na::Point2::new(20.0, 20.0),))?;
        
        if self.missed_count == 0
        {
            self.img_coolbeans.draw(ctx, DrawParam::default()
            .dest(nalgebra::Point2::new(0.0, 0.0)))?;
        }
        else
        {
            self.img_coolbeans_missed.draw(ctx, DrawParam::default()
            .dest(nalgebra::Point2::new(0.0, 0.0)))?;
            if self.missed_count < 10
            {
                graphics::Text::new(self.missed_count.to_string()).draw(ctx, DrawParam::default().dest(nalgebra::Point2::new(49.0, 47.0)))?;
            }
            else
            {
                graphics::Text::new(self.missed_count.to_string()).draw(ctx, DrawParam::default().dest(nalgebra::Point2::new(45.0, 47.0)))?;
            }

        }


        self.img_town.draw(ctx, DrawParam::default()
        .dest(nalgebra::Point2::new(0.0, graphics::drawable_size(ctx).1 as f32))
        .offset(nalgebra::Point2::new(0.0, 1.0)))?;

        self.img_paddle.draw(ctx, DrawParam::default()
        .dest(nalgebra::Point2::new(self.paddle_pos_x, self.paddle_pos_y)))?;
        //.offset(nalgebra::Point2::new(0.5, 0.0)))?;

        for entity in &self.beats
        {
            entity.img.draw(ctx, DrawParam::default()
            .dest(nalgebra::Point2::new(entity.pos_x, entity.pos_y)))?
            //.offset(nalgebra::Point2::new(0.5, 1.0)))?;
        }

        // graphics::Mesh::new_circle
        // (
        //     ctx,
        //     graphics::DrawMode::Fill,
        //     nalgebra::Point2::new(300.0, 300.0),
        //     20.0,
        //     1.0,
        //     graphics::Color::new(0.2, 1.0, 0.2, 1.0)
        // )?
        // .draw(ctx, DrawParam::default())?;

        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult 
{
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
    graphics::set_drawable_size(ctx, 1020.0, 1080.0)?;
    let rect = graphics::Rect::new(0.0, 0.0, 1920.0, 1080.0);
    graphics::set_screen_coordinates(ctx, rect)?;
    event::run(ctx, event_loop, state)
}