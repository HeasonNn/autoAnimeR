use crate::models::anime_list::*;
use crate::schema::anime_list::dsl::*;
use anyhow::Result;
use diesel::dsl::insert_into;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::{delete, RunQueryDsl};

// insert single data into anime_list
#[allow(dead_code)]
pub async fn add(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    item: AnimeListJson,
) -> Result<AnimeList, diesel::result::Error> {
    match anime_list
        .filter(mikan_id.eq(&item.mikan_id))
        .first::<AnimeList>(db_connection)
    {
        Ok(result) => Ok(result),
        Err(_) => {
            let new_anime_list = PostAnimeList {
                mikan_id: &item.mikan_id,
                anime_name: &item.anime_name,
                img_url: &item.img_url,
                update_day: &item.update_day,
                anime_type: &item.anime_type,
                subscribe_status: &item.subscribe_status,
                bangumi_id: &item.bangumi_id,
                bangumi_rank: &item.bangumi_rank,
                bangumi_summary: &item.bangumi_summary,
                website: &item.website,
                anime_status: &item.anime_status,
                total_episodes: &item.total_episodes,
                new_finished_episode: &item.new_finished_episode,
            };
            insert_into(anime_list)
                .values(&new_anime_list)
                .execute(db_connection)
                .expect("Error saving new anime");
            let result = anime_list.order(id.desc()).first(db_connection).unwrap();
            Ok(result)
        }
    }
}

// insert Vec<data> into anime_list
pub async fn add_vec(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    item_vec: Vec<AnimeListJson>,
) -> Result<i32, diesel::result::Error> {
    let mut sucess_num: i32 = 0;
    for item in &item_vec {
        if let Err(_) = anime_list
            .filter(mikan_id.eq(&item.mikan_id))
            .first::<AnimeList>(db_connection)
        {
            let new_anime_list = PostAnimeList {
                mikan_id: &item.mikan_id,
                anime_name: &item.anime_name,
                img_url: &item.img_url,
                update_day: &item.update_day,
                anime_type: &item.anime_type,
                subscribe_status: &item.subscribe_status,
                bangumi_id: &item.bangumi_id,
                bangumi_rank: &item.bangumi_rank,
                bangumi_summary: &item.bangumi_summary,
                website: &item.website,
                anime_status: &item.anime_status,
                total_episodes: &item.total_episodes,
                new_finished_episode: &item.new_finished_episode,
            };
            insert_into(anime_list)
                .values(&new_anime_list)
                .execute(db_connection)
                .expect("save failed");
            sucess_num += 1;
        }
    }
    Ok(sucess_num)
}

// get data by mikan_id
pub async fn get_by_mikanid(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    query_mikanid: i32,
) -> Result<AnimeList, diesel::result::Error> {
    let result: AnimeList = anime_list
        .filter(mikan_id.eq(query_mikanid))
        .first::<AnimeList>(db_connection)?;
    Ok(result)
}

pub async fn get_by_subscribestatus(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    query_subscribestatus: i32,
) -> Result<Vec<AnimeList>, diesel::result::Error> {
    let result: Vec<AnimeList> = anime_list
        .filter(subscribe_status.eq(query_subscribestatus))
        .load::<AnimeList>(db_connection)?;
    Ok(result)
}

// update subscribe_status by mikan_id
pub async fn update_subscribestatus_by_mikanid(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    query_mikanid: i32,
    update_subscribestatus: i32,
) -> Result<(), diesel::result::Error> {
    diesel::update(anime_list.filter(mikan_id.eq(query_mikanid)))
        .set(subscribe_status.eq(update_subscribestatus))
        .execute(db_connection)?;
    Ok(())
}

// update anime_status by mikan_id
pub async fn update_animestatus_by_mikanid(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    query_mikanid: i32,
    update_animestatus: i32,
) -> Result<(), diesel::result::Error> {
    diesel::update(anime_list.filter(mikan_id.eq(query_mikanid)))
        .set(anime_status.eq(update_animestatus))
        .execute(db_connection)?;
    Ok(())
}

