use std::{error::Error, io, time::Duration, thread, sync::mpsc};
use crossterm::{terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand, cursor::{Hide, Show}, event::{self, KeyCode, Event}};
use invaders::{frame, render};
use rusty_audio::Audio;

fn main() -> Result<(), Box<dyn Error>>{
    let mut audio = Audio::new();
    audio.add("explode","explosion.wav");
    audio.add("lose","game_over.wav");
    audio.add("move","monster_laugh.wav");
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

    //Game Loop
    'gameloop: loop {
        //per-frame init
        let curr_frame = frame::new_frame();

        //Input
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()?{
                match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    _ => {}
                }
            }
        }

        //draw & render
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));
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
