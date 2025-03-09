use gloo::utils::{document, window};
use wasm_bindgen::JsCast;

async fn yield_now() {
    // We will create a Promise that resolves after a short delay to allow the browser to update the UI
    let delay_promise = js_sys::Promise::new(&mut |resolve, _| {
        window()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0)
            .unwrap();
    });
    // We need to convert the Promise to Rust Future and await it
    wasm_bindgen_futures::JsFuture::from(delay_promise)
        .await
        .unwrap();
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
type TotalSquaredError = u64;
type TotalSquaredErrorFloat = f64; // less presice than TSE, but 53 bits of significand should be enough; f128 is not stable yet

impl ImageData {
    pub fn get_pixels(&self, x: u32, y: u32, count: usize) -> &[u8] {
        let index = (y * self.width + x) as usize * 4;
        &self.pixels[index..index + 4 * count]
    }

    pub fn from_image(image: &web_sys::HtmlImageElement) -> Result<ImageData, String> {
        let canvas: web_sys::HtmlCanvasElement = document()
            .create_element("canvas")
            .map_err(|e| format!("error creating canvas: {:?}", e))?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        let ctx: web_sys::CanvasRenderingContext2d = canvas
            .get_context("2d")
            .map_err(|e| format!("error getting 2d context: {:?}", e))?
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        let width = image.natural_width();
        let height = image.natural_height();

        // Set canvas size to match image
        canvas.set_width(width);
        canvas.set_height(height);

        ctx.draw_image_with_html_image_element(image, 0.0, 0.0)
            .unwrap();

        Ok(ImageData {
            width,
            height,
            pixels: ctx
                .get_image_data(0.0, 0.0, width.into(), height.into())
                .unwrap()
                .data()
                .to_vec(),
        })
    }

    /**
     * Calculate the total square error between the main image and a search image
     * starting at the given coordinates.
     * max_tse is just a hint, the function may return higher value when max_tse is exceeded.
     */
    pub fn total_square_error(
        &self,
        search_image: &ImageData,
        x: u32,
        y: u32,
        max_tse: TotalSquaredError,
    ) -> TotalSquaredError {
        let mut tse: TotalSquaredError = 0;
        for dy in 0..search_image.height {
            let main_pixels = self.get_pixels(x, y + dy, search_image.width as usize);
            let search_pixels = search_image.get_pixels(0, dy, search_image.width as usize);
            let square_errors: TotalSquaredError = main_pixels
                .iter()
                .zip(search_pixels)
                .map(|(m, s)| ((m - s) as i32).pow(2) as TotalSquaredError)
                .sum();
            tse += square_errors;

            // We might do this in the inner cycle. It would be more precise, but with more overhead. Not sure which is better.
            if tse > max_tse {
                return tse;
            }
        }
        tse
    }

    pub async fn find_subimage<F>(
        self: &ImageData,
        search_image: &ImageData,
        progress_callback: F,
        max_mse: f64,
        max_results: u16,
    ) -> Result<SearchResults, String>
    where
        F: Fn(f32) + 'static,
    {
        let squared_errors_divisor = search_image.width * search_image.height * 4;
        let max_tse = ((max_mse as TotalSquaredErrorFloat)
            * (squared_errors_divisor as TotalSquaredErrorFloat)
            * 65536.0)
            .ceil() as TotalSquaredError;
        let mut results = SearchResults::new(
            max_results,
            search_image.width,
            search_image.height,
            self.width,
            self.height,
            squared_errors_divisor,
            max_tse,
        );

        let total_rows = self.height - search_image.height;
        log::info!("max_tse: {}", max_tse);
        log::info!(
            "MSE for max_tse: {}",
            (max_tse as f64) / (squared_errors_divisor as f64) / 65536.0
        );

        if self.height < search_image.height {
            return Err("Main image height is smaller than search image height".to_string());
        }
        if self.width < search_image.width {
            return Err("Main image width is smaller than search image width".to_string());
        }
        // y comes first because of memory locality
        // half-open interval, hence + 1 for the upper bound
        for y in 0..(self.height - search_image.height + 1) {
            // Update progress once per row
            let progress = y as f32 / total_rows as f32;
            progress_callback(progress);

            // allow tasks threads to do some work
            yield_now().await;

            log::info!("Checking line {}", y);
            // half-open interval, hence + 1 for the upper bound
            for x in 0..(self.width - search_image.width + 1) {
                let tse = self.total_square_error(search_image, x, y, results.tse_threshold);
                if tse <= results.tse_threshold {
                    results.push(SearchResult { x, y, tse });
                    log::info!(
                        "pos ({}, {}) ({} pxs)",
                        x,
                        y,
                        search_image.width * search_image.height
                    );
                }
            }
        }

        progress_callback(1.0);

        Ok(results.finalize())
    }
}

