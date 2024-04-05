use crate::api::do_anime_task;
use crate::dao;
use crate::models::{anime_broadcast, anime_list, anime_seed, anime_subgroup, anime_task};
use crate::mods::qb_api::QbitTaskExecutor;
use crate::mods::spider::{self, Mikan};
use crate::Pool;
use actix_web::{get, post, web, Error, HttpResponse};
use anyhow::Result;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::SqliteConnection;
use futures::future::join_all;
use log;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

fn handle_error<E: std::fmt::Debug>(e: E, message: &str) -> actix_web::Error {
    log::error!("{}, error: {:?}", message, e);
    actix_web::error::ErrorInternalServerError("Internal server error")
}

fn get_img_name_from_url(img_url: &str) -> Option<String> {
    let parts: Vec<&str> = img_url.split('/').collect();
    if let Some(img_name) = parts.get(4) {
        Some(img_name.to_string())
    } else {
        log::warn!("unexpected img_url format: {}", img_url);
        None
    }
}

#[get("/home")]
pub async fn get_anime_home_handler(pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    log::info!("get_anime_home_handler: /home");
    let db_connection = &mut pool
        .get()
        .map_err(|e| handle_error(e, "get_anime_home_handler, failed to get db connection"))?;

    let res = get_anime_home(db_connection)
        .await
        .map_err(|e| handle_error(e, "get_anime_home_handler, get_anime_home failed"))?;

    Ok(HttpResponse::Ok().json(res))
}

async fn get_anime_home(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
) -> Result<Vec<anime_list::AnimeList>, Error> {
    let mut anime_vec = dao::anime_list::get_by_subscribestatus(db_connection, 1)
        .await
        .map_err(|e| {
            handle_error(
                e,
                "get_anime_home, dao::anime_list::get_by_subscribestatus failed",
            )
        })?;

    let task_vec = dao::anime_task::get_all(db_connection)
        .await
        .map_err(|e| handle_error(e, "get_anime_home, dao::anime_task::get_all failed"))?;

    let mut task_mikan_id_set: HashSet<i32> = HashSet::new();
    for task in task_vec {
        if !task_mikan_id_set.insert(task.mikan_id) {
            continue;
        }
        if let Ok(anime) = dao::anime_list::get_by_mikanid(db_connection, task.mikan_id)
            .await
            .map_err(|e| handle_error(e, "get_anime_home, dao::anime_list::get_by_mikanid failed"))
        {
            if anime.subscribe_status == 0 {
                anime_vec.push(anime);
            }
        }
    }

    for anime in anime_vec.iter_mut() {
        anime.img_url =
            get_img_name_from_url(&anime.img_url).unwrap_or_else(|| anime.img_url.clone());
    }
    anime_vec.sort();
    Ok(anime_vec)
}

#[get("/info/{mikan_id}")]
pub async fn get_anime_info_handler(
    pool: web::Data<Pool>,
    path: web::Path<(i32,)>,
) -> Result<HttpResponse, Error> {
    log::info!("get_anime_info_handler: /info/{}", path.0);
    let db_connection = &mut pool
        .get()
        .map_err(|e| handle_error(e, "get_anime_info_handler, failed to get db connection"))?;

    let res = get_anime_info(db_connection, path.0)
        .await
        .map_err(|e| handle_error(e, "get_anime_info_handler, get_anime_info failed"))?;

    Ok(HttpResponse::Ok().json(res))
}

