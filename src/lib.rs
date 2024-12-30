#![deny(clippy::all)]
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use futures::future;
use napi::Result;
use napi_derive::napi;
use playwright::Playwright;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use url::Url;

//get all the links of a website by giving the main url
#[napi]
pub fn scrape_website(start_url: String) -> napi::Result<Vec<String>> {
  //Todo: take array as argunment and scrape the links of all the elements of the array paralley

  let start_time = Instant::now();

  let base_url = Url::parse(&start_url).map_err(|e| napi::Error::from_reason(e.to_string()))?;
  let client = Client::new();
  let visited_links = Arc::new(Mutex::new(HashSet::new()));
  let all_links = Arc::new(Mutex::new(Vec::new()));
  crawl_website(&start_url, &base_url, &client, &visited_links, &all_links);

  let result: Vec<String> = all_links.lock().unwrap().iter().cloned().collect();
  let elapsed_time = start_time.elapsed();
  println!("{:?}", elapsed_time);
  Ok(result)
}

fn crawl_website(
  url: &str,
  base_url: &Url,
  client: &Client,
  visited_links: &Arc<Mutex<HashSet<String>>>,
  all_links: &Arc<Mutex<Vec<String>>>,
) {
  if visited_links.lock().unwrap().contains(url) {
    return;
  }

  visited_links.lock().unwrap().insert(url.to_string());

  let response = match client.get(url).send() {
    Ok(resp) => resp,
    Err(e) => {
      println!("Failed to fetch URL {}: {}", url, e);
      return;
    }
  };

  if !response.status().is_success() {
    println!(
      "Received non-success status code {} for URL: {}",
      response.status(),
      url
    );
    return;
  }

  let html = response.text().unwrap_or_default();
  let document = Html::parse_document(&html);
  let selector = Selector::parse("a").unwrap();
  let base_url_str = base_url.as_str();
  let new_base_url = if base_url_str.starts_with("https://www.") {
    base_url
      .to_string()
      .replace("https://www.", "")
      .replace("https://", "")
  } else {
    base_url.to_string()
  };
  let new_links: Vec<String> = document
    .select(&selector)
    .filter_map(|element| element.value().attr("href"))
    .filter_map(|href| base_url.join(href).ok())
    .map(|url| url.to_string())
    .filter(|url| url.contains(new_base_url.as_str()))
    .filter(|url| {
      !url.contains('#')
        && !url.contains("github.com")
        && !url.contains("discord.com")
        && !url.contains("twitter.com")
        && !url.contains("x.com")
        && !url.contains("facebook.com")
        && !url.contains("instagram.com")
        && !url.contains("web.whatsapp.com")
        && !url.contains("whatsapp.com")
        && !url.contains("youtube.com")
        && !url.contains("telegram.com")
        && !url.contains("?")
        && !url.contains("%")
        && !url.contains("_")
        && !url.contains("/undefined")
        && !url.ends_with(".zip")
        && !url.ends_with(".pdf")
        && !url.ends_with(".txt")
        && !url.ends_with("/signup")
        && !url.ends_with("/sign-up")
        && !url.ends_with("/register")
        && !url.ends_with("/login")
    })
    .collect();
  new_links.par_iter().for_each(|link| {
    if !visited_links.lock().unwrap().contains(link) {
      all_links.lock().unwrap().push(link.clone());
      crawl_website(link, base_url, client, visited_links, all_links);
    }
  });
}

pub async fn scrape_text_from_urls(urls: Vec<String>) -> Result<Vec<String>> {
  let start_time = Instant::now();
  println!("Starting to scrape {} URLs", urls.len());

  // Use `futures::future::join_all` to run all async tasks concurrently
  let tasks: Vec<_> = urls
    .into_iter()
    .map(|url| async move {
      match scrape_text_from_url_playwright(url.clone()).await {
        Ok(text) => text,
        Err(e) => {
          println!("Error scraping {}: {}", url, e);
          format!("Error scraping {}: {}", url, e)
        }
      }
    })
    .collect();

  // Wait for all tasks to complete
  let results = future::join_all(tasks).await;

  let elapsed_time = start_time.elapsed();
  println!(
    "Scraping completed in {:?}. Scraped {} URLs.",
    elapsed_time,
    results.len()
  );

  Ok(results)
}

