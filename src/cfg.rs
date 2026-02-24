use crate::prelude::*;

#[derive(Parser)]
#[command(author, version, about)]
pub struct ArgsConfig {
    #[arg(short, long, default_value_t = String::from("s."))]
    prefix: String,

    #[arg(short, long, default_value_t = String::from("kun-bot"))]
    title: String,

    #[arg(short, long, default_value = "whitelist.txt")]
    wl_path: PathBuf,

    #[arg(required = true, num_args = 1..)]
    paths: Vec<PathBuf>,

    #[arg(short, long)]
    config_path: Option<PathBuf>,

    #[arg(required = true, num_args = 1.., last = true)]
    admins: Vec<UserId>,
}

pub struct Data {
    pub prefix: String,
    pub admins: Vec<UserId>,
    pub whitelist: Mutex<Whitelist>,
    pub images: Vec<CreateMessage>,
    pub links: Arc<Mutex<HashMap<MessageId, MessageId>>>,
    pub stf: Spotify,
}

impl Data {
    pub async fn new() -> Result<Self> {
        let args = ArgsConfig::parse();

        let cred = Credentials::new(args.config_path)?;
        let stf = Spotify::new(cred).await?;

        let whitelist = {
            let mut data = Vec::new();

            if let Ok(mut f) = File::open(&args.wl_path).await {
                let mut buf = String::new();
                f.read_to_string(&mut buf).await?;

                data.extend(
                    buf.split_ascii_whitespace()
                        .map(try_into_guild_id)
                        .collect::<Result<Vec<GuildId>>>()?,
                );
            }
            Whitelist::new(data, args.wl_path)
        };

        let prefix = args.prefix;
        let admins = args.admins;
        let whitelist = Mutex::new(whitelist);
        let images = get_images(&args.title, args.paths).await?;
        let links = Default::default();

        let cfg = Self {
            prefix,
            admins,
            whitelist,
            images,
            links,
            stf,
        };
        Ok(cfg)
    }
}