async fn get_anime_info(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    mikan_id: i32,
) -> Result<anime_list::AnimeList, Error> {
    let mut anime_info = dao::anime_list::get_by_mikanid(db_connection, mikan_id)
        .await
        .map_err(|e| handle_error(e, "get_anime_info, dao::anime_list::get_by_mikanid failed"))?;

    anime_info.img_url =
        get_img_name_from_url(&anime_info.img_url).unwrap_or_else(|| anime_info.img_url.clone());
    Ok(anime_info)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnimeSubscribeRequestJson {
    pub mikan_id: i32,
    pub subscribe_status: i32,
}

#[post("/subscribe")]
pub async fn subscribe_anime_handler(
    pool: web::Data<Pool>,
    item: web::Json<AnimeSubscribeRequestJson>,
) -> Result<HttpResponse, Error> {
    log::info!("subscribe_anime_handler: /subscribe {:?}", item);

    let db_connection = &mut pool
        .get()
        .map_err(|e| handle_error(e, "subscribe_anime_handler, failed to get db connection"))?;

    let res = subscribe_anime(db_connection, item.into_inner())
        .await
        .map_err(|e| handle_error(e, "subscribe_anime_handler, subscribe_anime failed"))?;

    Ok(HttpResponse::Ok().json(res))
}

async fn subscribe_anime(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    item: AnimeSubscribeRequestJson,
) -> Result<(), Error> {
    let mikan_id = item.mikan_id;
    let subscribe_status = if item.subscribe_status == 1 { 0 } else { 1 };

    dao::anime_list::update_subscribestatus_by_mikanid(db_connection, mikan_id, subscribe_status)
        .await
        .map_err(|e| {
            handle_error(
                e,
                "subscribe_anime, dao::anime_list::update_subscribestatus_by_mikanid failed",
            )
        })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BroadcastRequestJson {
    pub year: i32,
    pub season: i32,
}

#[get("/broadcast/{year}/{season}")]
pub async fn get_anime_broadcast_handler(
    pool: web::Data<Pool>,
    path: web::Path<(i32, i32)>,
) -> Result<HttpResponse, Error> {
    let (year, season) = path.into_inner();
    log::info!(
        "get_anime_broadcast_handler: /broadcast/{}/{}",
        year,
        season
    );

    let db_connection = &mut pool.get().map_err(|e| {
        handle_error(
            e,
            "get_anime_broadcast_handler, failed to get db connection",
        )
    })?;

    let res = get_anime_broadcast(db_connection, year, season)
        .await
        .map_err(|e| handle_error(e, "get_anime_broadcast_handler, get_anime_broadcast failed"))?;

    Ok(HttpResponse::Ok().json(res))
}

async fn get_anime_broadcast(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    year: i32,
    season: i32,
) -> Result<Vec<anime_list::AnimeList>, Error> {
    let broadcast_list: Vec<anime_broadcast::AnimeBroadcast> =
        dao::anime_broadcast::get_by_year_season(db_connection, year, season)
            .await
            .map_err(|e| {
                handle_error(
                    e,
                    "get_anime_broadcast, dao::anime_broadcast::get_by_year_season failed",
                )
            })?;

    let mut anime_vec: Vec<anime_list::AnimeList> = Vec::new();

    for anime_broadcast in &broadcast_list {
        let mut anime = dao::anime_list::get_by_mikanid(db_connection, anime_broadcast.mikan_id)
            .await
            .map_err(|e| {
                handle_error(
                    e,
                    "get_anime_broadcast, dao::anime_list::get_by_mikanid failed",
                )
            })?;

        anime.img_url =
            get_img_name_from_url(&anime.img_url).unwrap_or_else(|| anime.img_url.clone());

        anime_vec.push(anime);
    }
    anime_vec.sort();
    Ok(anime_vec)
}

#[post("/broadcast/update")]
pub async fn update_anime_broadcast_handler(
    pool: web::Data<Pool>,
    item: web::Json<BroadcastRequestJson>,
) -> Result<HttpResponse, Error> {
    log::info!(
        "update_anime_broadcast_handler: /broadcast/update {:?}",
        item
    );

    let db_connection = &mut pool.get().map_err(|e| {
        handle_error(
            e,
            "update_anime_broadcast_handler, failed to get db connection",
        )
    })?;

    let res = update_anime_broadcast(db_connection, item.into_inner())
        .await
        .map_err(|e| {
            handle_error(
                e,
                "update_anime_broadcast_handler, update_anime_broadcast failed",
            )
        })?;

    Ok(HttpResponse::Ok().json(res))
}

async fn update_anime_broadcast(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    item: BroadcastRequestJson,
) -> Result<(), Error> {
    let year = item.year;
    let season = item.season;

    let mikan = spider::Mikan::new()?;
    let anime_list = mikan.get_anime(year, season).await?;
    let mut anime_list_json_vec: Vec<anime_list::AnimeListJson> = Vec::new();
    let mut anime_broadcast_json_vec: Vec<anime_broadcast::AnimeBroadcastJson> = Vec::new();
    let mut img_url_vec: Vec<String> = Vec::new();

    for anime in &anime_list {
        anime_list_json_vec.push(anime_list::AnimeListJson {
            mikan_id: anime.mikan_id,
            anime_name: anime.anime_name.clone(),
            img_url: anime.img_url.clone(),
            update_day: anime.update_day,
            anime_type: anime.anime_type,
            subscribe_status: anime.subscribe_status,
        });
        anime_broadcast_json_vec.push(anime_broadcast::AnimeBroadcastJson {
            mikan_id: anime.mikan_id,
            year: item.year,
            season: item.season,
        });
        img_url_vec.push(anime.img_url.clone());
    }

    dao::anime_list::add_vec(db_connection, anime_list_json_vec)
        .await
        .map_err(|e| handle_error(e, "update_anime_broadcast, dao::anime_list::add_vec failed"))?;

    dao::anime_broadcast::add_vec(db_connection, anime_broadcast_json_vec)
        .await
        .map_err(|e| {
            handle_error(
                e,
                "update_anime_broadcast, dao::anime_broadcast::add_vec failed",
            )
        })?;

    // TODO 需要从config中读取
    let save_path = "../../autoAnimeUI/src/assets/images/anime_list".to_string();
    if !img_url_vec.is_empty() {
        let _ = join_all(
            img_url_vec
                .into_iter()
                .map(|img_url| download_anime_img(img_url, &save_path, &mikan)),
        )
        .await;
    }

    Ok(())
}

pub async fn download_anime_img(
    img_url: String,
    save_path: &str,
    mikan: &Mikan,
) -> Result<(), Error> {
    Ok(mikan.download_img(&img_url, save_path).await?)
}

#[get("/seed/{mikan_id}")]
pub async fn get_anime_seed_handler(
    pool: web::Data<Pool>,
    path: web::Path<(i32,)>,
) -> Result<HttpResponse, Error> {
    log::info!("get_anime_seed_handler: /seed/{}", path.0);
    let db_connection = &mut pool
        .get()
        .map_err(|e| handle_error(e, "get_anime_seed_handler, failed to get db connection"))?;

    let res = get_anime_seed(db_connection, path.0)
        .await
        .map_err(|e| handle_error(e, "get_anime_seed_handler, get_anime_seed failed"))?;

    Ok(HttpResponse::Ok().json(res))
}

async fn get_anime_seed(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    mikan_id: i32,
) -> Result<Vec<anime_seed::AnimeSeed>, Error> {
    let seed_info = dao::anime_seed::get_anime_seed_by_mikan_id(db_connection, mikan_id)
        .await
        .map_err(|e| {
            handle_error(
                e,
                "get_anime_seed, dao::anime_seed::get_anime_seed_by_mikan_id failed",
            )
        })?;

    Ok(seed_info)
}

#[get("/subgroup")]
pub async fn get_subgroup_handler(pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    log::info!("get_subgroup_handler: /subgroup");
    let db_connection = &mut pool
        .get()
        .map_err(|e| handle_error(e, "get_subgroup_handler, failed to get db connection"))?;

    let res = get_subgroup(db_connection)
        .await
        .map_err(|e| handle_error(e, "get_subgroup_handler, get_subgroup failed"))?;

    Ok(HttpResponse::Ok().json(res))
}

async fn get_subgroup(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
) -> Result<Vec<anime_subgroup::AnimeSubgroup>, Error> {
    let subgroup_info = dao::anime_subgroup::get_all(db_connection)
        .await
        .map_err(|e| handle_error(e, "get_subgroup, dao::anime_subgroup::get_all failed"))?;

    Ok(subgroup_info)
}

#[get("/task/{mikan_id}")]
pub async fn get_task_handler(
    pool: web::Data<Pool>,
    path: web::Path<(i32,)>,
) -> Result<HttpResponse, Error> {
    log::info!("get_task_handler: /task/{}", path.0);
    let db_connection = &mut pool
        .get()
        .map_err(|e| handle_error(e, "get_task_handler, failed to get db connection"))?;

    let res = get_task(db_connection, path.0)
        .await
        .map_err(|e| handle_error(e, "get_task_handler, get_task failed"))?;

    Ok(HttpResponse::Ok().json(res))
}

async fn get_task(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    mikan_id: i32,
) -> Result<Vec<anime_task::AnimeTask>, Error> {
    let task_info = dao::anime_task::get_exist_anime_task_by_mikan_id(db_connection, mikan_id)
        .await
        .map_err(|e| {
            handle_error(
                e,
                "get_task, dao::anime_task::get_exist_anime_task_by_mikan_id failed",
            )
        })?;

    Ok(task_info)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnimeRequestJson {
    pub mikan_id: i32,
}

#[post("/seed/update")]
pub async fn seed_update_handler(
    pool: web::Data<Pool>,
    item: web::Json<AnimeRequestJson>,
) -> Result<HttpResponse, Error> {
    log::info!("seed_update_handler: /seed/update {:?}", item);

    let db_connection = &mut pool
        .get()
        .map_err(|e| handle_error(e, "seed_update_handler, failed to get db connection"))?;

    let res = seed_update(db_connection, item.into_inner())
        .await
        .map_err(|e| handle_error(e, "seed_update_handler, subscribe_anime failed"))?;

    Ok(HttpResponse::Ok().json(res))
}

async fn seed_update(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    item: AnimeRequestJson,
) -> Result<(), Error> {
    let mikan_id = item.mikan_id;

    let mikan = spider::Mikan::new()?;

    let anime_info = dao::anime_list::get_by_mikanid(db_connection, mikan_id)
        .await
        .map_err(|e| handle_error(e, "seed_update, dao::anime_list::get_by_mikanid failed"))?;

    let anime_type = anime_info.anime_type;

    let subgroup_list = mikan.get_subgroup(mikan_id).await?;

    let mut subgroup_id_vec: Vec<i32> = Vec::new();
    for s in &subgroup_list {
        subgroup_id_vec.push(s.subgroup_id);
    }

    let anime_subgroup_list = convert_spider_subgroup_to_anime_subgroup(subgroup_list);

    dao::anime_subgroup::add_vec(db_connection, anime_subgroup_list)
        .await
        .map_err(|e| handle_error(e, "update_seed, dao::anime_subgroup::add_vec failed"))?;

    let mut seed_vec: Vec<anime_seed::AnimeSeedJson> = Vec::new();
    if !subgroup_id_vec.is_empty() {
        let task_res_vec = join_all(subgroup_id_vec.into_iter().map(|subgroup_id| {
            get_anime_seed_by_spider(mikan_id, subgroup_id, anime_type, &mikan)
        }))
        .await;

        for task_res in task_res_vec {
            match task_res {
                Ok(seed_list) => {
                    seed_vec.extend(seed_list);
                }
                Err(_) => continue,
            }
        }
    }

    dao::anime_seed::add_bulk(db_connection, seed_vec)
        .await
        .map_err(|e| handle_error(e, "update_seed, dao::anime_seed::add_bulk failed"))?;

    Ok(())
}

fn convert_spider_subgroup_to_anime_subgroup(
    spider_vec: Vec<spider::Subgroup>,
) -> Vec<anime_subgroup::AnimeSubgroupJson> {
    spider_vec
        .into_iter()
        .map(|s| anime_subgroup::AnimeSubgroupJson {
            subgroup_name: s.subgroup_name,
            subgroup_id: s.subgroup_id,
        })
        .collect()
}

pub async fn get_anime_seed_by_spider(
    mikan_id: i32,
    subgroup_id: i32,
    anime_type: i32,
    mikan: &spider::Mikan,
) -> Result<Vec<anime_seed::AnimeSeedJson>, Error> {
    let seed_list: Vec<spider::Seed> = mikan.get_seed(mikan_id, subgroup_id, anime_type).await?;
    Ok(convert_spider_seed_to_anime_seed(seed_list))
}

fn convert_spider_seed_to_anime_seed(
    spider_vec: Vec<spider::Seed>,
) -> Vec<anime_seed::AnimeSeedJson> {
    spider_vec
        .into_iter()
        .map(|s| anime_seed::AnimeSeedJson {
            mikan_id: s.mikan_id,
            subgroup_id: s.subgroup_id,
            episode: s.episode,
            seed_name: s.seed_name,
            seed_url: s.seed_url,
            seed_status: s.seed_status,
            seed_size: s.seed_size,
        })
        .collect()
}

#[post("/seed/delete")]
pub async fn seed_delete_handler(
    pool: web::Data<Pool>,
    item: web::Json<AnimeRequestJson>,
) -> Result<HttpResponse, Error> {
    log::info!("seed_delete_handler: /seed/delete {:?}", item);

    let db_connection = &mut pool
        .get()
        .map_err(|e| handle_error(e, "seed_delete_handler, failed to get db connection"))?;

    let res = seed_delete(db_connection, item.into_inner())
        .await
        .map_err(|e| handle_error(e, "seed_delete_handler, seed_delete failed"))?;

    Ok(HttpResponse::Ok().json(res))
}

pub async fn seed_delete(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    item: AnimeRequestJson,
) -> Result<(), Error> {
    dao::anime_seed::delete_anime_seed_by_mikan_id(db_connection, item.mikan_id)
        .await
        .map_err(|e| {
            handle_error(
                e,
                "seed_delete, dao::anime_seed::delete_anime_seed_by_mikan_id",
            )
        })?;

    dao::anime_task::delete_anime_task_by_mikan_id(db_connection, item.mikan_id)
        .await
        .map_err(|e| {
            handle_error(
                e,
                "seed_delete, dao::anime_task::delete_anime_task_by_mikan_id",
            )
        })?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeedRequestJson {
    pub mikan_id: i32,
    pub subgroup_id: i32,
    pub episode: i32,
    pub seed_name: String,
    pub seed_url: String,
    pub seed_status: i32,
    pub seed_size: String,
}
#[post("/seed/download")]
pub async fn seed_download_handler(
    pool: web::Data<Pool>,
    item: web::Json<SeedRequestJson>,
    qb: web::Data<QbitTaskExecutor>,
) -> Result<HttpResponse, Error> {
    log::info!("seed_download_handler: /seed/download {:?}", item);

    let db_connection = &mut pool
        .get()
        .map_err(|e| handle_error(e, "seed_download_handler, failed to get db connection"))?;

    let res = seed_download(db_connection, item.into_inner(), qb)
        .await
        .map_err(|e| handle_error(e, "seed_download_handler, seed_download failed"))?;

    Ok(HttpResponse::Ok().json(res))
}

pub async fn seed_download(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    item: SeedRequestJson,
    qb: web::Data<QbitTaskExecutor>,
) -> Result<(), Error> {
    let mikan = spider::Mikan::new()?;

    dao::anime_seed::update_seedstatus_by_seedurl(db_connection, &item.seed_url, 1)
        .await
        .map_err(|e| {
            handle_error(
                e,
                "seed_download, dao::anime_seed::update_seedstatus_by_seedurl failed",
            )
        })?;

    let anime_seed = convert_json_seed_to_anime_seed(item);

    do_anime_task::create_anime_task_by_seed(&mikan, anime_seed, &qb, db_connection)
        .await
        .map_err(|e| {
            handle_error(
                e,
                "seed_download do_anime_task::create_anime_task_by_seed failed",
            )
        })?;
    Ok(())
}

fn convert_json_seed_to_anime_seed(sj: SeedRequestJson) -> anime_seed::AnimeSeed {
    anime_seed::AnimeSeed {
        id: None,
        mikan_id: sj.mikan_id,
        subgroup_id: sj.subgroup_id,
        episode: sj.episode,
        seed_name: sj.seed_name,
        seed_url: sj.seed_url,
        seed_status: sj.seed_status,
        seed_size: sj.seed_size,
    }
}

#[get("/detail/{mikan_id}")]
pub async fn get_anime_detail_handler(
    pool: web::Data<Pool>,
    path: web::Path<(i32,)>,
) -> Result<HttpResponse, Error> {
    log::info!("get_anime_detail_handler: /detail/{}", path.0);
    let db_connection = &mut pool
        .get()
        .map_err(|e| handle_error(e, "get_anime_detail_handler, failed to get db connection"))?;

    let res = get_anime_detail(db_connection, path.0)
        .await
        .map_err(|e| handle_error(e, "get_anime_detail_handler, get_anime_detail failed"))?;

    Ok(HttpResponse::Ok().json(res))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnimeDetail {
    pub anime_info: anime_list::AnimeList,
    pub seed_info: Vec<anime_seed::AnimeSeed>,
    pub subgroup_info: Vec<anime_subgroup::AnimeSubgroup>,
    pub task_info: Vec<anime_task::AnimeTask>,
}
async fn get_anime_detail(
    db_connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    mikan_id: i32,
) -> Result<AnimeDetail, Error> {
    let anime = get_anime_info(db_connection, mikan_id)
        .await
        .map_err(|e| handle_error(e, "get_anime_detail, get_anime_info failed"))?;
    let seed_vec = get_anime_seed(db_connection, mikan_id)
        .await
        .map_err(|e| handle_error(e, "get_anime_detail, get_anime_seed failed"))?;
    let subgroup_vec = get_subgroup(db_connection)
        .await
        .map_err(|e| handle_error(e, "get_anime_detail, get_subgroup failed"))?;
    let task_vec = get_task(db_connection, mikan_id)
        .await
        .map_err(|e| handle_error(e, "get_anime_detail, get_task failed"))?;

    let anime_detail = AnimeDetail {
        anime_info: anime,
        seed_info: seed_vec,
        subgroup_info: subgroup_vec,
        task_info: task_vec,
    };
    Ok(anime_detail)
}