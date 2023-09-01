extern crate base64;
extern crate gldf_rs;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::error::Error as StdError;

use base64::encode;
use gloo::file::callbacks::FileReader;
use gloo::file::{ObjectUrl,File};
use gloo::console;
use js_sys::{Date, JsString};
use web_sys::{DragEvent, Event, FileList, HtmlInputElement, Blob};
use yew::html::TargetCast;
use yew::{html, Callback, Component, Context, Html};
use yew::prelude::*;
// use yew::prelude::*;
// #use yew::ServerRenderer;
use gldf_rs::gldf::GldfProduct;
use gldf_rs::{FileBufGldf, BufFile};
use reqwest_wasm;
// use wasm_bindgen_test::console_log;
use crate::console::console;
// mod bindings;
// use bindings::string_test;

pub async  fn fetch_text_from_url(url: &str) -> Result<String, reqwest_wasm::Error> {
    let response = reqwest_wasm::get(url).await?;
    response.text().await
}

struct WasmGldfProduct(GldfProduct);
impl WasmGldfProduct{
    pub async fn get_ldc_by_id(self: &Self, file_id: String) -> anyhow::Result<String> {
        let mut result: String = "".to_owned();
        for f in self.0.general_definitions.files.file.iter() {
            if f.id == file_id{
                let mut ldc_path = "ldc/".to_owned();
                let file_name = f.file_name.to_owned();
                if f.type_attr == "url" {
                    result.push_str(fetch_text_from_url(&file_name).await?.as_str());
                } else {
                    ldc_path.push_str(&file_name);
                    result.push_str(&self.0.load_gldf_file_str(ldc_path)?.as_str());
                }
            }
        }
        Ok(result)
    }
    pub fn load_gldf_from_buf_all(buf: Vec<u8>) -> anyhow::Result<FileBufGldf> {
        let file_buf = GldfProduct::load_gldf_from_buf_all(buf)?;
        Ok(file_buf)
    }
}
struct FileDetails {
    name: String,
    file_type: String,
    data: Vec<u8>,
}

pub enum Msg {
    Loaded(String, String, Vec<u8>),
    Files(Vec<File>),
}

pub struct App {
    readers: HashMap<String, FileReader>,
    files: Vec<FileDetails>,
}

#[recursion_limit = "256"]
impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            readers: HashMap::default(),
            files: Vec::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded(file_name, file_type, data) => {
                console::log!("Got Files:", file_type.as_str());
                self.files.push(FileDetails {
                    data,
                    file_type,
                    name: file_name.clone(),
                });
                self.readers.remove(&file_name);
                true
            }
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let file_name = file.name();
                    let file_type = file.raw_mime_type();
                    console::log!("A File:", file_name.as_str(), file_type.as_str());

                    let task = {
                        let link = ctx.link().clone();
                        let file_name = file_name.clone();
                        console::log!("A File:", file_name.as_str(), file_type.as_str());

                        gloo::file::callbacks::read_as_bytes(&file, move |res| {
                            link.send_message(Msg::Loaded(
                                file_name,
                                file_type,
                                res.expect("failed to read file"),
                            ))
                        })
                    };
                    self.readers.insert(file_name, task);
                }
                true
            }
        }
    }
    #[recursion_limit = "256"]
    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div id="wrapper">
                <p id="title">{ "Upload GLDF File" }</p>
                <p id="info">{ "Copyright Holger Trahe" }</p>
                <p id="info"><a href="mailto:trahe@mac.com">{"trahe@mac.com"}</a></p>
                <p id="info"><a id="title" href="https://gldf.io">{"Click here for GLDF Information"}</a></p>
                <p id="info"><a id="title" href="https://eulumdat.icu/">{"Also check QLumEdit in the Browser"}</a></p>
                <p>{"This is without any backend, no data is uploaded"}</p>
                <p>{"Everything happens only in your browser"}</p>
                <p>{"Actually it is all in a signed .wasm, to be even more trustfull!"}</p>
                <p>{"Ask your tech-guys they will confirm ;)"}</p>
                <p>{"It shows the content of the GLDF Container"}</p>
                <p>{"And for more the convenient workflow, it shows as well the JSON version"}</p>
                <p>{"New Feature now has the Open in QLumEdit, which will load the .ldt file in QLumEdit in a new tab"}</p>

                <label for="file-upload">
                    <div
                        id="drop-container"
                        ondrop={ctx.link().callback(|event: DragEvent| {
                            event.prevent_default();
                            let files = event.data_transfer().unwrap().files();
                            Self::upload_files(files)
                        })}
                        ondragover={Callback::from(|event: DragEvent| {
                            event.prevent_default();
                        })}
                        ondragenter={Callback::from(|event: DragEvent| {
                            event.prevent_default();
                        })}
                    >
                        <i class="fa fa-cloud-upload"></i>
                        <p>{"Drop your GLDF file here or click to select"}</p>
                    </div>
                </label>
                <input
                    id="file-upload"
                    type="file"
                    accept="image/*,video/*"
                    multiple={true}
                    onchange={ctx.link().callback(move |e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Self::upload_files(input.files())
                    })}
                />
                <div id="preview-area">
                    { for self.files.iter().map(Self::view_file) }
                </div>
            </div>
        }
    }
}

fn parse_file_for_gldf(file: &FileDetails) -> FileBufGldf{
    return WasmGldfProduct::load_gldf_from_buf_all(file.data.clone()).unwrap();
}

