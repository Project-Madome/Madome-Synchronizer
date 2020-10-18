extern crate madome_synchronizer;

use std::env;

use std::fs;

use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

use anyhow;
use env_logger;
use log::{debug, error, info, trace};
use madome_client::auth::Token;
use madome_client::book::{Book, Language};
use madome_client::{AuthClient, BookClient, FileClient};
use rayon::prelude::*;

use fp_core::lens::Lens;

use crate::madome_synchronizer::parser;
use crate::madome_synchronizer::parser::Parser;

use crate::madome_synchronizer::stage::{DownloadStage, ParseStage, UploadStage};
use crate::madome_synchronizer::utils::{IntoResultVec, TextStore, VecUtil};

const MADOME_URL: &'static str = "https://api.madome.app";
const FILE_REPOSITORY_URL: &'static str = "https://file.madome.app";
const TEMP_DIR: &'static str = "./.temp";

fn init_logger() {
    env_logger::init()
}

pub struct TokenLens;

impl Lens<Token, String> for TokenLens {
    fn get(s: &Token) -> Option<&String> {
        Some(&s.token)
    }

    fn set(a: String, _: &Token) -> Token {
        Token { token: a }
    }
}

pub struct TokenManager;

impl TokenManager {
    pub fn refresh(auth_client: &AuthClient, token: Token) -> anyhow::Result<Token> {
        let old_token = TokenLens::get(&token).unwrap();
        let new_token = auth_client.refresh_token(old_token)?;

        fs::write("./.token", &new_token)?;

        let new_token = TokenLens::set(new_token, &token);

        Ok(new_token)
    }
}

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(15)
        .build_global()
        .unwrap();

    if let Err(err) = sync() {
        error!("{:#?}", err);
    }
}

fn get_environment_variables() -> (usize, usize, u64, bool) {
    let is_infinity_synchronize = env::var("INFINITY").is_ok();
    let page = env::var("PAGE").unwrap_or("1".to_string());
    let per_page = env::var("PER_PAGE").unwrap_or("25".to_string());
    let latency = env::var("LATENCY").unwrap_or("3600".to_string());

    let page: usize = page
        .parse()
        .expect("Can't parse PAGE from environment variables");
    let per_page: usize = per_page
        .parse()
        .expect("Can't parse PER_PAGE from environment variables");
    let latency: u64 = latency
        .parse()
        .expect("Can't parse LATENCY from environment variables");

    (page, per_page, latency, is_infinity_synchronize)
}

