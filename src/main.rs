extern crate madome_synchronizer;

use std::env;
use std::fs;
use std::sync::Mutex;
use std::thread;
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

use crate::madome_synchronizer::stage::{self, Stage};
use crate::madome_synchronizer::utils::{IntoResultVec, TextStore};

const MADOME_URL: &'static str = "https://api.madome.app";
const FILE_REPOSITORY_URL: &'static str = "https://file.madome.app";

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

#[derive(Debug)]
struct Config {
    infinity_synchronize: bool,
    retry_fail: bool,
    page: usize,
    per_page: usize,
    latency: u64,

    specified_id: Option<u32>,
}

impl Config {
    pub fn new() -> Self {
        let infinity_synchronize = env::var("INFINITY").is_ok();
        let retry_fail = env::var("RETRY_FAIL").is_ok();
        let page = env::var("PAGE").unwrap_or("1".to_string());
        let per_page = env::var("PER_PAGE").unwrap_or("25".to_string());
        let latency = env::var("LATENCY").unwrap_or("3600".to_string());
        let specified_id = env::var("ID").ok().and_then(|x| x.parse::<u32>().ok());

        let page: usize = page
            .parse()
            .expect("Can't parse PAGE from environment variables");
        let per_page: usize = per_page
            .parse()
            .expect("Can't parse PER_PAGE from environment variables");
        let latency: u64 = latency
            .parse()
            .expect("Can't parse LATENCY from environment variables");

        Self {
            infinity_synchronize,
            retry_fail,
            page,
            per_page,
            latency,

            specified_id,
        }
    }
}

fn parse_ids(page: usize, per_page: usize, language: Language) -> anyhow::Result<Vec<u32>> {
    trace!("parse_ids({}, {}, {:#?})", page, per_page, language);
    parser::Nozomi::new(page, per_page, language)
        .request()?
        .parse()
}

fn parse_images(id: u32) -> anyhow::Result<Vec<parser::File>> {
    trace!("parse_image({})", id);
    parser::Image::new(id).request()?.parse()
}

fn get_ext(x: &str) -> Option<&str> {
    x.split('.').last()
}

fn add_images(id: u32, images: &Vec<parser::File>, token: &Token) -> anyhow::Result<()> {
    let file_client = FileClient::new(FILE_REPOSITORY_URL);

    images
        .par_iter()
        .enumerate()
        .map(|(page, file)| (page + 1, file))
        .map(|(page, image)| -> anyhow::Result<String> {
            image.download(id, false).and_then(|(origin_url, buf)| {
                let ext = get_ext(&origin_url).unwrap_or("jpg");
                let filename = format!("{}.{}", page, ext);
                let url_path = format!("image/library/{}/{}", id, filename);

                file_client.upload(TokenLens::get(token).unwrap(), &url_path, buf)?;

                Ok(url_path)
            })
        })
        .collect::<Vec<_>>()
        .into_result_vec()
        .and_then(|image_list| add_image_list(id, image_list, token))
}

fn add_thumbnail(id: u32, image: &parser::File, token: &Token) -> anyhow::Result<()> {
    let file_client = FileClient::new(FILE_REPOSITORY_URL);

    image.download(id, true).and_then(|(origin_url, buf)| {
        let ext = get_ext(&origin_url).unwrap_or("jpg");
        let url_path = format!("image/library/{}/thumbnail.{}", id, ext);
        file_client.upload(TokenLens::get(&token).unwrap(), url_path, buf)
    })
}

fn add_image_list(id: u32, image_list: Vec<String>, token: &Token) -> anyhow::Result<()> {
    let file_client = FileClient::new(FILE_REPOSITORY_URL);

    let image_list_txt = image_list
        .into_iter()
        .fold(String::new(), |mut acc, url_path| {
            acc.push_str(&format!("{}/{}", FILE_REPOSITORY_URL, url_path));
            acc.push_str("\n");
            acc
        });

    file_client.upload(
        TokenLens::get(token).unwrap(),
        format!("image/library/{}/image_list.txt", id).as_str(),
        image_list_txt.trim(),
    )
}

fn parse_book(id: u32, page: usize) -> anyhow::Result<Book> {
    let gallery_data = parser::Gallery::new(id).request()?.parse()?;
    let mut gallery_block_data = parser::GalleryBlock::new(id).request()?.parse()?;

    gallery_block_data.groups = gallery_data.groups;
    gallery_block_data.characters = gallery_data.characters;

    Ok(Book {
        page_count: page,
        ..Book::from(gallery_block_data)
    })
}

