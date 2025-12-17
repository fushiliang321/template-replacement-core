use crate::extract::index::{out_tag, raw_variables};
use crate::office::excel::Excel;
use crate::office::office::Office;
use crate::office::word::Word;
use crate::office::zip::Error::{NotSupported, ReadFailure};
use crate::replace::image::generate_id;
use async_zip::base::read::mem::ZipFileReader;
use futures::future::join_all;
use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

pub struct RelationshipInfo {
    pub id: String,
    pub target: String,
    pub _type: String,
}

pub struct Zip {
    reader: ZipFileReader,
    archive: ZipArchive<Cursor<Vec<u8>>>,
    office: Box<dyn Office>,
    files: HashMap<String, Box<[u8]>>,
    relationships: HashMap<String, Vec<RelationshipInfo>>,
}

const TYPE_WORD: u8 = 0;
const TYPE_EXCEL: u8 = 1;

fn office(archive: &mut ZipArchive<Cursor<Vec<u8>>>) -> Option<Box<dyn Office>> {
    if archive.by_name("word/document.xml").is_ok() {
        return Some(Box::new(Word {}));
    }
    if archive.by_name("xl/workbook.xml").is_ok() {
        return Some(Box::new(Excel {}));
    }
    None
}

pub enum Error {
    ReadFailure(String), // 读取失败
    NotSupported(Vec<u8>), // 不支持的文件类型
}

pub async fn new(file: Vec<u8>) -> Result<Zip, Error> {
    if let Ok(mut archive) = ZipArchive::new(Cursor::new(file.clone())) {
        if let Some(office) = office(&mut archive) {
            return match ZipFileReader::new(file).await {
                Ok(reader) => {
                    Ok(Zip {
                        office,
                        reader,
                        archive,
                        files: HashMap::new(),
                        relationships: HashMap::new(),
                    })
                }
                Err(err) => {
                    Err(ReadFailure(err.to_string()))
                }
            };
        }
    }
    Err(NotSupported(file))
}

impl Zip {
    pub fn office(&self) -> &Box<dyn Office> {
        &self.office
    }

    //获取指定文件内容
    pub async fn get_document_content(&self, index: usize) -> Option<Box<[u8]>> {
        if let Ok(mut reader) = self.reader.reader_with_entry(index).await {
            let mut bytes = Vec::new();
            if let Ok(_) = reader.read_to_end_checked(&mut bytes).await {
                return Some(bytes.into_boxed_slice());
            }
        }
        None
    }

    //根据文件名获取文件内容
    pub async fn get_document_content_by_name(&self, file_name: &str) -> Option<Box<[u8]>> {
        let entries = self.reader.file().entries();
        for (i, entry) in entries.iter().enumerate() {
            let entry_file_name = entry.filename().as_str().unwrap();
            if entry_file_name.ends_with('/') {
                continue;
            }
            if !file_name.eq(entry_file_name) {
                continue;
            }
            return self.get_document_content(i).await;
        }
        None
    }

    //匹配文件名
    pub async fn match_document_names(&self) -> Vec<&str> {
        let entries = self.reader.file().entries();
        let mut file_names = Vec::new();
        let regex = self.office.regex();
        for entry in entries {
            let file_name = entry.filename().as_str().unwrap();
            if file_name.ends_with('/') {
                continue;
            }
            if regex.is_match(file_name) {
                file_names.push(file_name);
            }
        }
        file_names
    }

    //匹配文件内容
    pub async fn match_document_contents(&self) -> Option<HashMap<String, Box<[u8]>>> {
        let entries = self.reader.file().entries();
        let mut tasks = Vec::new();
        let mut names = Vec::new();
        let regex = self.office.regex();
        for (i, entry) in entries.iter().enumerate() {
            let file_name = entry.filename().as_str().unwrap();
            if file_name.ends_with('/') {
                continue;
            }
            if regex.is_match(file_name) {
                names.push(file_name);
                tasks.push(self.get_document_content(i));
            }
        }
        if tasks.is_empty() {
            return None;
        }
        let mut res = join_all(tasks).await;
        let mut files = HashMap::new();
        while let Some(file) = res.pop() {
            let name = names.pop().unwrap();
            if let Some(file) = file {
                files.insert(name.to_string(), file);
            }
        }
        Some(files)
    }