#[derive(Debug)]
pub struct SearchResult {
    pub x: u32,
    pub y: u32,
    pub tse: TotalSquaredError,
}

impl SearchResult {
    pub fn get_mse(&self, square_errors_divisor: u32) -> f64 {
        (self.tse as f64) / (square_errors_divisor as f64) / 65536.0
    }
}

#[derive(Debug)]
pub struct SearchResults {
    // We expect about 100 items max => inserting in the first position causes move of cca 1 600 bytes.
    // Not sure if it more or less than allocation overhead caused by tree structures etc, but it is acceptable.
    // Ordered by mse ascending. Not sure if ascending or descending order is better.
    results_ordered: Vec<SearchResult>,
    capacity: u16,
    overflown: bool,
    template_width: u32,
    template_height: u32,
    main_width: u32,
    main_height: u32,
    squared_error_divisor: u32,
    tse_threshold: TotalSquaredError,
}

impl SearchResults {
    pub fn new(
        capacity: u16,
        template_width: u32,
        template_height: u32,
        main_width: u32,
        main_height: u32,
        squared_error_divisor: u32,
        tse_threshold: TotalSquaredError,
    ) -> SearchResults {
        SearchResults {
            results_ordered: Vec::with_capacity(capacity as usize),
            capacity,
            overflown: false,
            template_height,
            template_width,
            main_width,
            main_height,
            squared_error_divisor,
            tse_threshold,
        }
    }
    pub fn push(&mut self, result: SearchResult) {
        if self.results_ordered.len() < self.capacity as usize {
            self.insert_ordered(result);
        } else {
            self.overflown = true;
            if result.tse < self.results_ordered[self.results_ordered.len() - 1].tse {
                self.results_ordered.pop();
                self.insert_ordered(result);
            } else {
                // not worth inserting
            }
        }
        assert!(
            self.results_ordered.len() <= self.capacity as usize,
            "results_ordered.len() <= self.capacity"
        );
    }
    fn insert_ordered(&mut self, result: SearchResult) {
        // find element with higher tse
        match self.results_ordered.iter().position(|r| r.tse > result.tse) {
            Some(pos) => self.results_ordered.insert(pos, result), // insert before the first element with higher tse
            None => self.results_ordered.push(result),
        }
        if self.results_ordered.len() == self.capacity as usize {
            // results with the same TSE aren't interesting, so we could go even lower (-1),
            // but we have to handle integer underflow
            self.tse_threshold =
                TotalSquaredError::saturating_sub(self.results_ordered.last().unwrap().tse, 1);
            log::info!(
                "too many items, new TSE threshold: {} => MSE threshold: {}",
                self.tse_threshold,
                (self.tse_threshold as f64) / (self.squared_error_divisor as f64) / 65536.0
            );
        }
    }
    pub fn has_overflown(&self) -> bool {
        self.overflown
    }
    fn shrink(&mut self) {
        self.results_ordered.shrink_to_fit();
    }
    fn finalize(mut self) -> Self {
        self.shrink();
        self
    }
    pub fn get_matches(&self) -> &[SearchResult] {
        &self.results_ordered
    }
    pub fn get_template_height(&self) -> u32 {
        self.template_height
    }
    pub fn get_template_width(&self) -> u32 {
        self.template_width
    }
    pub fn get_main_height(&self) -> u32 {
        self.main_height
    }
    pub fn get_main_width(&self) -> u32 {
        self.main_width
    }
    pub fn get_square_errors_divisor(&self) -> u32 {
        self.squared_error_divisor
    }
}