// update bangumi info by mikan_id
pub async fn update_bangumiinfo_by_mikanid(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    query_mikanid: i32,
    item: BangumiInfoJson,
) -> Result<(), diesel::result::Error> {
    diesel::update(anime_list.filter(mikan_id.eq(query_mikanid)))
        .set((
            bangumi_id.eq(item.bangumi_id),
            bangumi_rank.eq(item.bangumi_rank),
            bangumi_summary.eq(item.bangumi_summary),
            website.eq(item.website),
            total_episodes.eq(item.total_episodes),
        ))
        .execute(db_connection)?;
    Ok(())
}

// query all data from anime_list
#[allow(dead_code)]
pub async fn get_all(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
) -> Result<Vec<AnimeList>, diesel::result::Error> {
    let result: Vec<AnimeList> = anime_list.load::<AnimeList>(db_connection)?;
    Ok(result)
}

// delete single data by mikan_id
#[allow(dead_code)]
pub async fn del_by_mikan_id(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    i: i32,
) -> Result<usize, diesel::result::Error> {
    let result = delete(anime_list.filter(mikan_id.eq(i))).execute(db_connection)?;
    Ok(result)
}

#[allow(dead_code)]
pub async fn get_mikanid_by_anime_name(
    query_anime_name: &str,
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
) -> Result<i32, diesel::result::Error> {
    let result: AnimeList = anime_list
        .filter(anime_name.eq(query_anime_name))
        .first::<AnimeList>(db_connection)?;
    Ok(result.mikan_id)
}

#[allow(dead_code)]
pub async fn search_by_anime_name(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    query_keyword: &str,
) -> Result<Vec<AnimeList>, diesel::result::Error> {
    let pattern = format!("%{}%", query_keyword);
    let results = anime_list
        .filter(anime_name.like(pattern))
        .load::<AnimeList>(db_connection)?;
    Ok(results)
}

#[allow(dead_code)]
pub async fn get_by_subscribe_and_anime_status(
    query_subscribe_status: &i32,
    query_anime_status: &i32,
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
) -> Result<Vec<AnimeList>, diesel::result::Error> {
    let result: Vec<AnimeList> = anime_list
        .filter(subscribe_status.eq(query_subscribe_status))
        .filter(anime_status.eq(query_anime_status))
        .load::<AnimeList>(db_connection)?;
    Ok(result)
}

#[allow(dead_code)]
pub async fn get_new_finished_episode_nb(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    query_mikanid: &i32,
) -> Result<i32, diesel::result::Error> {
    let result: AnimeList = anime_list
        .filter(mikan_id.eq(query_mikanid))
        .first::<AnimeList>(db_connection)?;
    Ok(result.new_finished_episode)
}

#[allow(dead_code)]
pub async fn update_new_finished_episode_nb(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    query_mikanid: &i32,
    new_finished_episode_nb: &i32,
) -> Result<(), diesel::result::Error> {
    diesel::update(anime_list.filter(mikan_id.eq(query_mikanid)))
        .set(new_finished_episode.eq(new_finished_episode_nb))
        .execute(db_connection)?;
    Ok(())
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::Pool;
    use actix_web::web;
    use diesel::r2d2::ConnectionManager;

    #[tokio::test]
    async fn test_add() {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let database_pool = Pool::builder()
            .build(ConnectionManager::<SqliteConnection>::new(database_url))
            .expect("Failed to create pool.");

        let pool = web::Data::new(database_pool);
        let db_connection = &mut pool.get().unwrap();
        // let test_anime_seed_json = AnimeListJson {
        //     mikan_id: 3143,
        //     anime_name: "米奇与达利".to_string(),
        //     update_day: 1,
        //     img_url: "/images/Bangumi/202310/69e733eb.jpg".to_string(),
        //     anime_type: 0,
        //     subscribe_status: 1,
        //     bangumi_id: 12,
        //     bangumi_rank: "4.3".to_string(),
        //     bangumi_summary: "asdasd".to_string(),
        //     website: "www.baidu.com".to_string(),
        //     anime_status: -1,
        //     total_episodes: -1,
        //     new_finished_episode: 0,
        // };

        // let r = add(db_connection, test_anime_seed_json).await.unwrap();

        let r = get_new_finished_episode_nb(db_connection, &3407).await.unwrap();
        println!("{:?}", r);
    }
}
