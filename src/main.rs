extern crate madome_synchronizer;

use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use anyhow;
use bytes::Bytes;
use env_logger;
use futures::stream::{self, StreamExt};
use itertools::Itertools;
use log::{debug, error, info, trace, warn};
use madome_client::auth::Token;
use madome_client::book::{Book, Language, MetadataBook};
use madome_client::{AuthClient, BookClient, Client, FileClient};
use tokio;

use fp_core::chain::Chain;
use fp_core::extend::Extend;
use fp_core::hkt::HKT;
use fp_core::lens::Lens;

use crate::madome_synchronizer::parser;
use crate::madome_synchronizer::parser::Parser;

use crate::madome_synchronizer::utils::{Flat, FutureUtil, PinFuture, VecUtil};

static MADOME_URL: &'static str = "https://api.madome.app";
static FILE_REPOSITORY_URL: &'static str = "https://file.madome.app";

fn init_logger() {
    env_logger::init()
}

async fn fetch_books(content_ids: Vec<u32>) -> anyhow::Result<Vec<Book>> {
    content_ids
        .into_iter()
        .map(|content_id| {
            Box::pin(async move {
                debug!("Content ID #{}", content_id);

                let (gallery_parser, gallery_block_parser): (
                    Box<parser::Gallery>,
                    Box<parser::GalleryBlock>,
                ) = tokio::try_join!(
                    parser::Gallery::new(content_id).request(),
                    parser::GalleryBlock::new(content_id).request()
                )?;
                debug!("Ready RequestData #{}", content_id);

                let (gallery_data, mut gallery_block_data): (MetadataBook, MetadataBook) =
                    tokio::try_join!(gallery_parser.parse(), gallery_block_parser.parse(),)?;
                debug!("Ready ParseData #{}", content_id);

                gallery_block_data.groups = gallery_data.groups;
                gallery_block_data.characters = gallery_data.characters;

                Ok(Book::from(gallery_block_data))
            }) as PinFuture<Book>
        })
        .collect::<Vec<PinFuture<Book>>>()
        .await_futures()
        .await
}

pub struct TokenLens;

impl Lens<Arc<Token>, String> for TokenLens {
    fn get(s: &Arc<Token>) -> Option<&String> {
        Some(&s.token)
    }

    fn set(a: String, _: &Arc<Token>) -> Arc<Token> {
        Arc::new(Token { token: a })
    }
}

pub struct TokenManage;

impl TokenManage {
    pub async fn refresh(
        auth_client: &AuthClient,
        token: Arc<Token>,
    ) -> anyhow::Result<Arc<Token>> {
        let old_token = TokenLens::get(&token).unwrap();
        let new_token = auth_client.refresh_token(old_token).await?;

        fs::write("./.token", &new_token)?;

        let new_token = TokenLens::set(new_token, &token);

        Ok(new_token)
    }
}

fn chain_string(a: String, b: String) -> String {
    let mut a = a;
    a.push_str(b.as_str());

    a
}

pub struct ParseFails {
    fails: HashSet<u32>,
}

impl ParseFails {
    pub fn add(&mut self, id: u32) -> anyhow::Result<()> {
        if let true = self.fails.insert(id) {
            return Ok(());
        }
        Err(anyhow::Error::msg("Can't insert"))
    }

    pub fn has(&self, id: &u32) -> bool {
        self.fails.contains(id)
    }

    pub fn remove(&mut self, id: &u32) -> anyhow::Result<()> {
        if let true = self.fails.remove(id) {
            return Ok(());
        }
        Err(anyhow::Error::msg("Can't remove"))
    }

    pub fn synchronize(&self, path: &str) -> std::io::Result<()> {
        let chained_string = self.fails.iter().fold(String::from(""), |acc, id| {
            chain_string(acc, format!("{}\n", id))
        });

        fs::write(path, &chained_string)?;

        Ok(())
    }

    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let fails = fs::read_to_string(path)?;

        if fails.is_empty() {
            return Ok(Self {
                fails: HashSet::new(),
            });
        }

        debug!("{:?}", fails.split("\n").collect::<Vec<_>>());

        let fails = fails
            .trim()
            .split("\n")
            .map(|s| u32::from_str_radix(s, 10).unwrap())
            .collect::<HashSet<_>>();

        Ok(Self { fails })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger();

    // let client = Arc::new(Client::new("https://api.madome.app"));

    let auth_client = Arc::new(AuthClient::new(MADOME_URL));
    let book_client = Arc::new(BookClient::new(MADOME_URL));
    let file_client = Arc::new(FileClient::new(FILE_REPOSITORY_URL));

    // parse ids
    // parse books
    // upload books info
    // parse image info
    // download image and upload image

    let mut page: usize = 1;

    'a: loop {
        let token = fs::read("./.token")?;
        let token = String::from_utf8(token)?;
        debug!("{}", token);
        let token = Token { token };
        let token = TokenManage::refresh(&auth_client, Arc::new(token)).await?;
        let fails = ParseFails::from_file("./fails.txt")?;

        let mut non_exists_ids = vec![];

        'b: loop {
            trace!("Parsing IDs");
            let nozomi_parser = parser::Nozomi::new(page, 50, Language::Korean)
                .request()
                .await?;

            let is_not_fail = |id: &u32| !fails.has(id);

            let content_ids = nozomi_parser
                .parse()
                .await?
                .into_iter()
                .filter(is_not_fail)
                .collect::<Vec<_>>();

            // content_ids.sort_by(|a, b| a.cmp(b));

            debug!("{} page ids = {:?}", page, content_ids);

            let last_id_of_page = content_ids.last().unwrap();

