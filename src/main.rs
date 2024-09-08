use std::{
    borrow::Cow,
    net::SocketAddr,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::error::SbResult as Result;
use chrono::NaiveDateTime;
use clap::{Parser, Subcommand};
use config::Config;
use error::Error;
use scatterbrain::{
    crypto::{B64SessionState, SessionState},
    mdns::{HostRecord, ServiceScanner},
    response::{Message, PrettyPrint},
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use uuid::Uuid;

mod config;
pub use sbcli::error;

#[derive(Debug, Subcommand, Clone)]
enum ConnectCommand {
    GetIdentity {
        id: Option<Uuid>,
    },
    ImportIdentity {
        id: Option<Uuid>,
    },
    GetMessages {
        application: String,
        limit: Option<i32>,
        from: Option<NaiveDateTime>,
        to: Option<NaiveDateTime>,
    },
    SendMessage {
        application: String,
        text: String,
    },
    GetEvents {
        #[clap(long, short, action, default_value_t = false)]
        block: bool,
        number: Option<u32>,
    },
}

#[derive(Debug, Subcommand)]
enum Mode {
    Scan,
    Connect {
        host: SocketAddr,
        #[command(subcommand)]
        cmd: ConnectCommand,
    },
    Pair {
        host: SocketAddr,
    },
}

#[derive(Debug, Parser)]
struct Args {
    /// Mode to run
    #[command(subcommand)]
    mode: Mode,
    #[clap(long, short, default_value_t = {"test".to_owned()} )]
    app_name: String,
    #[clap(long, short)]
    config_dir: Option<PathBuf>,
}

struct App {
    args: Args,
    session: Config<SessionState, B64SessionState>,
}

impl App {
    async fn new() -> Result<Self> {
        let args = Args::parse();
        let session = if let Some(ref config_dir) = args.config_dir {
            log::info!("using config directory {}", config_dir.display());
            Config::try_load_dir(Cow::Borrowed("sbcli"), config_dir.clone()).await?
        } else {
            Config::try_load(Cow::Borrowed("sbcli")).await?
        };
        let s = Self { args, session };
        Ok(s)
    }

    async fn scan(&self) -> Result<()> {
        let scanner = ServiceScanner::new();
        scanner
            .mdns_scan(|r| async move {
                for (name, record) in r.iter() {
                    println!("{}:", name);
                    for ip in record.addr.iter() {
                        let v = SocketAddr::new(ip.clone(), record.port);
                        println!("\t{}", v);
                    }
                }
                Ok(())
            })
            .await?;
        Ok(())
    }

    async fn run(mut self) -> Result<()> {
        match self.args.mode {
            Mode::Scan => self.scan().await?,
            Mode::Connect { host, cmd } => {
                let host: HostRecord = host.into();
                println!("starting connect with {}", host.name);
                let stream = host.connect().await?;
                let mut stream = stream
                    .key_exchange(self.session.config)
                    .await?
                    .ok_or_else(|| Error::NotPaired)?;

                match cmd {
                    ConnectCommand::GetIdentity { id } => {
                        let id = stream.get_identity(id).await?;
                        println!("{}", id.print_output()?);
                    }
                    ConnectCommand::ImportIdentity { id } => {
                        let state = stream.initiate_identity_import(id).await?;
                        println!("{:?}", state);
                    }
                    ConnectCommand::GetMessages {
                        application,
                        limit,
                        from,
                        to,
                    } => {
                        let m = if let (Some(from), Some(to)) = (from, to) {
                            stream
                                .get_messages_recieve_date(application, limit, from, to)
                                .await?
                        } else {
                            stream.get_messages(application, limit).await?
                        };
                        println!("{}", m.print_output()?);
                    }
                    ConnectCommand::SendMessage { application, text } => {
                        let message = Message {
                            from_fingerprint: None,
                            to_fingerprint: None,
                            application,
                            is_file: false,
                            extension: "ext".to_owned(),
                            send_date: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs() as i64,
                            file_name: "nothing".to_owned(),
                            receive_date: 0,
                            id: None,
                            mime: "application/text".to_owned(),
                            body: text.into_bytes(),
                        };

                        stream.send_messages(vec![message], None).await?;
                        println!("Sent message successfully");
                    }
                    ConnectCommand::GetEvents { block, number } => {
                        let events = stream.get_events(block, number).await?;
                        println!("{:?}", events);
                    }
                }
            }
            Mode::Pair { host } => {
                let host: HostRecord = host.into();
                println!("starting pairing with {}", host.name);
                let stream = host.connect().await?;
                let session = stream
                    .pair(
                        self.session.config,
                        self.args.app_name,
                        |words| async move {
                            let text = format!(
                                "got key\n {}\nDo the words match on the phone? (y/n)> ",
                                words
                            );

                            let stdin = tokio::io::stdin();
                            let mut stdout = tokio::io::stdout();
                            stdout.write(text.as_bytes()).await?;
                            stdout.flush().await?;
                            let line = BufReader::new(stdin).lines().next_line().await?;
                            if let Some("y") | Some("yes") = line.as_deref() {
                                Ok(true)
                            } else {
                                Ok(false)
                            }
                        },
                    )
                    .await?;

                println!("Pairing succesful!");

                self.session.config = session.state;
                self.session.persist().await?;
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let app = App::new().await.unwrap();
    app.run().await
}