pub fn get_blob(buf_file: &BufFile) -> String {
    let uint8arr = js_sys::Uint8Array::new(&unsafe { js_sys::Uint8Array::view(&buf_file.content.clone().unwrap()) }.into());
    let array = js_sys::Array::new();
    array.push(&uint8arr.buffer());
    let blob = Blob::new_with_str_sequence_and_options(
        &array,
        web_sys::BlobPropertyBag::new().type_("application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
    ).unwrap();
    let download_url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
    return download_url
}

impl App {

    fn view_file(file: &FileDetails) -> Html {
        fn view_gldf_file(buf_file: &BufFile) -> Html {
            // console_log!("Action for file_type{?}:", buf_file);
            html! {
                <div id="buf_file"><p>{ format!("{}", buf_file.name.clone().unwrap_or("".to_string())) }</p>
                    if buf_file.name.clone().expect("REASON").to_lowercase().ends_with(".jpg"){
                        <img src={format!("data:image/jpeg;base64,{}", encode(buf_file.clone().content.unwrap_or(Vec::new()))  )} />
                    }
                    else if buf_file.name.clone().expect("REASON").to_lowercase().ends_with(".png"){
                        <img src={format!("data:image/png;base64,{}", encode(buf_file.clone().content.unwrap_or(Vec::new()))  )} />
                    }
                    else if buf_file.name.clone().expect("REASON").to_lowercase().ends_with(".ldt"){
                        // <a href={format!(r"http://gldf.icu/QLumEdit/QLumEdit.html?ldc_name={}&ldc_base64={}",buf_file.name, base64::encode(buf_file.content.clone()))}</a>
                        // base64 is the bad approach, as it transfers data to the server, thus can be somehow intercepted and is just unneeded garbage as the server shall not know anyhow
                        // <a target="_blank" href={format!(r"https://eulumdat.icu/?ldc_name=trahe.ldt&ldc_base64={}", encode(buf_file.clone().content.unwrap_or(Vec::new())))}>{"Open in QLumEdit"}</a>
                        <a href={format!(r"/QLumEdit/QLumEdit.html?ldc_name=trahe.ldt&ldc_blob_url={}", get_blob(buf_file))}>{"Open in QLumEdit"}</a>
                        // <a target="_blank" href={format!(r"/QLumEdit/QLumEdit.html?ldc_name=trahe.ldt&ldc_blob_url={}", get_blob(buf_file))}>{"Open in QLumEdit"}</a>
                        <br/><textarea value={format!(r"{}", String::from_utf8_lossy(buf_file.content.clone().unwrap().as_slice()))}></textarea>
                   //     <textarea value={format!(r"{}", string_test("test"))}></textarea>
                    }
                    else if buf_file.name.clone().expect("REASON").to_lowercase().ends_with(".xml"){
                        <textarea value={format!(r"{}", String::from_utf8_lossy(buf_file.content.clone().unwrap().as_slice()))}></textarea>
                    }
                </div>
            }
        }
        console::log!("Action for file_type:", file.file_type.as_str(), file.name.as_str());
        //let mut ldt_file_contents= Vec::with_capacity(0);
        let mut have_gldf = false;
        let loaded = parse_file_for_gldf(file);
        let buf_files = loaded.files.to_vec();
        console::log!("Files:{?}", buf_files.len());
        console::log!("Author:", loaded.gldf.header.author.as_str());
        html! {
            <div class="preview-tile">
                <p class="preview-name">{ format!("{}", file.name) }</p>
                <div class="preview-media">
                    if file.file_type.contains("url") {
                        <img src={ format!("{}", file.name) } />
                    }
                    else if file.file_type.contains("image") {
                        <img src={format!("data:{};base64,{}", file.file_type, encode(&file.data))} />
                    }
                    else if file.file_type.contains("video") {
                        <video controls={true}>
                            <source src={format!("data:{};base64,{}", file.file_type, encode(&file.data))} type={file.file_type.clone()}/>
                        </video>
                    } else if file.name.as_str().ends_with(".gldf"){
                        //<textarea value={format!(r#"{{"product": {}}}"#, GldfProduct.gldf::load_gldf_from_buf(file.data.clone()).unwrap().to_json().expect("REASON").as_str())}></textarea>
                        <textarea value={format!(r#"{{"product": {}}}"#, loaded.gldf.to_pretty_json().expect("REASON").as_str())}></textarea>
                        { for buf_files.iter().map(view_gldf_file) }
                    }
                    else {
                        <p>{ "Unknown file type" }</p>
                    }
                    // for file in loaded.gldf.general_definitions.files.file.iter() {
                    //     let content_type = &file.content_type;
                    //     <source src={format!("{}", file.file_type)}/>
                    // }

                </div>
            </div>
        }

    }

    fn upload_files(files: Option<FileList>) -> Msg {
        let mut result = Vec::new();

        if let Some(files) = files {
            let files = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .map(|v| web_sys::File::from(v.unwrap()))
                .map(File::from);
            result.extend(files);
        }
        Msg::Files(result)
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}


// #[tokio::main(flavor = "current_thread")]
// async fn no_main() {
//     let renderer = ServerRenderer::<App>::new();
//
//     let rendered = renderer.render().await;
//
//     // Prints: <div>Hello, World!</div>
//     println!("{}", rendered);
// }