            if let Ok(_) = book_client
                .get_image_list(TokenLens::get(&token).unwrap(), *last_id_of_page)
                .await
            {
                break 'b;
            }

            for id in content_ids {
                if let Err(err) = book_client
                    .get_image_list(TokenLens::get(&token).unwrap(), id)
                    .await
                {
                    if err
                        .to_string()
                        .contains(format!("{}", reqwest::StatusCode::NOT_FOUND).as_str())
                    {
                        non_exists_ids.push(id);
                    }
                }
            }

            // debug!("non_exists_ids = {:?}", non_exists_ids);

            break 'b;

            page += 1;
        }

        if non_exists_ids.is_empty() {
            page = 0;
            sleep(Duration::from_secs(3600));
            continue 'a;
        }

        let non_exists_ids = non_exists_ids.into_iter().unique();
        let mut non_exists_ids = non_exists_ids.collect::<Vec<_>>();

        non_exists_ids.sort_by(|a, b| a.cmp(b));

        debug!("non_exists_ids = {:?}", non_exists_ids);

        let fails = Arc::new(Mutex::new(ParseFails::from_file("./fails.txt")?));

        // upload books info
        let _ = fetch_books(vec![non_exists_ids[0]])
            .await?
            .into_iter()
            .map(|book| {
                let file_client = Arc::clone(&file_client);
                let book_client = Arc::clone(&book_client);
                let token = Arc::clone(&token);
                let book = Arc::new(book);
                let fails = Arc::clone(&fails);
                Box::pin(async move {
                    let image_parser = parser::Image::new(book.id);
                    let image_files = image_parser.request().await?.parse().await?;
                    // let image_files = vec![image_files.into_iter().last().unwrap()];

                    debug!("image_files_len = {:?}", image_files.len());

                    let mut image_list: Vec<(String, usize)> = vec![];

                    let seperated_image_files = image_files.seperate(15);

                    debug!("seperated by 15 image_files = {:?}", seperated_image_files);

                    'c: for files in seperated_image_files {
                        let book = Arc::clone(&book);
                        let file_client = Arc::clone(&file_client);
                        let token = Arc::clone(&token);

                        let r = files
                            .into_iter()
                            .map(|(page, file)| {
                                let book = Arc::clone(&book);
                                let file_client = Arc::clone(&file_client);
                                let token = Arc::clone(&token);
                                Box::pin(async move {
                                    if page == 1 {
                                        let thumbnail_bytes = file.download(book.id, true).await?;

                                        trace!("download finish\nid = {}\nthumbnail", book.id);

                                        let filepath =
                                            format!("image/library/{}/thumbnail.jpg", book.id);

                                        file_client
                                            .upload(
                                                TokenLens::get(&token).unwrap(),
                                                filepath.as_str(),
                                                thumbnail_bytes,
                                            )
                                            .await?;

                                        trace!(
                                            "upload finish\nid = {}\nilepath = {}",
                                            book.id,
                                            filepath
                                        );
                                    }

                                    let ext = Path::new(file.name.as_str())
                                        .extension()
                                        .unwrap_or(OsStr::new("img"));
                                    let ext = ext.to_str().unwrap();

                                    debug!(
                                        "id = {}\nimage_page = {}\next = {}",
                                        book.id, page, ext
                                    );

                                    let image_bytes = file.download(book.id, false).await?;

                                    trace!(
                                        "download finish\nid = {}\npage = {}\next = {}",
                                        book.id,
                                        page,
                                        ext
                                    );

                                    let filepath =
                                        format!("image/library/{}/{}.{}", book.id, page, ext);

                                    file_client
                                        .upload(
                                            TokenLens::get(&token).unwrap(),
                                            filepath.as_str(),
                                            image_bytes,
                                        )
                                        .await?;

                                    trace!(
                                        "upload finish\nid = {}\nfilepath = {}",
                                        book.id,
                                        filepath
                                    );

                                    Ok((format!("{}/v1/{}", FILE_REPOSITORY_URL, filepath), page))
                                }) as PinFuture<(String, usize)>
                            })
                            .collect::<Vec<_>>()
                            .await_futures()
                            .await;

                        trace!("sep end");

                        match r {
                            Ok(mut images) => {
                                images.sort_by(|(_, a_page), (_, b_page)| a_page.cmp(b_page));
                                for image in images {
                                    image_list.push(image);
                                }
                                continue 'c;
                            }
                            Err(err) => {
                                trace!("Failed id = {}", book.id);
                                error!("{}", err);
                                fails.lock().unwrap().add(book.id)?;
                                fails.lock().unwrap().synchronize("./fails.txt")?;
                                return Err(err);
                            }
                        }
                    }

                    debug!("image_list = {:?}", image_list);

                    trace!("upload book info id = {}", book.id);
                    image_list.sort_by(|(_, a), (_, b)| a.cmp(b));

                    let image_list_len = image_list.len();

                    let image_list = image_list
                        .into_iter()
                        // .unique()
                        .map(|(url, _)| url)
                        .join("\n");

                    debug!("image_list.txt = {}", image_list);

                    let token = TokenLens::get(&token).unwrap();
                    file_client
                        .upload(
                            token,
                            format!("image/library/{}/image_list.txt", book.id).as_str(),
                            Bytes::from(image_list),
                        )
                        .await?;

                    let mut book: Book = book.into();
                    book.page_count = image_list_len;

                    debug!("book = {:?}", book);

                    book_client.create_book(token, book).await?;

                    Ok(())
                }) as PinFuture<_>
            })
            .collect::<Vec<_>>()
            .await_futures()
            .await;

        break 'a;
    }

    Ok(())
}
