use std::{sync::{Mutex, mpsc::{Sender, channel, Receiver}}, io::BufReader};
use rodio::{Sink, OutputStream};

pub struct MusicPlayer {
    handle: Mutex<Sender<PlayCommand>>
}

impl MusicPlayer {
    pub fn new() -> Self {  
        let (tx, rx): (Sender<PlayCommand>, Receiver<PlayCommand>) = channel();
        std::thread::spawn(move || {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            loop {
                if let Ok(command) = rx.recv() {
                    command.run(&sink)
                }
            }
        });

        Self { handle: Mutex::new(tx) }
    }

    fn send(&self, command: PlayCommand) {
        if let Ok(h) = self.handle.lock() {
            h.send(command).unwrap();
        }
    }

    pub fn add_to_queue(&self, path: &std::path::PathBuf) {
        self.send(PlayCommand::AddToQueue(path.to_path_buf()));
    }

    pub fn play(&self) {
        // if self.sink.is_paused() {self.sink.play()} else {self.sink.pause()};
    }

    pub fn pause(&self) {

    }


}

// create enum for different commands to thread
enum PlayCommand {
    Play,
    Pause,
    AddToQueue(std::path::PathBuf),
    SetVolume(f32),
}

impl PlayCommand {
    fn run(&self, sink: &Sink) {
        match self {
            PlayCommand::Play => play(sink),
            PlayCommand::Pause => pause(sink),
            PlayCommand::AddToQueue(path) => add_to_queue(sink, path),
            PlayCommand::SetVolume(value) => set_volume(sink, *value),
        }
    }
}

// create enum method that matches enums to calls MusicPlayer methods passing sink and sending the method using a universal Send method



fn add_to_queue(sink: &Sink, path: &std::path::PathBuf) {
    let file = std::fs::File::open(path).unwrap();
    sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
}

fn play(sink: &Sink) {
    sink.play();
}

fn pause(sink: &Sink) {
    sink.pause();
}

fn set_volume(sink: &Sink, value: f32) {
    sink.set_volume(value);
}