fn sync() -> anyhow::Result<()> {
    init_logger();

    let (mut page, per_page, latency, is_infinity_synchronize) = get_environment_variables();

    let auth_client = AuthClient::new(MADOME_URL);
    let book_client = BookClient::new(MADOME_URL);
    let file_client = FileClient::new(FILE_REPOSITORY_URL);

    'a: loop {
        let token = fs::read("./.token")?;
        let token = String::from_utf8(token)?.trim().to_string();
        debug!("{}", token);
        let token = Token { token };
        let token = TokenManager::refresh(&auth_client, token)?;
        let fails = Mutex::new(TextStore::from_file("./fails.txt")?);

        trace!("Parsing IDs");
        let nozomi_parser = parser::Nozomi::new(page, per_page, Language::Korean).request()?;


        let content_ids = nozomi_parser.parse()?;

        debug!("{} page ids = {:#?}", page, content_ids);

        let is_not_fail = |id: &u32| !(fails.lock().unwrap().has(id));
        let is_notfound_error = |err: &anyhow::Error| {
            err.to_string()
                .contains(format!("{}", reqwest::StatusCode::NOT_FOUND).as_str())
        };

        let mut non_exists_ids = content_ids
            .into_par_iter()
            .filter(is_not_fail)
            .map(|id| (id, book_client.get_image_list(TokenLens::get(&token).unwrap(), id)))
            .filter_map(|(id , r)| r.err().filter(is_notfound_error).and_then(|_| Some(id)))
            .collect::<Vec<_>>();

        debug!("page = {}", page);
        debug!("non_exists_ids = {:#?}", non_exists_ids);

        if !is_infinity_synchronize && non_exists_ids.is_empty() {
            page = 1;
            sleep(Duration::from_secs(latency));
            continue 'a;
        }

        non_exists_ids.sort_by(|a, b| a.cmp(b));

        debug!("non_exists_ids = {:#?}", non_exists_ids);

        for ids in non_exists_ids.seperate(10) {
            ids.par_iter()
                .try_for_each(|id| fs::create_dir_all(format!("{}/{}", TEMP_DIR, id)))?;

            ids.par_iter()
                .map(|content_id| -> anyhow::Result<_> {
                    debug!("Content ID #{}", content_id);

                    let gallery_parser = parser::Gallery::new(*content_id).request()?;
                    let gallery_block_parser = parser::GalleryBlock::new(*content_id).request()?;
                    debug!("Ready RequestData #{}", content_id);

                    let gallery_data = gallery_parser.parse()?;
                    let mut gallery_block_data = gallery_block_parser.parse()?;
                    debug!("Ready ParseData #{}", content_id);

                    gallery_block_data.groups = gallery_data.groups;
                    gallery_block_data.characters = gallery_data.characters;

                    Ok(Book::from(gallery_block_data))
                })
                .inspect(ParseStage::update)
                .map(|r| -> anyhow::Result<()> {
                    let book = r?;
                    let image_parser = parser::Image::new(book.id);
                    let image_files = image_parser.request()?.parse()?;
                    let image_files_len = image_files.len();

                    debug!("image_files_len = {:#?}", image_files_len);

                    let (origin_url, buf) = image_files[0].download(book.id, true)?;
                    let ext = origin_url.split(".").last().unwrap_or("jpg");
                    let url_path = format!("image/library/{}/thumbnail.{}", book.id, ext);
                    file_client.upload(TokenLens::get(&token).unwrap(), url_path, buf)?;

                    let _download_result = image_files
                        .par_iter()
                        .enumerate()
                        .map(|(current_page, file)| {
                            let current_page = current_page + 1;
                            let (origin_url, buf) = file.download(book.id, false)?;
                            let ext = origin_url.split(".").last().unwrap().to_string();
                            fs::write(
                                format!("{}/{}/{}.{}", TEMP_DIR, book.id, current_page, ext),
                                buf,
                            )?;
                            Ok((book.id, current_page, ext))
                        })
                        .inspect(DownloadStage::update)
                        .collect::<Vec<Result<(u32, usize, String), _>>>()
                        .into_result_vec()?;

                    let image_dir =
                        fs::read_dir(format!("{}/{}", TEMP_DIR, book.id))?.collect::<Vec<_>>();

                    debug!("image_dir = {:#?}", image_dir);

                    let mut upload_result = image_dir
                        .into_par_iter()
                        .enumerate()
                        .map(
                            |(current_page, r)| -> anyhow::Result<(String, Vec<u8>, usize)> {
                                let filename = r?.file_name().to_str().unwrap().to_string();
                                debug!("filename = {}", filename);
                                let buf =
                                    fs::read(format!("{}/{}/{}", TEMP_DIR, book.id, filename))?;
                                Ok((filename, buf, current_page))
                            },
                        )
                        .map(|r| -> anyhow::Result<_> {
                            let (filename, buf, current_page) = r?;
                            let url_path = format!("image/library/{}/{}", book.id, filename);
                            file_client.upload(TokenLens::get(&token).unwrap(), &url_path, buf)?;
                            Ok((
                                format!("{}/v1/{}", FILE_REPOSITORY_URL, url_path),
                                current_page,
                            ))
                        })
                        .inspect(UploadStage::update)
                        .collect::<Vec<Result<(String, usize), _>>>()
                        .into_result_vec()?;

                    upload_result.sort_by(|(_, a), (_, b)| a.cmp(b));

                    let image_list_txt =
                        upload_result
                            .into_iter()
                            .fold(String::new(), |mut acc, (url_path, _)| {
                                acc.push_str(&url_path);
                                acc.push_str("\n");
                                acc
                            });

                    debug!("image_list.txt = {}", image_list_txt);

                    let token = TokenLens::get(&token).unwrap();
                    file_client.upload(
                        token,
                        format!("image/library/{}/image_list.txt", book.id).as_str(),
                        image_list_txt.trim(),
                    )?;

                    let mut book: Book = book.into();
                    book.page_count = image_files_len;

                    info!("book = {:#?}", book);

                    book_client.create_book(token, book)?;

                    Ok(())
                })
                .zip(ids.clone())
                .for_each(|(r, id)| {
                    if let Err(err) = r {
                        trace!("Failed id = {}", id);
                        error!("{:#?}", err);
                        fails.lock().unwrap().add(id).expect("Can't add fails");
                        fails
                            .lock()
                            .unwrap()
                            .synchronize("./fails.txt")
                            .expect("Can't synchronize fails");
                    }
                });

            ids.par_iter()
                .try_for_each(|id| fs::remove_dir_all(format!("{}/{}", TEMP_DIR, id)))?;
        }

        // break 'a;

        page += 1;
    }

    // Ok(())
}
