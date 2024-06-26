use crate::models::anime_broadcast::*;
use crate::schema::anime_broadcast::dsl::*;
use anyhow::Result;
use diesel::dsl::insert_into;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::RunQueryDsl;

// insert single data into anime_broadcast
#[allow(dead_code)]
pub async fn add(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    item: AnimeBroadcastJson,
) -> Result<AnimeBroadcast, diesel::result::Error> {
    match anime_broadcast
        .filter(mikan_id.eq(&item.mikan_id))
        .filter(year.eq(&item.year))
        .filter(season.eq(&item.season))
        .first::<AnimeBroadcast>(db_connection)
    {
        Ok(result) => Ok(result),
        Err(_) => {
            let new_anime_broadcast = PostAnimeBroadcast {
                mikan_id: &item.mikan_id,
                year: &item.year,
                season: &item.season,
            };
            insert_into(anime_broadcast)
                .values(&new_anime_broadcast)
                .execute(db_connection)
                .expect("Error saving new anime");
            let result = anime_broadcast
                .order(id.desc())
                .first(db_connection)
                .unwrap();
            Ok(result)
        }
    }
}

// insert Vec<data> into anime_broadcast
pub async fn add_vec(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    item_vec: Vec<AnimeBroadcastJson>,
) -> Result<i32, diesel::result::Error> {
    let mut sucess_num: i32 = 0;
    for item in &item_vec {
        if let Err(_) = anime_broadcast
            .filter(mikan_id.eq(&item.mikan_id))
            .filter(year.eq(&item.year))
            .filter(season.eq(&item.season))
            .first::<AnimeBroadcast>(db_connection)
        {
            let new_anime_broadcast = PostAnimeBroadcast {
                mikan_id: &item.mikan_id,
                year: &item.year,
                season: &item.season,
            };
            insert_into(anime_broadcast)
                .values(&new_anime_broadcast)
                .execute(db_connection)
                .expect("save failed");
            sucess_num += 1;
        }
    }
    Ok(sucess_num)
}

pub async fn get_by_year_season(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    query_year: i32,
    query_season: i32,
) -> Result<Vec<AnimeBroadcast>, diesel::result::Error> {
    let result: Vec<AnimeBroadcast> = anime_broadcast
        .filter(year.eq(query_year))
        .filter(season.eq(query_season))
        .load::<AnimeBroadcast>(db_connection)?;
    Ok(result)
}

#[allow(dead_code)]
// query all data from anime_broadcast
pub async fn get_all(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
) -> Result<Vec<AnimeBroadcast>, diesel::result::Error> {
    let result: Vec<AnimeBroadcast> = anime_broadcast.load::<AnimeBroadcast>(db_connection)?;
    Ok(result)
}
