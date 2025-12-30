use crate::extract::index::{out_tag, raw_variables};
use crate::office::excel::Excel;
use crate::office::office::Office;
use crate::office::word::Word;
use crate::office::zip::Error::NotSupported;
use crate::replace::image::generate_id;
use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

pub struct RelationshipInfo {
    pub id: String,
    pub target: String,
    pub _type: &'static str,
}

pub struct Zip {
    archive: ZipArchive<Cursor<Vec<u8>>>,
    office: Box<dyn Office>,
    files: HashMap<String, Box<[u8]>>,
    relationships: HashMap<String, Vec<RelationshipInfo>>,
}

// static RELATIONSHIPS_DEFAULT_CONTENT: OnceLock<String> = OnceLock::new();

fn get_relationships_default_content() -> String
{
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"></Relationships>"#.to_string()
}


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
    NotSupported(), // 不支持的文件类型
}

pub fn new(file: Vec<u8>) -> Result<Zip, Error> {
    if let Ok(mut archive) = ZipArchive::new(Cursor::new(file)) &&
        let Some(office) = office(&mut archive) {
        return Ok(Zip {
            office,
            archive,
            files: HashMap::new(),
            relationships: HashMap::new(),
        });
    }
    Err(NotSupported())
}

impl Zip {
    pub fn office(&self) -> &Box<dyn Office> {
        &self.office
    }

    //获取指定文件内容
    pub fn get_document_content(&mut self, index: usize) -> Option<Vec<u8>> {
        if let Ok(mut reader) = self.archive.by_index(index) {
            let mut bytes = Vec::new();
            if let Ok(_) = reader.read_to_end(&mut bytes) {
                return Some(bytes);
            }
        }
        None
    }

    //根据文件名获取文件内容
    pub fn get_document_content_by_name(&mut self, file_name: &str) -> Option<Vec<u8>> {
        if let Ok(mut reader) = self.archive.by_name(file_name) {
            let mut bytes = Vec::new();
            if let Ok(_) = reader.read_to_end(&mut bytes) {
                return Some(bytes);
            }
        }
        None
    }

    //匹配文件名
    pub fn match_document_names(&mut self) -> Vec<String> {
        let mut file_names = Vec::new();
        let regex = self.office.regex();
        for i in 0..self.archive.len() {
            if let Ok(file_in_zip) = self.archive.by_index(i) &&
                file_in_zip.is_file() &&
                regex.is_match(file_in_zip.name()) {
                file_names.push(file_in_zip.name().to_string());
            }
        }
        file_names
    }

    //匹配文件内容
    pub fn match_document_contents(&mut self) -> HashMap<String, Vec<u8>> {
        let regex = self.office.regex();
        let mut files = HashMap::new();
        for i in 0..self.archive.len() {
            if let Ok(mut file_in_zip) = self.archive.by_index(i) &&
                file_in_zip.is_file() &&
                regex.is_match(file_in_zip.name()) {
                let mut buf = Vec::new();
                if let Ok(_) = file_in_zip.read_to_end(&mut buf) {
                    files.insert(file_in_zip.name().to_string(), buf);
                }
            }
        }

        files
    }

    //提取变量名
    pub fn extract_variable_names(&mut self) -> Vec<String> {
        let mut names_set = HashSet::new();
        let contents = self.match_document_contents();
        for (_, content) in contents {
            if let Ok(s) = std::str::from_utf8(&content) {
                let variables = raw_variables(s);
                for variable in variables {
                    names_set.insert(out_tag(variable));
                }
            }
        }
        if names_set.len() == 0 {
            return Vec::new();
        }
        names_set.iter().cloned().collect()
    }

    //写入关联文件信息
    pub fn write_relationships(&mut self, file_name: String, relationships: Vec<RelationshipInfo>) {
        self.relationships.insert(file_name, relationships);
    }

    //获取媒体文件
    pub fn get_medias(&mut self) -> HashMap<String, (String, Vec<u8>)> {
        let media_dir = self.office.root_dir().to_owned() + "media/";
        let mut media_map = HashMap::new();
        for i in 0..self.archive.len() {
            if let Ok(mut file_in_zip) = self.archive.by_index(i) {
                if !file_in_zip.is_file() || !file_in_zip.name().starts_with(&media_dir) {
                    continue;
                }
                let mut buf = Vec::new();
                if let Ok(_) = file_in_zip.read_to_end(&mut buf) {
                    let id = generate_id(&buf);
                    media_map.insert(id, (file_in_zip.name().to_string(), buf));
                }
            }
        }
        media_map
    }

    //获取媒体文件名
    pub fn get_media_names(&mut self) -> HashMap<String, String> {
        let media_dir = self.office.root_dir().to_owned() + "media/";
        let mut media_map = HashMap::new();
        for i in 0..self.archive.len() {
            if let Ok(mut file_in_zip) = self.archive.by_index(i) {
                if !file_in_zip.is_file() || !file_in_zip.name().starts_with(&media_dir) {
                    continue;
                }
                let mut buf = Vec::new();
                if let Ok(_) = file_in_zip.read_to_end(&mut buf) {
                    let id = generate_id(&buf);
                    media_map.insert(id, file_in_zip.name().to_string());
                }
            }
        }
        media_map
    }

    //写入文件
    pub fn write_file(&mut self, file_name: &String, file_data: Box<[u8]>) {
        self.files.insert(file_name.clone(), file_data);
    }

    //完成写入
    pub fn finish(&mut self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let mut writer = ZipWriter::new(Cursor::new(&mut buffer));
        let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for (name, relationships) in self.relationships.iter() {
            if relationships.is_empty() {
                continue;
            }
            let name = self.office.root_dir().to_owned() + "_rels/" + name;
            let mut content = {
                let mut bytes = Vec::new();
                if let Ok(mut reader) = self.archive.by_name(&name) &&
                    reader.read_to_end(&mut bytes).is_ok() &&
                    let Ok(str) = String::from_utf8(bytes)
                {
                    str
                } else {
                    get_relationships_default_content()
                }
            };
            if content.is_empty() {
                continue;
            }
            let mut relationships_text_vec = vec![];
            for relationship in relationships {
                relationships_text_vec.push(r#"<Relationship Id=""#);
                relationships_text_vec.push(&relationship.id);
                relationships_text_vec.push(r#"" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/"#);
                relationships_text_vec.push(&relationship.target);
                relationships_text_vec.push(r#""/>"#);
            }
            match content.rfind("</Relationships>") {
                Some(index) => {
                    content.insert_str(index, &relationships_text_vec.join(""));
                }
                None => {
                    content.push_str( &relationships_text_vec.join(""));
                }
            }
            self.files.insert(name, content.into_bytes().into_boxed_slice());
        }

        for (name, file) in self.files.iter() {
            writer.start_file(name, options).unwrap();
            writer.write_all(file).unwrap();
        }

        for i in 0..self.archive.len() {
            let entry = self.archive.by_index(i).unwrap();
            if !self.files.contains_key(entry.name()) {
                writer.raw_copy_file(entry).unwrap();
            }
        }

        writer.finish().unwrap();
        buffer
    }
}
