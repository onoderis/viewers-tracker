use std::collections::HashSet;
use std::error::Error;
use std::sync::mpsc;

use chrono::Local;
use clap::Parser;
use twitch_api2::tmi::TmiClient;
use twitch_api2::types::Nickname;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let channel_name = cli.channel.as_str();

    let (sender, receiver) = mpsc::channel::<Command>();
    sender.send(Command::Init).unwrap();

    let exit_sender = sender.clone();
    ctrlc::set_handler(move || {
        exit_sender.send(Command::Exit)
            .expect("Failed to send exit message");
    })
        .expect("Error setting Ctrl-C handler");

    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(30));
            sender.send(Command::Update).unwrap();
        }
    });


    let mut channel_spy = ChannelSpy::new(channel_name.to_string());

    loop {
        match receiver.recv().unwrap() {
            Command::Init => {
                let diff = channel_spy.update_viewers().await?;
                let now = Local::now();
                for v in diff.new_viewers.iter() {
                    println!("{} was here: {}", now.format(&DATE_FORMAT), v);
                }
            }
            Command::Update => {
                let diff = channel_spy.update_viewers().await?;
                let now = Local::now();
                for v in diff.new_viewers.iter() {
                    println!("{} joined:   {}", now.format(&DATE_FORMAT), v);
                }
                for v in diff.left_viewers.iter() {
                    println!("{} left:     {}", now.format(&DATE_FORMAT), v);
                }
            }
            Command::Exit => {
                println!("Exiting...");
                break;
            }
        }
    }
    return Ok(());
}

const DATE_FORMAT: &str = "%F %R";

struct ChannelSpy<'a> {
    channel_nickname: Nickname,
    tmi_client: TmiClient<'a, reqwest::Client>,
    prev_viewers: HashSet<String>,
}

impl ChannelSpy<'_> {
    fn new(channel_name: String) -> ChannelSpy<'static> {
        let http_client = reqwest::Client::new();
        let tmi_client: TmiClient<reqwest::Client> = TmiClient::with_client(http_client.into());
        ChannelSpy {
            channel_nickname: channel_name.into(),
            tmi_client,
            prev_viewers: HashSet::new(),
        }
    }

    async fn update_viewers(&mut self) -> Result<ViewersDiff, Box<dyn Error>> {
        let get_chatters = self.tmi_client.get_chatters(&self.channel_nickname).await?;

        let mut viewers = HashSet::new();
        Self::add_nicknames(&mut viewers, &get_chatters.chatters.broadcaster);
        Self::add_nicknames(&mut viewers, &get_chatters.chatters.viewers);
        Self::add_nicknames(&mut viewers, &get_chatters.chatters.vips);
        Self::add_nicknames(&mut viewers, &get_chatters.chatters.moderators);
        Self::add_nicknames(&mut viewers, &get_chatters.chatters.staff);
        Self::add_nicknames(&mut viewers, &get_chatters.chatters.admins);
        Self::add_nicknames(&mut viewers, &get_chatters.chatters.global_mods);

        let new_viewers = &viewers - &self.prev_viewers;
        let left_viewers = &self.prev_viewers - &viewers;

        self.prev_viewers = viewers;

        Ok(ViewersDiff { new_viewers, left_viewers })
    }

    fn add_nicknames(viewers: &mut HashSet<String>, nicknames: &Vec<Nickname>) {
        for n in nicknames.iter() {
            viewers.insert(n.clone().into_string());
        }
    }
}


#[derive(Parser, Debug)]
struct Cli {
    #[arg(long)]
    channel: String,
}

enum Command {
    Init,
    Update,
    Exit,
}

struct ViewersDiff {
    new_viewers: HashSet<String>,
    left_viewers: HashSet<String>,
}
