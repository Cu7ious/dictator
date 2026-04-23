pub fn play_start(path: &str) {
    play(path);
}

pub fn play_stop(path: &str) {
    play(path);
}

fn play(path: &str) {
    std::process::Command::new("afplay").arg(path).spawn().ok();
}
