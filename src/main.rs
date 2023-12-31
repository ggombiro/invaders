use std::{error::Error, io, time::{Duration, Instant}, thread, sync::mpsc};
use crossterm::{terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand, cursor::{Hide, Show}, event::{self, KeyCode, Event}};
use invaders::{frame::{self, Drawable}, render, player::Player, invaders::Invaders};
use rusty_audio::Audio;

fn main() -> Result<(), Box<dyn Error>>{
    let mut audio = Audio::new();
    audio.add("explode","explosion.wav");
    audio.add("lose","game_over.wav");
    audio.add("move","move.wav");
    audio.add("shoot","shooting_gun.wav");
    audio.add("startup","game_starting.wav");
    audio.add("win","victory.wav");

    audio.play("startup");

    //Terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    //render loop in a separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move ||{
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);

        loop{
            let curr_frame = match render_rx.recv(){
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });
    
    let mut player = Player::new();
    let mut instant = Instant::now();
    let mut invaders = Invaders::new();

    //Game Loop
    'gameloop: loop {
        //per-frame init
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = frame::new_frame();

        //Input
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()?{
                match key_event.code {
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        if player.shoot(){
                            audio.play("shoot");
                        }
                    },
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    _ => {}
                }
            }
        }

        //updates
        player.update(delta);

        if invaders.update(delta){
            audio.play("move");
        }

        if player.detect_hits(&mut invaders){
            audio.play("explode")
        }

        //draw & render
        let drawables : Vec<&dyn Drawable> = vec![&player, &invaders];

        for drawable in drawables {
            drawable.draw(&mut curr_frame)
        }

        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));

        //win or lose
        if invaders.all_killed(){
            audio.play("win");
            break 'gameloop;
        }

        if invaders.reached_bottom(){
            audio.play("lose");
            break 'gameloop;
        }
    }

    //Cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    
    Ok(())
}
