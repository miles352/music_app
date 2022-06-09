
use std::{sync::{Mutex, mpsc::{Receiver, Sender, self}, Arc, Once, RwLock}, fs::{DirEntry, ReadDir}, path::PathBuf, ffi::OsStr};
use egui::{Rect, Pos2};
use egui_extras::{StripBuilder, Size, TableBuilder};
use lofty::{Accessor, AudioFile};

use crate::audio::*;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]


//     #[serde(skip)] to unsave vars
pub struct MusicApp<'a> {
    // Example stuff:
    label: String,
    value: f32,
    #[serde(skip)]
    watch_folder: Option<PathBuf>,
    #[serde(skip)]
    music_player: MusicPlayer,
    #[serde(skip)]
    songs: Arc<RwLock<Vec<Song<'a>>>>,
    #[serde(skip)]
    t_path_handle: Option<Mutex<Sender<PathBuf>>>,
    #[serde(skip)]
    t_index_handle: Option<Sender<usize>>
    
}

impl Default for MusicApp<'_> {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            music_player: MusicPlayer::new(),
            watch_folder: None,
            songs: Arc::new(RwLock::new(Vec::new())),
            t_path_handle: None,
            t_index_handle: None
        }
    }
}

fn generate_songs_from_folder(folder: &PathBuf) ->  Option<Vec<DirEntry>>{
    // let files = std::fs::read_dir(folder);
    let files = match std::fs::read_dir(folder) {
        Ok(file) => file,
        Err(_err) => return None
    };

    let mut result: Vec<DirEntry> = Vec::new();

    for file in files {
        let current_path = file.as_ref().unwrap().path();
        if current_path.is_dir() {
            if let Some(songs) = &mut generate_songs_from_folder(&current_path) {
                result.append(songs);
            }
        } else {
            result.push(file.unwrap());
        }   
    }
    return Some(result)
}

fn get_song_tags<'a>(file_path: &'a PathBuf, file_name: &'a OsStr) -> Option<&'a TagData> {
    let tagged_file = lofty::read_from_path(file_path, false); //try changing to true for druation to work???
    if tagged_file.is_err() {
        println!("{}", file_name.to_string_lossy().to_string());
        return None
    };
    let tagged_file = tagged_file.unwrap();
    let tag_data: Option<TagData> = {
        if let Some(tag) = tagged_file.primary_tag() {
            Some(TagData {
                title: tag.title().unwrap_or(&file_name.to_string_lossy()).to_string(),
                artist: tag.artist().unwrap_or("Unknown Artist").to_string(),
                album: tag.album().unwrap_or("Unknown Album").to_string(),
                duration: tagged_file.properties().duration().as_secs() as u32
            })  
        } else {
            None
        }
    };
    if tag_data.is_some() {
        return Some(&tag_data.unwrap())
        
    } else {
        return Some(&TagData { 
            title: file_name.to_string_lossy().to_string(),
            artist: "Unknown Artist".to_string(),
            album: "Unknown Album".to_string(),
            duration: tagged_file.properties().duration().as_secs() as u32
        })
    };
}

impl MusicApp<'_> {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_visuals(egui::style::Visuals::light());
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        
        // add font
        Self {
            ..Default::default()
        }
        
    }

    

    fn create_file_thread(&mut self) {
        let (t_path, r_path) = mpsc::channel();
        let (t_index, r_index): (Sender<usize>, Receiver<usize>) = mpsc::channel();
        // let (tSongs, rSongs) = mpsc::channel();
        let songs = Arc::clone(&self.songs);
        std::thread::spawn(move || {
            loop {
                // std::thread::sleep(std::time::Duration::from_millis(1000));

                if let Ok(index) = r_index.try_recv() {
                    let mut song = &mut songs.write().unwrap()[index];
                    song.tag_data = get_song_tags(&song.path, song.path.file_stem().unwrap());
                }

                let file_entries = generate_songs_from_folder(&r_path.recv().unwrap());
                if file_entries.is_none() {continue};
                // for each one grab song shit and create Song objects 
                // let mut songs_temp = songs.lock().unwrap();
                // songs_temp.clear();
                let mut songs_temp = Vec::new();
                for file in file_entries.unwrap() {
                    
                    let mut found = &false;
                    let file_path = file.path();
                    let file_extension = file_path.extension();
                    let file_name = file_path.file_stem().unwrap();
                    if file_extension.is_none() {continue}; 
                    for &extension in ["mp3", "m4a", "flac", "wav"].iter() {
                        if extension == file_extension.unwrap() {
                            found = &true;
                            break;
                        }
                    };
                    if !found {continue};
                    for song in songs.read().unwrap().iter() {
                        if song.path == file_path {
                            songs_temp.push(Song {
                                path: file.path(),
                                tag_data: song.tag_data
                              });
                        } else {
                            songs_temp.push(Song {
                                path: file.path(),
                                tag_data: None
                              });
                        }
                    }
                    
                    
                    // let temp_song = get_song_tags(&file_path, file_name);
                    // if temp_song.is_none() { continue }; // unknown error
                    // songs_temp.push(temp_song.unwrap());
                    
                };    
                *songs.write().unwrap() = songs_temp;
                     
            };
        });
        self.t_path_handle = Some(Mutex::new(t_path));
        self.t_index_handle = Some(t_index);

    }

    
}


