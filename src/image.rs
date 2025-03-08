use wasm_bindgen::JsCast;

async fn yield_now() {
    // We will create a Promise that resolves after a short delay to allow the browser to update the UI
    let delay_promise = js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(
            &resolve,
            0,
        ).unwrap();
    });
    // We need to convert the Promise to Rust Future and await it
    wasm_bindgen_futures::JsFuture::from(delay_promise).await.unwrap();
}

pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pixels: Vec<u8>, // RGBA pixel data
}

/*
subpixel: 8b
square error: 16b
resolution like 1920x1080 needs additional 21b, i.e., 37b in total, so u32 is not enough
*/
type TSE = u64;
type TSEF = f64; // less presice than TSE, but 53 bits of significand should be enough; f128 is not stable yet

impl ImageData {
    pub fn get_pixels(&self, x: u32, y: u32, count: usize) -> &[u8] {
        let index = (y * self.width + x) as usize * 4;
        &self.pixels[index..index + 4 * count]
    }

    pub fn from_image(image: &web_sys::HtmlImageElement) -> Result<ImageData, String> {
        let canvas: web_sys::HtmlCanvasElement = web_sys::window().unwrap().document().unwrap().create_element("canvas").map_err(|e| format!("error creating canvas: {:?}", e))?.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        let ctx: web_sys::CanvasRenderingContext2d = canvas.get_context("2d").map_err(|e| format!("error getting 2d context: {:?}", e))?.unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();

        let width = image.natural_width();
        let height = image.natural_height();
    
        // Set canvas size to match image
        canvas.set_width(width);
        canvas.set_height(height);
    
        ctx.draw_image_with_html_image_element(&image, 0.0, 0.0).unwrap();

        Ok(ImageData {
            width,
            height,
            pixels: ctx.get_image_data(0.0, 0.0, width.into(), height.into()).unwrap().data().to_vec(),
        })    
    }

    /**
     * Calculate the total square error between the main image and a search image
     * starting at the given coordinates.
     * max_tse is just a hint, the function may return higher value when max_tse is exceeded.
     */
    pub fn total_square_error(&self, search_image: &ImageData, x: u32, y: u32, max_tse: TSE) -> TSE {
        let mut tse: TSE = 0;
        for dy in 0..search_image.height {
            let main_pixels = self.get_pixels(x, y + dy, search_image.width as usize);
            let search_pixels = search_image.get_pixels(0, dy, search_image.width as usize);
            let square_errors: TSE = main_pixels.iter().zip(search_pixels).map(|(m, s)|
                ((m-s) as i32).pow(2) as TSE
            ).sum();
            tse += square_errors;
            if tse > max_tse {
                return tse;
            }
        }
        tse
    }

    pub async fn find_subimage<F>(self: &ImageData, search_image: &ImageData, progress_callback: F, max_mse: f64) -> String
        where F: Fn(f32) + 'static
    {
        let square_errors_divisor = search_image.width * search_image.height * 4;
        let max_tse = ((max_mse as TSEF) * (square_errors_divisor as TSEF) * 65536.0).ceil() as TSE;
        let total_rows = self.height - search_image.height;
        log::info!("max_tse: {}", max_tse);
        log::info!("MSE for max_tse: {}", (max_tse as f64) / (square_errors_divisor as f64) / 65536.0);

        // y comes first because of memory locality
        for y in 0..(self.height - search_image.height) {
            // Update progress once per row
            let progress = y as f32 / total_rows as f32;
            progress_callback(progress);

            // allow tasks threads to do some work
            yield_now().await;

            log::info!("Checking line {}", y);
            for x in 0..(self.width - search_image.width) {
                let sse = self.total_square_error(&search_image, x, y, max_tse);
                let mse: f64 = (sse as f64) / (square_errors_divisor as f64) / (65536.0);
                if mse < max_mse {
                    log::info!("pos ({}, {}) ({} pxs)", x, y, search_image.width * search_image.height);
                    log::info!("sum of square errors: {} / MSE: {}", sse, mse);
                }
            }
        }

        progress_callback(1.0);

        "Images loaded for processing".to_string()
    }

}
