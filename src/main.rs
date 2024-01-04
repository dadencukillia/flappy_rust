#![cfg_attr(
    all(
        target_os = "windows",
        not(feature = "console"),
    ),
    windows_subsystem = "windows"
)]

use std::path::PathBuf;

use ggez::audio::{Source, SoundSource};
use ggez::conf::{WindowSetup, WindowMode};
use ggez::glam::Vec2;
use ggez::winit::event::VirtualKeyCode;
use ggez::{Context, ContextBuilder, GameResult};
use ggez::graphics::{self, Color, DrawParam, Image, Sampler, FontData, Text, TextLayout};
use ggez::event::{self, EventHandler};
use rand::prelude::*;
use rand::thread_rng;

fn main() {
    let res_path;
    if cfg!(debug_assertions) {
        res_path = std::env::current_dir().unwrap().join(PathBuf::from("resources"));
    } else {
        res_path = std::env::current_exe().unwrap().parent().unwrap().join(PathBuf::from("resources"));
    }

    let (mut ctx, event_loop) = ContextBuilder::new("Flappy Rust", "Crocoby")
        .window_setup(WindowSetup::default().title("Flappy Rust"))
        .window_mode(WindowMode::default().dimensions(1280.0, 720.0))
        .add_resource_path(res_path)
        .build()
        .expect("Error running window");

    let my_game = MyGame::new(&mut ctx);

    event::run(ctx, event_loop, my_game);
}

struct MyGame {
    player_sprite: Image,
    ground_sprite: Image,
    upper_sprite: Image,
    lower_sprite: Image,

    scoreup: Source,
    jump: Source,
    loose: Source,

    texts: Vec<Text>,
    score: i32,

    offset_x: u128,

    game_started: bool,
    player_y: f32,
    player_dir: f32,
    player_died: bool,

    last_block: u128,
    blocks_list: Vec<(i32, i32)>,
    random: rand::rngs::ThreadRng,
}

impl MyGame {
    pub fn new(ctx: &mut Context) -> MyGame {
        let player_sprite = Image::from_path(ctx, "/player.png").unwrap();
        let ground_sprite = Image::from_path(ctx, "/ground.png").unwrap();
        let upper_sprite = Image::from_path(ctx, "/upper_part.png").unwrap();
        let lower_sprite = Image::from_path(ctx, "/lower_part.png").unwrap();

        let scoreup = Source::new(ctx, "/scoreup.wav").unwrap();
        let jump = Source::new(ctx, "/jump.wav").unwrap();
        let loose = Source::new(ctx, "/loose.wav").unwrap();

        let font = FontData::from_path(ctx, "/NotoSans.ttf").unwrap();
        ctx.gfx.add_font("notosans", font);
        
        let mut texts: Vec<Text> = Vec::with_capacity(2);
        texts.push(Text::new("Press SPACE button to start").set_font("notosans").set_wrap(false).set_layout(TextLayout::center()).set_scale(64.0).clone());
        texts.push(Text::new("Press R button to play again").set_font("notosans").set_wrap(false).set_layout(TextLayout::center()).set_scale(64.0).clone());

        MyGame {
            player_sprite,
            ground_sprite,
            upper_sprite,
            lower_sprite,

            scoreup,
            jump,
            loose,

            texts,
            score: 0,

            offset_x: 0,

            game_started: false,
            player_y: 200.0,
            player_dir: 1.0,
            player_died: false,

            last_block: 0,
            blocks_list: Vec::new(),

            random: thread_rng()
        }
    }

    pub fn generate_new_block(&mut self, width: u32, height: u32) {
        if self.blocks_list.len() == 0 {
            let y = self.random.gen_range(300..=height-200);
            self.blocks_list.push((width as i32, y as i32));
        } else {
            let last_y = self.blocks_list.last().unwrap().1;
            let y = self.random.gen_range(300.max(last_y-100)..=(height as i32-200).min(last_y+100));
            self.blocks_list.push((width as i32, y));
        }
    }

