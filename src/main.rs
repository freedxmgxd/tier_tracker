use std::{env, io};

// use dotenvy::dotenv;

use firestore::{path, paths, FirestoreDb};
use serde::{Deserialize, Serialize};
use serenity::{
    async_trait,
    model::{
        channel::Message,
        gateway::{Presence, Ready},
    },
    prelude::*,
    utils::{EmbedMessageBuilding, MessageBuilder},
};
use tier_tracker::{
    clear_current_role,
    lol::{get_summoner_id, get_summoner_rank},
    update_role,
};
use tokio::net::TcpListener;

struct Bot {
    database: FirestoreDb,
}

const GUILDS_COLLECTION: &str = "guilds";
const USERS_COLLECTION: &str = "users";
#[derive(Debug, Clone, Deserialize, Serialize)]
struct Guild {
    id: u64,
    name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct User {
    disc_id: u64,
    name: String,
    summoner_id: String,
    rank: String,
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        let (command, args) = msg
            .content
            .split_once(" ")
            .unwrap_or((msg.content.as_str(), ""));
        let author_id = msg.author.id;

        let db = &self.database;

        match command {
            "!ping" => {
                let guild = msg
                    .guild_id
                    .unwrap()
                    .to_partial_guild(&ctx.http)
                    .await
                    .unwrap();

                let guild_data = Guild {
                    id: *(&guild).id.as_u64(),
                    name: (&guild).name.clone(),
                };

                let guild_doc: Result<Guild, firestore::errors::FirestoreError> = db
                    .get_obj(GUILDS_COLLECTION, &guild_data.id.to_string())
                    .await;

                match guild_doc {
                    Ok(doc) => {
                        println!("Found guild: {:?}", doc);
                    }
                    Err(firestore::errors::FirestoreError::DataNotFoundError(_)) => {
                        println!("Guild not found, creating new one");
                        println!("Guild: {:?}", guild_data);
                        db.create_obj(GUILDS_COLLECTION, &guild_data.id.to_string(), &guild_data)
                            .await
                            .unwrap();
                    }
                    Err(err) => {
                        println!("Error: {:?}", err);
                    }
                };

                let response = MessageBuilder::new()
                    .push("Pong!")
                    .push_line("")
                    .push_bold_safe("Bot created by: ")
                    .push_named_link("Freedxm_Gxd", "https://github.com/freedxmgxd")
                    .build();

                if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                    println!("Error sending message: {:?}", why);
                }
            }
            "!track" => {
                let summoner_id = get_summoner_id(&args.to_string())
                    .await
                    .expect("Error getting summoner id");
                let summoner_elo = get_summoner_rank(&summoner_id)
                    .await
                    .expect("Error getting summoner rank");

                let guild = msg
                    .guild_id
                    .unwrap()
                    .to_partial_guild(&ctx.http)
                    .await
                    .unwrap();

                let user_data = User {
                    disc_id: *(&author_id).as_u64(),
                    name: msg.author.name,
                    summoner_id,
                    rank: (&summoner_elo).to_string(),
                };

                let users_path = format!(
                    "{}/{}/{}",
                    db.get_documents_path(),
                    GUILDS_COLLECTION,
                    &guild.id.as_u64()
                );

                // Using update instead of create because we want to update the user if they already exist
                let user_doc: Result<User, firestore::errors::FirestoreError> = db
                    .update_obj_at(
                        &users_path,
                        USERS_COLLECTION,
                        user_data.disc_id.to_string(),
                        &User {
                            ..user_data.clone()
                        },
                        Some(paths!(User::{name, summoner_id, rank, disc_id})),
                    )
                    .await;

                match user_doc {
                    Ok(_) => {}
                    Err(err) => {
                        println!("Error: {:?}", err);
                    }
                };

                update_role(&ctx.http, &guild, author_id, summoner_elo.as_str()).await;
            }
            "!untrack" => {
                let guild = msg
                    .guild_id
                    .unwrap()
                    .to_partial_guild(&ctx.http)
                    .await
                    .unwrap();

                let users_collection = format!(
                    "{}/{}/{}",
                    db.get_documents_path(),
                    GUILDS_COLLECTION,
                    &guild.id.as_u64()
                );

                db.delete_by_id_at(
                    &users_collection,
                    USERS_COLLECTION,
                    author_id.as_u64().to_string(),
                )
                .await
                .unwrap();

                clear_current_role(&ctx.http, &guild, author_id).await;
            }
            _ => {}
        }
    }
    async fn presence_update(&self, ctx: Context, new_data: Presence) {
        let db = &self.database;

        let user_id = new_data.user.id;

        let guild = new_data
            .guild_id
            .unwrap()
            .to_partial_guild(&ctx.http)
            .await
            .unwrap();

        let users_path = format!(
            "{}/{}/{}",
            db.get_documents_path(),
            GUILDS_COLLECTION,
            &guild.id.as_u64()
        );

        let user_doc: Result<User, firestore::errors::FirestoreError> = db
            .get_obj_at(&users_path, USERS_COLLECTION, user_id.as_u64().to_string())
            .await;

        match user_doc {
            Ok(user) => {
                let new_elo = get_summoner_rank(&user.summoner_id)
                    .await
                    .expect("Error getting summoner rank");

                if new_elo != user.rank {
                    db.update_obj_at(
                        &users_path,
                        USERS_COLLECTION,
                        user_id.as_u64().to_string(),
                        &User { ..user.clone() },
                        Some(paths!(User::{rank})),
                    )
                    .await
                    .unwrap();

                    update_role(&ctx.http, &guild, user_id, new_elo.as_str()).await;
                    println!("Updated user: {:?}", user);
                }
            }
            Err(firestore::errors::FirestoreError::DataNotFoundError(_)) => {}
            Err(err) => {
                println!("Error: {:?}", err);
            }
        };
    }
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let project_id = env::var("PROJECT_ID").expect("Expected a database url in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_PRESENCES;

    let database = FirestoreDb::new(project_id).await.unwrap();

    let bot = Bot { database };

    let mut client = Client::builder(&token, intents)
        .event_handler(bot)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}


// async fn main() -> io::Result<()> {
//     let port: u16 = env::var("PORT")
//         .unwrap_or_else(|_| "3000".to_string())
//         .parse()
//         .expect("PORT must be a number");

//     let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

//     loop {
//         let (socket, _) = listener.accept().await?;

//         process_socket(socket).await;
//     }
// }