struct Song<'a> {
    path: std::path::PathBuf,
    tag_data: Option<&'a TagData>
    // file_extension: String  
}

struct TagData {
    title: String,
    artist: String,
    album: String,
    duration: u32, 
}

static INIT: Once = Once::new();

impl eframe::App for MusicApp<'_> {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {


        INIT.call_once(|| {
            self.create_file_thread();
        });


        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });

            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            
            if ui.button("Open file…").clicked() {
                if let Some(path) = &rfd::FileDialog::new().set_directory("%userprofile/Music").pick_folder() {
                    self.watch_folder = Some(path.to_path_buf());
                }
            }
            
            if self.t_path_handle.is_some() && self.watch_folder.is_some() {
                // send watch_folder to transfer channel of thread
                // self.t_path_handle.as_ref().unwrap().lock().unwrap().send(self.watch_folder.as_ref().unwrap().to_path_buf()).expect("oops");
                if let Ok(lock) = self.t_path_handle.as_ref().unwrap().try_lock() {
                    lock.send(self.watch_folder.as_ref().unwrap().to_path_buf()).expect("aaaa");
                }
            }
            // use show_rows to make faster           
            egui::ScrollArea::vertical().auto_shrink([false, true]).show(ui, |ui| {
                
                
                let songs = &*self.songs.read().unwrap();
                if songs.len() == 0 {
                    ui.label("You must select a folder to play songs!");
                    return;
                }         
                let text_height = egui::TextStyle::Heading.resolve(ui.style()).size;
                let mouse_pos = ctx.input().pointer.hover_pos();
                TableBuilder::new(ui) 
                    .striped(true)
                    .column(Size::relative(0.025))
                    .column(Size::remainder())
                    .column(Size::relative(0.2))
                    .column(Size::relative(0.2))
                    .column(Size::relative(0.05))
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("Title");
                        });
                        header.col(|ui| {
                            ui.heading("Artist");
                        });
                        header.col(|ui| {
                            ui.heading("Album");
                        });
                        header.col(|ui| {
                            ui.heading("Duration");
                        });
                    })
                    .body(|body| {
                        let row_width:f32 = body.widths().iter().sum::<f32>();
                        body.rows(text_height, songs.len(), |row_index, mut row| {
                            
                            row.col(|ui| {
                            let clip_rect = ui.clip_rect();
                            let row_rect = Rect { 
                                min: Pos2 {
                                    x: clip_rect.min.x, // bottom left of first col
                                    y: clip_rect.min.y // bottom left of first col
                                },
                                max: Pos2 { 
                                    x: clip_rect.min.x + row_width, // bottom left of first col + width of row = row bottom left
                                    y: clip_rect.max.y // top right y coord of column is same as row
                                }
                            };
                            if let Some(pos) = mouse_pos {
                                if row_rect.contains(pos) { 
                                    //                play button
                                    if ui.button("\u{25B6}").clicked() {
                                        self.music_player.add_to_queue(&songs[row_index].path);
                                    }
                                }
                            }                 
                            });
                            // songs becomes Vec<Option<Song>>
                            // check if Song is Some
                            // if some draw using values
                            // if not some, request data through channel and draw as blank for this frame
                            // this way it loads per item as scrolled through

                            if songs[row_index].tag_data.is_none() {
                                // if self.t_index_handle.is_some() {
                                //     if let Ok(lock) = self.t_index_handle.as_ref().unwrap().try_lock() {
                                //         lock.send(self.watch_folder.as_ref().unwrap().to_path_buf()).expect("aaaa");
                                //     }
                                // }
                                if self.t_index_handle.is_some() {
                                    self.t_index_handle.as_ref().unwrap().send(row_index);
                                }
                                row.col(|ui| {
                                    ui.heading(" ");
                                });
                                row.col(|ui| {
                                    ui.heading(" ");
                                });
                                row.col(|ui| {
                                    ui.heading(" ");
                                });
                                row.col(|ui| {
                                    ui.heading(" ");
                                    //format!("{:02}:{:02}", (&songs[row_index].duration - (&songs[row_index].duration % 60)) / 60, &songs[row_index].duration % 60)
                                });
                            } else {
                                row.col(|ui| {
                                    ui.heading(&songs[row_index].tag_data.as_ref().unwrap().title);
                                });
                                row.col(|ui| {
                                    ui.heading(&songs[row_index].tag_data.as_ref().unwrap().artist);
                                });
                                row.col(|ui| {
                                    ui.heading(&songs[row_index].tag_data.as_ref().unwrap().album);
                                });
                                row.col(|ui| {
                                    ui.heading(&songs[row_index].tag_data.as_ref().unwrap().duration.to_string());
                                    //format!("{:02}:{:02}", (&songs[row_index].duration - (&songs[row_index].duration % 60)) / 60, &songs[row_index].duration % 60)
                                });
                            }
                            
                            
                        })
                    }); 
            });
        });   
    }
}

