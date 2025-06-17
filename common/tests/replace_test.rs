use async_std::path::Path;
use common::office::zip::{new as new_zip, Zip};
use common::replace::data::Data;
use common::replace::data::Value::Text;
use common::replace::index::{File, Replace};
use common::{file_decode, file_encode};
use futures::future::join_all;
use futures::FutureExt;
use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind};
use std::time::Instant;

const TYPE_WORD: u8 = 0;
const TYPE_EXCEL: u8 = 1;


async fn new_office(file: Vec<u8>, name: String) -> Result<(Zip, String), Error> {
    match new_zip(file).await {
        Ok(zip) => Ok((zip, name)),
        Err(err) => Err(Error::new(ErrorKind::Other, "")),
    }
}

struct FileList {
    names: Vec<String>,
    data: Vec<File>,
}

async fn read_dir(path: &str) -> FileList {
    let dirs = fs::read_dir(path).unwrap();

    let mut tasks = vec![];
    dirs.for_each(|dir| {
        let path = dir.unwrap().path();
        if path.is_dir() {
            return;
        }

        let path = Path::new(&path);
        if let Ok(data) = fs::read(path) {
            let name = path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name).unwrap();
            tasks.push(new_office(data, name.to_string()));
        }
    });

    let mut res = join_all(tasks).await;


    let mut list = FileList {
        names: vec![],
        data: vec![],
    };

    while let Some(r) = res.pop() {
        if let Ok((zip, name)) = r {
            list.data.push(File::Zip(zip));
            list.names.push(name);
        }
    }

    list
}


#[async_std::test]
async fn replace() {
    let start = Instant::now();
    let variables = Data {
        text: Some(HashMap::from([
            (String::from("${公司名}"), Text(String::from("xxxx公司"))),
            (String::from("${法人}"), Text(String::from("张三"))),
        ])),
        media: Some(HashMap::from([])),
    };

    let mut list = read_dir("D:\\其他\\test").await;
    // let mut list = read_dir("D:\\其他\\A 人权生成模板2022简").await;

    let mut execute_results = Replace::new(list.data, variables).execute().await;
    println!("{:?}", start.elapsed());
    while let Some(result) = execute_results.pop() {
        let name = list.names.pop().unwrap();
        fs::write("./out/".to_owned() + &*name, result);
    }
}

#[async_std::test]
async fn extract() {
    let mut list = read_dir("D:\\其他\\test").await;
    if let Some(file) = list.data.pop() {
        match file {
            File::Zip(zip) => {
                println!("{:?}", zip.extract_variable_names().await);
            }
            _ => {}
        }
    }
}
#[async_std::test]
async fn extract_medias() {
    let mut list = read_dir("D:\\其他\\test").await;
    if let Some(file) = list.data.pop() {
        match file {
            File::Zip(zip) => {
                println!("{:?}", zip.get_medias().await);
            }
            _ => {}
        }
    }
}

#[async_std::test]
async fn file_encode_test() {
    let dirs = fs::read_dir("D:\\其他\\test").unwrap();

    dirs.for_each(|dir| {
        let path = dir.unwrap().path();
        if path.is_dir() {
            return;
        }

        let path = Path::new(&path);
        if let Ok(data) = fs::read(path) {
            let name = path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name).unwrap();

            let res = file_encode(data);
            fs::write("./out/en/".to_owned() + &*name, res).expect("TODO: panic message");
        }
    });
}

#[async_std::test]
async fn file_decode_test() {
    let dirs = fs::read_dir("./out/en/").unwrap();

    dirs.for_each(|dir| {
        let path = dir.unwrap().path();
        if path.is_dir() {
            return;
        }

        let path = Path::new(&path);
        if let Ok(data) = fs::read(path) {
            let name = path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name).unwrap();

            let res = file_decode(data);


            // let runtime = tokio::runtime::Builder::new_current_thread()
            //     .worker_threads(4) // 设置工作线程数（可选）
            //     .enable_all()       // 启用所有功能（I/O、时间等）
            //     .build()
            //     .unwrap();
            // 
            // runtime.block_on(async {
            //     if let Ok(office) = new_office(res.clone(), name.parse().unwrap()).await {
            //         println!("{},ok", name)
            //     } else {
            //         println!("{},error", name)
            //     }
            // });


            fs::write("./out/de/".to_owned() + &*name, res).expect("TODO: panic message");
        }
    });
}

