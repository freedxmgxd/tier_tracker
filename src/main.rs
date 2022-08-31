use std::env;

use dotenvy::dotenv;

use tier_tracker::{
    clear_current_role,
    lol::{get_summoner_id, get_summoner_rank},
    update_role,
};
use serenity::{
    async_trait,
    model::{
        channel::Message,
        gateway::{Presence, Ready}
    },
    prelude::*,
};
use sqlx::{mysql::MySqlPoolOptions, query, MySqlPool};

struct Bot {
    database: MySqlPool,
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        let (command, args) = msg
            .content
            .split_once(" ")
            .unwrap_or((msg.content.as_str(), ""));
        let author_id = msg.author.id;

        match command {
            "!ping" => {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                    println!("Error sending message: {:?}", why);
                }
            }
            "!track" => {
                let summoner_id = get_summoner_id(&args.to_string())
                    .await
                    .expect("Error getting summoner id");
                let summoner_rank = get_summoner_rank(&summoner_id)
                    .await
                    .expect("Error getting summoner rank");

                match query!(
                    "SELECT * FROM summoners WHERE discord_id = ?",
                    author_id.to_string()
                )
                .fetch_one(&self.database)
                .await
                {
                    Ok(_) => {
                        query!(
                            "UPDATE summoners SET summoner_id = ?, rank = ? WHERE discord_id = ?",
                            summoner_id,
                            summoner_rank,
                            author_id.to_string()
                        )
                        .execute(&self.database)
                        .await
                        .unwrap();
                    }
                    Err(sqlx::Error::RowNotFound) => {
                        query!(
                        "INSERT INTO summoners (discord_id, summoner_id, rank) VALUES (?, ?, ?)",
                        author_id.to_string(),
                        summoner_id,
                        summoner_rank
                    )
                        .execute(&self.database)
                        .await
                        .unwrap();
                    }
                    Err(_) => {
                        todo!(); // TODO: Handle error
                    }
                };

                let guild = msg
                    .guild_id
                    .unwrap()
                    .to_partial_guild(&ctx.http)
                    .await
                    .unwrap();

                clear_current_role(&ctx.http, &guild, author_id).await;

                update_role(&ctx.http, &guild, author_id, summoner_rank.as_str()).await;
            }
            "!untrack" => {
                let guild = msg
                    .guild_id
                    .unwrap()
                    .to_partial_guild(&ctx.http)
                    .await
                    .unwrap();

                query!(
                    "DELETE FROM summoners WHERE discord_id = ?",
                    author_id.to_string()
                )
                .execute(&self.database)
                .await
                .unwrap();

                clear_current_role(&ctx.http, &guild, author_id).await;
            }
            _ => {}
        }
    }
    async fn presence_update(&self, _ctx: Context, new_data: Presence) {

        let user_id = new_data.user.id;

        let row = query!(
            "SELECT * FROM summoners WHERE discord_id = ?",
            user_id.to_string()
        )
        .fetch_one(&self.database)
        .await
        .unwrap();

        let new_rank = get_summoner_rank(&row.summoner_id).await;
        match new_rank {
            Ok(new_rank) => {
                if new_rank != row.rank {
                    query!(
                        "UPDATE summoners SET rank = ? WHERE discord_id = ?",
                        new_rank,
                        user_id.to_string()
                    )
                    .execute(&self.database)
                    .await
                    .unwrap();
                }
            }
            Err(_) => {
                todo!(); // TODO: Handle error
            }
        }
    }
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let db_url = env::var("DATABASE_URL").expect("Expected a database url in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_PRESENCES;

    let database = MySqlPoolOptions::new().connect(&db_url).await.unwrap();

    let bot = Bot { database };

    let mut client = Client::builder(&token, intents)
        .event_handler(bot)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
