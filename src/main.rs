use std::error::Error;
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

    //Cleanup
    audio.wait();
    Ok(())
}
