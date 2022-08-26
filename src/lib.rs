pub mod lol;

use serenity::{
    http::{CacheHttp, Http},
    model::{id::UserId, prelude::PartialGuild},
};

pub async fn clear_current_role(
    http: &(impl AsRef<Http> + CacheHttp),
    guild: &PartialGuild,
    user_id: UserId,
) -> () {
    let ranks_lol = vec![
        "UNRANKED",
        "IRON",
        "BRONZE",
        "SILVER",
        "GOLD",
        "PLATINUM",
        "DIAMOND",
        "MASTER",
        "GRANDMASTER",
        "CHALLENGER",
    ];

    for rank in ranks_lol {
        let role = guild.role_by_name(rank);
        match role {
            Some(role) => {
                let role_id = role.id;
                let mut member = guild.member(http, user_id).await.unwrap();

                member
                    .remove_role(http, role_id)
                    .await
                    .expect("Failed to remove role");
            }
            None => {
                continue;
            }
        }
    }
}

pub async fn update_role(
    http: &(impl AsRef<Http> + CacheHttp),
    guild: &PartialGuild,
    user_id: UserId,
    rank: &str,
) -> () {
    let mut member = guild.member(http, user_id).await.unwrap();

    let guild_role = guild.role_by_name(rank);

    match guild_role {
        Some(role) => {
            member
                .add_role(http, role.id)
                .await
                .expect("Failed to add role");
        }
        None => {
            let role = guild
                .create_role(http, |r| r.hoist(true).name(rank))
                .await;

            member
                .add_role(http, role.unwrap().id)
                .await
                .expect("Failed to add role");
        }
    }
}
