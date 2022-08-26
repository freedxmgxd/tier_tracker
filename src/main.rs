use std::env;

use dotenv::dotenv;

use elo_tracker::{
    clear_current_role,
    lol::{get_summoner_id, get_summoner_rank}, update_role,
};
use serenity::{
    async_trait,
    model::{
        channel::Message,
        gateway::{Presence, Ready},
        prelude::ChannelId,
    },
    prelude::*,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let (command, args) = msg
            .content
            .split_once(" ")
            .unwrap_or((msg.content.as_str(), ""));

        match command {
            "!ping" => {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                    println!("Error sending message: {:?}", why);
                }
            }
            "!track" => {
                let summoner_id = get_summoner_id(String::from(args)).await.unwrap();
                let summoner_rank = get_summoner_rank(summoner_id).await.unwrap();

                let guild = msg
                    .guild_id
                    .unwrap()
                    .to_partial_guild(&ctx.http)
                    .await
                    .unwrap();

                clear_current_role(&ctx.http, &guild, msg.author.id).await;

                update_role(&ctx.http, &guild, msg.author.id, summoner_rank.as_str()).await;
            }
            "!untrack" => {
                let guild = msg
                    .guild_id
                    .unwrap()
                    .to_partial_guild(&ctx.http)
                    .await
                    .unwrap();

                clear_current_role(&ctx.http, &guild, msg.author.id).await;
            }
            _ => {}
        }
    }
    async fn presence_update(&self, _ctx: Context, new_data: Presence) {
        let channel: ChannelId = ChannelId(env::var("CHANNEL_ID").unwrap().parse().unwrap());

        // let member = new_data.guild_id.unwrap().member(&_ctx.http, new_data.user.id).await.unwrap();

        let nick = new_data.user.name.unwrap_or("Slug!".to_string());

        let response = "Hi ".to_string() + &nick + "!";

        channel.say(&_ctx.http, response).await.expect("Deu ruim");

        println!(
            "Hello from the presence_update event! id: {} name: {}",
            new_data.user.id, nick
        );
    }
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_PRESENCES;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
