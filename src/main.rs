use serde::Deserialize;
use std::env;

use actix_web::{get, http::StatusCode, web, App, HttpResponse, HttpServer, Responder};
use opencv::{
    core::{convert_scale_abs, Vector},
    imgcodecs,
};

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    brightness: f64,
    contrast: f64,
}

#[get("/image")]
async fn index(info: web::Query<AuthRequest>) -> impl Responder {
    let out = format!(
        "Request for brightness = {} and contrast = {:?}!",
        info.brightness, info.contrast
    );
    let args: Vec<String> = env::args().collect();
    assert!(args.len() == 2, "There could be only 1 argument!");
    let path_to_original_image = args[1].clone();
    let mut _img = opencv::imgcodecs::imread(
        path_to_original_image.as_str(),
        opencv::imgcodecs::IMREAD_COLOR,
    )
    .unwrap();
    let mut processed = _img.clone();
    let _res =
        convert_scale_abs(&mut _img, &mut processed, info.brightness, info.contrast).unwrap();
    let _res = imgcodecs::imwrite("test_output.JPG", &processed, &Vector::new());

    let image_content = web::block(|| std::fs::read("./test_output.JPG"))
        .await
        .unwrap()
        .unwrap();
    println!("{out}");
    HttpResponse::build(StatusCode::OK)
        .content_type("image/jpeg")
        .body(image_content)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

/*
Нужно разработать бэкенд для простейшего HTTP сервиса отбработки изображений,
позволяющего изменять по запросу параметры яркости и контрастности фиксированного изображения.
Путь до исходного изображения задется при запуске сервиса первым и единственным аргументом
командной строки. После запуска сервис принимает GET запросы
по URL вида: http://127.0.0.1:8080/image?brightness=0.5&contrast=1.0 и
 возвращает в ответ обработанное изображение в формате JPEG, пригодное для
 отображения в браузере как есть (без дополнительного фронтенда).
Яркость изображения регулируется параметром запроса brightness, принимающим
значения в интервале [0.0; +inf]. Контрастность регулируется параметром запроса
contrast, принимающим значения в интервале [0.0; 1.0]. Параметры brightness = 0.5,
 contrast = 1.0 служат средней точкой, в которой картинка должна быть визуально неотличима от оригинала.
Для обработки изображения необходимо использовать библиотеку OpenCV. Можно
использовать любой HTTP фреймворк с поддержкой асинхронности. На дополнительные крейты ограничений нет
*/