    //提取变量名
    pub async fn extract_variable_names(&self) -> Option<Vec<String>> {
        let mut names_set = HashSet::new();
        if let Some(contents) = self.match_document_contents().await {
            for (_, content) in contents {
                if let Ok(s) = std::str::from_utf8(&content) {
                    let variables = raw_variables(s);
                    for variable in variables {
                        names_set.insert(out_tag(variable));
                    }
                }
            }
        }
        if names_set.len() == 0 {
            return None;
        }

        Some(names_set.iter().cloned().collect())
    }

    //写入文件
    pub fn write_file(&mut self, file_name: &String, file_data: Box<[u8]>) {
        self.files.insert(file_name.clone(), file_data);
    }

    //获取关联文件内容
    async fn get_relationship(&self, file_name: &String) -> Box<[u8]> {
        if let Some(content) = self.get_document_content_by_name(file_name).await {
            content
        } else {
            Box::from(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"></Relationships>"#.as_bytes())
        }
    }

    //写入关联文件信息
    pub fn write_relationships(&mut self, file_name: String, relationships: Vec<RelationshipInfo>) {
        self.relationships.insert(file_name, relationships);
    }

    //获取媒体文件
    pub async fn get_medias(&self) -> Option<HashMap<String, (String, Box<[u8]>)>> {
        let entries = self.reader.file().entries();
        let media_dir = self.office.root_dir().to_owned() + "media/";

        let mut tasks = Vec::new();
        let mut names = vec![];
        for (i, entry) in entries.iter().enumerate() {
            let file_name = entry.filename().as_str().unwrap();
            if !file_name.starts_with(&media_dir) || file_name.ends_with('/') {
                continue;
            }
            names.push(file_name);
            tasks.push(self.get_document_content(i));
        }

        let mut res = join_all(tasks).await;
        let mut media_map = HashMap::new();
        while let Some(file) = res.pop() {
            let name = names.pop().unwrap().to_string();
            if let Some(file) = file {
                let id = generate_id(&file.to_vec());
                media_map.insert(id, (name, file));
            }
        }
        if media_map.is_empty() {
            return None;
        }
        Some(media_map)
    }

    //获取媒体文件名
    pub async fn get_media_names(&self) -> Option<HashMap<String, String>> {
        let entries = self.reader.file().entries();
        let media_dir = self.office.root_dir().to_owned() + "media/";

        let mut tasks = Vec::new();
        let mut names = vec![];
        for (i, entry) in entries.iter().enumerate() {
            let file_name = entry.filename().as_str().unwrap();
            if !file_name.starts_with(&media_dir) || file_name.ends_with('/') {
                continue;
            }
            names.push(file_name);
            tasks.push(self.get_document_content(i));
        }

        let mut res = join_all(tasks).await;
        let mut media_map = HashMap::new();
        while let Some(file) = res.pop() {
            let name = names.pop().unwrap().to_string();
            if let Some(file) = file {
                let id = generate_id(&file.to_vec());
                media_map.insert(id, name);
            }
        }
        if media_map.is_empty() {
            return None;
        }
        Some(media_map)
    }

    //写入媒体文件
    pub fn write_media(&mut self, file_name: String, file_data: Box<[u8]>) {
        self.files
            .insert(file_name, file_data);
    }

    //完成写入
    pub async fn finish(&mut self) -> Box<[u8]> {
        let mut buffer = Vec::new();
        let mut writer = ZipWriter::new(Cursor::new(&mut buffer));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for (name, relationships) in self.relationships.iter() {
            if relationships.is_empty() {
                continue;
            }

            let name = self.office.root_dir().to_owned() + "_rels/" + name;
            let content = self.get_relationship(&name).await;
            if let Ok(mut str) = String::from_utf8(content.to_vec()) {
                let mut relationships_text = String::new();
                for relationship in relationships {
                    relationships_text.push_str(&format!(r#"<Relationship Id="{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/{}"/>"#, relationship.id, relationship.target))
                }
                match str.rfind("</Relationships>") {
                    Some(index) => {
                        str.insert_str(index, &relationships_text);
                    }
                    None => {
                        str.push_str(&relationships_text);
                    }
                }
                self.files.insert(name, str.into_bytes().into_boxed_slice());
            }
        }

        for (name, file) in self.files.iter() {
            writer.start_file(name, options).unwrap();
            writer.write_all(file).unwrap();
        }

        for i in 0..self.archive.len() {
            let entry = self.archive.by_index(i).unwrap();
            let name = entry.name().to_string();

            if !self.files.contains_key(&name) {
                writer.raw_copy_file(entry).unwrap();
            }
        }

        writer.finish().unwrap();
        buffer.into_boxed_slice()
    }
}
