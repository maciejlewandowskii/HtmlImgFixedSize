use std::fs;
use std::path::Path;
use clap::Parser;
use imageinfo::ImageInfo;
use kuchiki::traits::TendrilSink;
use progress_bar::{finalize_progress_bar, inc_progress_bar, init_progress_bar, print_progress_bar_info, set_progress_bar_action, Color, Style};

#[derive(Parser, Debug)]
#[command(version = "1.0", about = "Implements fixed image size based on liked images")]
struct Args {
    #[arg()]
    html_file_path: String
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("Processing {} Html...", args.html_file_path);
    let html_file = fs::read_to_string(&args.html_file_path)
        .expect("Cannot Read File");

    let doc = kuchiki::parse_html().one(html_file.as_str());

    let images: Vec<_> = doc.select("img")
        .expect("Can't Select Images From Html").collect();
    let images_count = images.len();

    println!("Found {} images, starting size determination...", &images_count);
    init_progress_bar(images_count);
    set_progress_bar_action("Processing", Color::White, Style::Bold);

    for img in images {
        let mut img_attr = img.attributes.borrow_mut();

        match img_attr.get("src") {
            Some(img_src) => {
                if img_src.contains("http") {
                    let image = reqwest::get(img_src).await;
                    match image {
                        Ok(image) => {
                            let image_data = &image.bytes().await;
                            match image_data {
                                Ok(image_data) => {
                                    match ImageInfo::from_raw_data(image_data) {
                                        Ok(img_info) => {
                                            img_attr.insert("width", img_info.size.width.to_string());
                                            img_attr.insert("height", img_info.size.height.to_string());
                                        }
                                        Err(e) => print_progress_bar_info("Failed", e.to_string().as_str(), Color::Red, Style::Bold)
                                    }
                                }
                                Err(e) => print_progress_bar_info("Failed", e.to_string().as_str(), Color::Red, Style::Bold)
                            }
                        }
                        Err(e) => print_progress_bar_info("Failed", e.to_string().as_str(), Color::Red, Style::Bold)
                    }

                }
                else { print_progress_bar_info("Failed", "to load image, image src is relative.", Color::Red, Style::Bold); }
            }
            None => print_progress_bar_info("Failed", "to load image, no src attribute.", Color::Red, Style::Bold)
        }

        inc_progress_bar();
    }
    finalize_progress_bar();

    println!("Done !");

    let result_html_path = Path::new(&args.html_file_path).with_file_name("imageFixedSize.html");
    doc.serialize_to_file(&result_html_path)
        .expect("Cannot Save Result File");

    println!("Saved to: {}", result_html_path.to_str().unwrap());
}