pub async fn scrape_text_from_url_playwright(url: String) -> napi::Result<String> {
  let playwright = Playwright::initialize()
    .await
    .map_err(|e| napi::Error::from_reason(e.to_string()))?;
  let _ = playwright.prepare();
  let chromium = playwright.chromium();
  let browser = chromium
    .launcher()
    .headless(true)
    .launch()
    .await
    .map_err(|e| napi::Error::from_reason(e.to_string()))?;
  let context = browser
    .context_builder()
    .build()
    .await
    .map_err(|e| napi::Error::from_reason(e.to_string()))?;
  let page = context
    .new_page()
    .await
    .map_err(|e| napi::Error::from_reason(e.to_string()))?;
  page
    .goto_builder(&url)
    .goto()
    .await
    .map_err(|e| napi::Error::from_reason(e.to_string()))?;

  let clean_content_script = r#"
        let header = document.querySelector('header');
        if (header) header.remove();

        let footer = document.querySelector('footer');
        if (footer) footer.remove();

        let headersToRemove = document.querySelectorAll('.header, .site-header, #header, .footer, .site-footer, #footer');
        headersToRemove.forEach(el => el.remove());
        document.body.innerText;
    "#;

  let content = page
    .evaluate::<Value, String>(clean_content_script, serde_json::json!({}))
    .await
    .map_err(|e| napi::Error::from_reason(e.to_string()))
    .unwrap()
    .trim()
    .to_string();

  Ok(content)
}

// fn scrape_text_from_url(url: String) -> napi::Result<String> {
//   let options = LaunchOptionsBuilder::default()
//     .headless(true)
//     .build()
//     .map_err(|e| napi::Error::from_reason(e.to_string()))?;

//   let browser = Browser::new(options).map_err(|e| napi::Error::from_reason(e.to_string()))?;
//   let tab = browser
//     .new_tab()
//     .map_err(|e| napi::Error::from_reason(e.to_string()))?;

//   tab
//     .navigate_to(&url)
//     .map_err(|e| napi::Error::from_reason(e.to_string()))?;
//   tab
//     .wait_until_navigated()
//     .map_err(|e| napi::Error::from_reason(e.to_string()))?;

//   let js_script = r#"
//           function getMainContent() {
//                 let main = document.querySelector('main');
//                 // if (main) return main.innerText;

//                 // If there's no <main> tag, try common content container classes
//                 // let content = document.querySelector('.content, #content, .main-content, #main-content');
//                 // if (content) return content.innerText;

//                 let body = document.body;
//                 let header = document.querySelector('header');
//                 let footer = document.querySelector('footer');
//                 let aside = document.querySelector('aside')
//                 if (header) header.parentNode.removeChild(header);
//                 if (footer) footer.parentNode.removeChild(footer);
//                 if (aside) aside.parentNode.removeChild(aside)
//                 return body.innerText;
//             }
//             getMainContent();
//         "#;

//   let text_content = tab
//     .evaluate(js_script, false)
//     .map_err(|e| napi::Error::from_reason(e.to_string()))?
//     .value
//     .and_then(|v| v.as_str().map(String::from))
//     .ok_or("Failed to extract text content")
//     .map_err(|e| napi::Error::from_reason(e.to_string()))?;

//   let regex = Regex::new(r"\n").map_err(|e| napi::Error::from_reason(e.to_string()))?;
//   let cleaned_content = regex.replace_all(&text_content, " ");
//   let final_content = Regex::new(r"\s{2,}")
//     .map_err(|e| napi::Error::from_reason(e.to_string()))?
//     .replace_all(&cleaned_content, " ");
//   if final_content.is_empty() {
//     return Err(napi::Error::from_reason("No content found".to_string()));
//   }
//   Ok(final_content.into_owned())
// }

#[napi]
pub enum ModelTypes {
  AllMiniLML6V2,
  BGESmallENV15,
  MultilingualE5Large,
  AllMiniLML12V2,
  AllMiniLML6V2Q,
  BGELargeENV15Q,
  NomicEmbedTextV15,
  BGEBaseENV15,
  NomicEmbedTextV1,
  BGEBaseENV15Q,
  ParaphraseMLMiniLML12V2,
  ParaphraseMLMpnetBaseV2,
  MultilingualE5Small,
  GTEBaseENV15,
  GTELargeENV15,
  MxbaiEmbedLargeV1,
  MultilingualE5Base,
  ParaphraseMLMiniLML12V2Q,
  BGESmallENV15Q,
  BGESmallZHV15,
  MxbaiEmbedLargeV1Q,
  NomicEmbedTextV15Q,
  GTEBaseENV15Q,
  AllMiniLML12V2Q,
  GTELargeENV15Q,
  BGELargeENV15,
}