fn add_book(book: Book, token: &Token) -> anyhow::Result<()> {
    let book_client = BookClient::new(MADOME_URL);

    let book: Book = book.into();
    book_client.create_book(TokenLens::get(token).unwrap(), book)
}

fn add_fail(id: u32, fail_store: &Mutex<TextStore<u32>>) {
    fail_store.lock().unwrap().add(id).expect("Can't add fails");
    fail_store
        .lock()
        .unwrap()
        .synchronize("./fails.txt")
        .expect("Can't synchronize fails");
}

fn sync(id: u32, token: &Token, fail_store: &Mutex<TextStore<u32>>) -> anyhow::Result<()> {
    parse_images(id)
        .and_then(|images| stage::update(id, Stage::ParsedImages).and_then(|_| Ok(images)))
        .and_then(|images| {
            add_thumbnail(id, &images[0], token)
                .and_then(|_| stage::update(id, Stage::AddedThumbnail))
                // add images and image_list.txt
                .and_then(|_| add_images(id, &images, token))
                .and_then(|_| stage::update(id, Stage::AddedImages))
                .and_then(|_| Ok(images.len()))
        })
        .and_then(|images_len| parse_book(id, images_len))
        .and_then(|book| stage::update(id, Stage::ParsedBook).and_then(|_| Ok(book)))
        .and_then(|book| add_book(book, token))
        .and_then(|_| stage::update(id, Stage::AddedBook))
        .map_err(|err| {
            error!("{:#?}", err);
            add_fail(id, &fail_store);
            stage::update(id, Stage::Fail).unwrap();
            err
        })
}

fn main() {
    init_logger();

    rayon::ThreadPoolBuilder::new()
        .num_threads(15)
        .build_global()
        .unwrap();

    loop {
        let a = thread::spawn(|| -> anyhow::Result<()> {
            let config = Config::new();

            info!("{:#?}", config);

            let Config {
                mut page,
                per_page,
                latency,
                infinity_synchronize,
                retry_fail,
                specified_id,
            } = config;

            let auth_client = AuthClient::new(MADOME_URL);
            let book_client = BookClient::new(MADOME_URL);

            let token = fs::read("./.token")?;
            let token = String::from_utf8(token)?.trim().to_string();
            let token = Token { token };
            let token = TokenManager::refresh(&auth_client, token)?;
            let fail_store = Mutex::new(TextStore::from_file("./fail_store.txt")?);

            let is_not_fail = |id: &u32| {
                if retry_fail {
                    return true;
                }
                !(fail_store.lock().unwrap().has(id))
            };
            let is_notfound_error = |err: &anyhow::Error| {
                err.to_string()
                    .contains(format!("{}", reqwest::StatusCode::NOT_FOUND).as_str())
            };

            if let Some(id) = specified_id {
                sync(id, &token, &fail_store).unwrap();
                std::process::exit(0)
            }

            'a: loop {
                let ids = if retry_fail {
                    let r = fail_store
                        .lock()
                        .unwrap()
                        .iter()
                        .map(|id| *id)
                        .collect::<Vec<_>>();

                    Ok(r)
                } else {
                    parse_ids(page, per_page, Language::Korean)
                };

                let r = ids
                    .and_then(|ids| {
                        let ids = ids
                            .into_par_iter()
                            .filter(is_not_fail)
                            .filter_map(|id| {
                                book_client
                                    .get_image_list(TokenLens::get(&token).unwrap(), id)
                                    .err()
                                    .filter(is_notfound_error)
                                    .and_then(|_| Some(id))
                            })
                            .collect::<Vec<_>>();

                        Ok(ids)
                    })
                    .and_then(|ids| {
                        if ids.is_empty() && !infinity_synchronize {
                            return Err(anyhow::Error::msg("empty ids"));
                        }

                        Ok(ids)
                    })
                    .and_then(|ids| {
                        ids.into_par_iter()
                            .try_for_each(|id| sync(id, &token, &fail_store))
                    });

                if let Err(err) = r {
                    if err.to_string() == "empty ids" {
                        page = 1;
                        thread::sleep(Duration::from_secs(latency));
                        continue 'a;
                    }

                    return Err(err);
                }

                page += 1;
            }
        });

        if let Err(err) = a.join() {
            error!("{:#?}", err);
        }
    }
}
