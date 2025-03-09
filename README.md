# Subimage Search

This project is a web application that allows users to search for subimages within a main image. User can configure maximum tolerance for differences. The application is built with Yew, a modern Rust framework for creating multi-threaded front-end web apps with WebAssembly.

## Development

1. Install dependencies:

    ```sh
    npm install
    ```

2. Start the development server:

    ```sh
    npm run dev
    ```

The development server automatically compiles Rust, SASS etc.

## Usage

1. Upload the main image and the search image using the provided input fields.
2. Adjust the search parameters (Maximum Mean Squared Error and Maximum Results).
3. Click the "Search subimage" button to start the search process.
4. View the search results and progress.

## Usage of AI in development

There are various areas with various level of AI usage:

1. The computation (image.rs) – low usage of AI. Smart autocompletion is fine, but I've written the code mostly by myself.
2. UI – I didn't want to write UI, so I tried to tell it to write UI on its own, using a declarative approach through some framework. Github Copilot chose to use yew. The code still needed various adjustments. However, it is great to add a parameter to the computation (e.g., maximum results) and tell the AI to reflect it in UI.
3. Styles – I let the AI to handle this mostly autonomously, with my feedback (e.g., that part needs to be more highlighted). Still, I wanted the SCSS to have some structure and not to contain a dead code.
4. Overhead code (CI and build files) – mostly written by AI.

## License

This project is licensed under the MIT License.
