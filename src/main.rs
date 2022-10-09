use serde::Deserialize;
use std::{env, path::Path, sync::Mutex};

use actix_web::{get, http::StatusCode, web, App, HttpResponse, HttpServer, Responder};
use opencv::{
    core::{convert_scale_abs, Vector},
    imgcodecs,
    prelude::Mat,
};

#[derive(Debug, Deserialize)]
pub struct Request {
    brightness: f64,
    contrast: f64,
}

struct AppState {
    original_image: Mutex<Mat>,
}

enum UserError {
    ConvertScaleAbsError,
    ImageWriteError,
}

impl Request {
    // we need it because library scale brightness default 1.0 [0.0; +inf] and contrast default 0.0 [0.0; +inf]
    // and for task default 0.5 [0.0; +inf] and default 1.0 [0.0; 1.0] respectively
    fn change_scale(&mut self) {
        self.brightness = self.brightness * 2.0;
        self.contrast = 1.0 / self.contrast - 1.0;
    }

    fn convert_image(&mut self, original_image: &mut Mat) -> Result<(), UserError> {
        self.change_scale();
        let mut processed = Mat::default();
        match convert_scale_abs(
            original_image,
            &mut processed,
            self.brightness,
            self.contrast,
        ) {
            Ok(_) => (),
            Err(_) => return Err(UserError::ConvertScaleAbsError),
        }
        match imgcodecs::imwrite("temp_output.JPEG", &processed, &Vector::new()) {
            Ok(_) => (),
            Err(_) => return Err(UserError::ImageWriteError),
        }
        Ok(())
    }
}

#[get("/image")]
async fn index(mut info: web::Query<Request>, data: web::Data<AppState>) -> impl Responder {
    println!(
        "Request for brightness = {} and contrast = {:?}!",
        info.brightness, info.contrast
    );
    if info.brightness < 0.0 {
        return HttpResponse::build(StatusCode::BAD_REQUEST).body(format!(
            "Bad brightness value: {}, allowed range is [0.0; +inf]",
            info.brightness
        ));
    }
    if info.contrast < 0.0 || info.contrast > 1.0 {
        return HttpResponse::build(StatusCode::BAD_REQUEST).body(format!(
            "Bad contrast value: {}, allowed range is [0.0; 1.0]",
            info.contrast
        ));
    }
    match info.convert_image(&mut data.original_image.lock().unwrap()) {
        Ok(_) => (),
        Err(UserError::ConvertScaleAbsError) => {
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Error converting image")
        }
        Err(UserError::ImageWriteError) => {
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Error writing image")
        }
    }

    let image_content = web::block(|| std::fs::read("./temp_output.JPEG"))
        .await
        .unwrap()
        .unwrap();
    HttpResponse::build(StatusCode::OK)
        .content_type("image/jpeg")
        .body(image_content)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() == 2, "There could be only 1 argument!");
    assert!(Path::new(&args[1]).is_file(), "File does not exist");
    let img = match opencv::imgcodecs::imread(args[1].as_str(), opencv::imgcodecs::IMREAD_COLOR) {
        Ok(image) => web::Data::new(AppState {
            original_image: Mutex::new(image),
        }),
        Err(_) => panic!("Error reading image"), // actually if there is no file opencv returns empty matrix
    };
    HttpServer::new(move || App::new().app_data(img.clone()).service(index))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
