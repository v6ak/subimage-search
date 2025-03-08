use wasm_bindgen::JsCast;

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


impl ImageData {
    pub fn get_pixel(&self, x: u32, y: u32) -> &[u8] {
        let index = (y * self.width + x) as usize * 4;
        let data: &[u8] = &self.pixels;
        &data[index..index+4]
    }
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

    pub fn total_square_error(&self, search_image: &ImageData, x: u32, y: u32) -> TSE {
        let mut tse: TSE = 0;
        for dy in 0..search_image.height {
            /*for dx in 0..search_image.width {
                let main_pixel = get_pixel(&main_image, x + dx, y + dy);
                let search_pixel = get_pixel(&search_image, dx, dy);
                let square_errors: TSE = main_pixel.iter().zip(search_pixel).map(|(m, s)|
                    ((m-s) as i32).pow(2) as TSE
                ).sum();
                //log::info!("Square errors for {:?} and {:?}: {}", main_pixel, search_pixel,  square_errors);
                sse += square_errors;
            }*/
            let main_pixels = self.get_pixels(x, y + dy, search_image.width as usize);
            let search_pixels = search_image.get_pixels(0, dy, search_image.width as usize);
            let square_errors: TSE = main_pixels.iter().zip(search_pixels).map(|(m, s)|
                ((m-s) as i32).pow(2) as TSE
            ).sum();
            tse += square_errors;
        }
        tse
    }

}