    pub fn reset_values(&mut self) {
        self.offset_x = 0;
        self.score = 0;
        self.game_started = false;
        self.player_y = 200.0;
        self.player_dir = 1.0;
        self.player_died = false;
        self.last_block = 0;
        self.blocks_list.clear();
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if !self.game_started {
            return Ok(());
        }
        let sc_co = ctx.gfx.window().inner_size();

        self.player_y += self.player_dir*(ctx.time.delta().as_millis() as f32/1.5);
        self.player_dir += ctx.time.delta().as_millis() as f32/400.0;
        if self.player_dir > 1.0 {
            self.player_dir = 1.0;
        }

        if !self.player_died {
            self.offset_x += ctx.time.delta().as_millis() / 4;
            if self.offset_x != 0 {
                self.offset_x = self.offset_x % (self.ground_sprite.width() as u128);
            }

            if self.player_y > (sc_co.height-self.ground_sprite.height()) as f32 {
                self.loose.play_detached(ctx).unwrap();
                self.player_dir = -1.5;
                self.player_died = true;
            } else if self.player_y < 0.0 {
                self.loose.play_detached(ctx).unwrap();
                self.player_dir = 0.0;
                self.player_died = true;
            }
        

            let current_time = ctx.time.time_since_start().as_millis();
            if current_time-self.last_block > 2500 || self.last_block == 0 {
                self.last_block = current_time;
                self.generate_new_block(sc_co.width, sc_co.height);
            }

            let leftpart_player = 200-16;
            let rightpart_player = 200+16;
            let bottompart_player = self.player_y as i32+16;
            let toppart_player = self.player_y as i32-16;
            for ls in self.blocks_list.iter_mut() {
                let prev_rightpart = ls.0+(self.upper_sprite.width() as i32)*5;
                ls.0 -= (ctx.time.delta().as_millis() / 4) as i32;
                let leftpart_block = ls.0;
                let rightpart_block = ls.0+(self.upper_sprite.width() as i32)*5;
                let toppart_bblock = ls.1+100;
                let bottompart_tblock = ls.1-100;

                if leftpart_player < prev_rightpart && leftpart_player >= rightpart_block {
                    self.score += 1;
                    self.scoreup.play_detached(ctx).unwrap();
                } else if rightpart_player > leftpart_block && leftpart_player < rightpart_block {
                    if toppart_player < bottompart_tblock || bottompart_player > toppart_bblock {
                        self.player_died = true;
                        self.loose.play_detached(ctx).unwrap();
                    }
                }
            }

            self.blocks_list = self.blocks_list.iter().filter(|x| x.0 > -(self.upper_sprite.width() as i32)*5).cloned().collect();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from_rgb(78, 221, 244));
        let sc_co = canvas.screen_coordinates().unwrap();
        canvas.set_sampler(Sampler::nearest_clamp());

        for i in (0..=(sc_co.w as usize)+(self.offset_x as usize)).step_by(self.ground_sprite.width() as usize) {
            let x = (i as i128)-(self.offset_x as i128);
            canvas.draw(&self.ground_sprite, DrawParam::new().dest(Vec2::new(x as f32, sc_co.h-(self.ground_sprite.height() as f32))));
        }

        let player_rotate;
        if self.player_died {
            player_rotate = self.player_y/100.0;
        } else {
            if self.player_dir < 1.0 {
                player_rotate = (1.0-self.player_dir.abs())/4.0;
            } else {
                player_rotate = 0.0;
            }
        }

        for block in self.blocks_list.iter() {
            canvas.draw(&self.upper_sprite, DrawParam::new().dest(Vec2::new(block.0 as f32, block.1 as f32 + 100.0)).scale(Vec2::new(5.0, 5.0)));
            for y in ((block.1+100+(self.upper_sprite.height() as i32)*5)..=sc_co.h as i32).step_by(self.lower_sprite.height() as usize*5) {
                canvas.draw(&self.lower_sprite, DrawParam::new().dest(Vec2::new(block.0 as f32, y as f32)).scale(Vec2::new(5.0, 5.0)));
            }

            canvas.draw(&self.upper_sprite, DrawParam::new().dest(Vec2::new(block.0 as f32, block.1 as f32 - 100.0)).scale(Vec2::new(5.0, -5.0)));
            for y in (0..=block.1-100-(self.upper_sprite.height() as i32)*5).rev().step_by((self.lower_sprite.height()*5) as usize) {
                canvas.draw(&self.lower_sprite, DrawParam::new().dest(Vec2::new(block.0 as f32, y as f32)).scale(Vec2::new(5.0, -5.0)));
            }
        }

        canvas.draw(&self.player_sprite, DrawParam::new().dest(Vec2::new(200.0, self.player_y)).scale(Vec2::new(3.0, 3.0)).rotation(player_rotate).offset(Vec2::new(0.5, 0.5)));

        let score_text = Text::new("Score: ".to_string()+self.score.to_string().as_str()).set_font("notosans").set_wrap(false).set_scale(32.0).clone();
        canvas.draw(&score_text, DrawParam::new().dest(Vec2::new(10.0, 10.0)));

        if !self.game_started {
            canvas.draw(self.texts.get(0).unwrap(), DrawParam::new().dest(Vec2::new(sc_co.w/2.0, sc_co.h/2.0)).offset(Vec2::new(0.0, 0.5)));
        } else if self.player_died {
            canvas.draw(self.texts.get(1).unwrap(), DrawParam::new().dest(Vec2::new(sc_co.w/2.0, sc_co.h/2.0)).offset(Vec2::new(0.0, 0.5)));
        }

        canvas.finish(ctx)
    }

    fn key_down_event(&mut self, ctx: &mut Context, input: ggez::input::keyboard::KeyInput, _repeat: bool) -> GameResult {
        match input.keycode {
            Some(VirtualKeyCode::Space) => {
                if !self.player_died {
                    self.player_dir = -1.0;

                    if !self.game_started {
                        self.game_started = true;
                    }

                    self.jump.play_detached(ctx).unwrap();
                }
            },
            Some(VirtualKeyCode::R) => {
                if self.player_died {
                    self.reset_values();
                }
            },
            _ => {}
        };
        Ok(())
    }
}