//generate embeddings
#[napi]
pub fn generate_embeddings(
  documents: Vec<String>,
  model_name: ModelTypes,
) -> napi::Result<Vec<Vec<f32>>> {
  let start_time = Instant::now();

  let matched_embedding_model: EmbeddingModel;
  match model_name {
    ModelTypes::AllMiniLML6V2 => matched_embedding_model = EmbeddingModel::AllMiniLML6V2,
    ModelTypes::BGESmallENV15 => matched_embedding_model = EmbeddingModel::BGESmallENV15,
    ModelTypes::MultilingualE5Large => {
      matched_embedding_model = EmbeddingModel::MultilingualE5Large
    }
    ModelTypes::AllMiniLML12V2 => matched_embedding_model = EmbeddingModel::AllMiniLML12V2,
    ModelTypes::AllMiniLML6V2Q => matched_embedding_model = EmbeddingModel::AllMiniLML6V2Q,
    ModelTypes::BGELargeENV15Q => matched_embedding_model = EmbeddingModel::BGELargeENV15Q,
    ModelTypes::NomicEmbedTextV15 => matched_embedding_model = EmbeddingModel::NomicEmbedTextV15,
    ModelTypes::BGEBaseENV15 => matched_embedding_model = EmbeddingModel::BGEBaseENV15,
    ModelTypes::NomicEmbedTextV1 => matched_embedding_model = EmbeddingModel::NomicEmbedTextV1,
    ModelTypes::BGEBaseENV15Q => matched_embedding_model = EmbeddingModel::BGEBaseENV15Q,
    ModelTypes::ParaphraseMLMiniLML12V2 => {
      matched_embedding_model = EmbeddingModel::ParaphraseMLMiniLML12V2
    }
    ModelTypes::ParaphraseMLMpnetBaseV2 => {
      matched_embedding_model = EmbeddingModel::ParaphraseMLMpnetBaseV2
    }
    ModelTypes::MultilingualE5Small => {
      matched_embedding_model = EmbeddingModel::MultilingualE5Small
    }
    ModelTypes::GTEBaseENV15 => matched_embedding_model = EmbeddingModel::GTEBaseENV15,
    ModelTypes::GTELargeENV15 => matched_embedding_model = EmbeddingModel::GTELargeENV15,
    ModelTypes::MxbaiEmbedLargeV1 => matched_embedding_model = EmbeddingModel::MxbaiEmbedLargeV1,
    ModelTypes::MultilingualE5Base => matched_embedding_model = EmbeddingModel::MultilingualE5Base,
    ModelTypes::ParaphraseMLMiniLML12V2Q => {
      matched_embedding_model = EmbeddingModel::ParaphraseMLMiniLML12V2Q
    }
    ModelTypes::BGESmallENV15Q => matched_embedding_model = EmbeddingModel::BGESmallENV15Q,
    ModelTypes::BGESmallZHV15 => matched_embedding_model = EmbeddingModel::BGESmallZHV15,
    ModelTypes::MxbaiEmbedLargeV1Q => matched_embedding_model = EmbeddingModel::MxbaiEmbedLargeV1Q,
    ModelTypes::NomicEmbedTextV15Q => matched_embedding_model = EmbeddingModel::NomicEmbedTextV15Q,
    ModelTypes::GTEBaseENV15Q => matched_embedding_model = EmbeddingModel::GTEBaseENV15Q,
    ModelTypes::AllMiniLML12V2Q => matched_embedding_model = EmbeddingModel::AllMiniLML12V2Q,
    ModelTypes::GTELargeENV15Q => matched_embedding_model = EmbeddingModel::GTELargeENV15Q,
    ModelTypes::BGELargeENV15 => matched_embedding_model = EmbeddingModel::BGELargeENV15,
  }

  let model = TextEmbedding::try_new(
    InitOptions::new(matched_embedding_model).with_show_download_progress(true),
  )
  .map_err(|e| napi::Error::from_reason(e.to_string()))?;
  let embeddings = model
    .embed(documents, Some(512))
    .map_err(|e| napi::Error::from_reason(e.to_string()))?;
  println!("Embeddings generated in {:?}", start_time.elapsed());
  Ok(embeddings)
}
