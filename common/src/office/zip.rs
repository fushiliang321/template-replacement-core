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
    pub _type: String,
}

pub struct Zip {
    archive: ZipArchive<Cursor<Vec<u8>>>,
    office: Box<dyn Office>,
    files: HashMap<String, Box<[u8]>>,
    relationships: HashMap<String, Vec<RelationshipInfo>>,
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
    if let Ok(mut archive) = ZipArchive::new(Cursor::new(file)) {
        if let Some(office) = office(&mut archive) {
            return Ok(Zip {
                office,
                archive,
                files: HashMap::new(),
                relationships: HashMap::new(),
            });
        }
    }
    Err(NotSupported())
}

impl Zip {
    pub fn office(&self) -> &Box<dyn Office> {
        &self.office
    }

    //获取指定文件内容
    pub fn get_document_content(&mut self, index: usize) -> Option<Box<[u8]>> {
        if let Ok(mut reader) = self.archive.by_index(index) {
            let mut bytes = Vec::new();
            if let Ok(_) = reader.read_to_end(&mut bytes) {
                return Some(bytes.into_boxed_slice());
            }
        }
        None
    }

    //根据文件名获取文件内容
    pub fn get_document_content_by_name(&mut self, file_name: &str) -> Option<Box<[u8]>> {
        if let Ok(mut reader) = self.archive.by_name(file_name) {
            let mut bytes = Vec::new();
            if let Ok(_) = reader.read_to_end(&mut bytes) {
                return Some(bytes.into_boxed_slice());
            }
        }
        None
    }

    //匹配文件名
    pub fn match_document_names(&mut self) -> Vec<String> {
        let mut file_names = Vec::new();
        let regex = self.office.regex();
        for i in 0..self.archive.len() {
            if let Ok(file_in_zip) = self.archive.by_index(i) {
                if file_in_zip.is_file() && regex.is_match(file_in_zip.name()) {
                    file_names.push(file_in_zip.name().to_string());
                }
            }
        }
        file_names
    }

    //匹配文件内容
    pub fn match_document_contents(&mut self) -> Option<HashMap<String, Box<[u8]>>> {
        let mut names = Vec::new();
        let mut contents = Vec::new();
        let regex = self.office.regex();

        for i in 0..self.archive.len() {
            if let Ok(mut file_in_zip) = self.archive.by_index(i) {
                if file_in_zip.is_file() {
                    if regex.is_match(file_in_zip.name()) {
                        let mut buf = Vec::new();
                        if let Ok(_) = file_in_zip.read_to_end(&mut buf) {
                            names.push(file_in_zip.name().to_string());
                            contents.push(buf.into_boxed_slice());
                        }
                    }
                }
            }
        }

        if contents.is_empty() {
            return None;
        }
        let mut files = HashMap::new();
        while let Some(file) = contents.pop() {
            let name = names.pop().unwrap();
            files.insert(name.to_string(), file);
        }
        Some(files)
    }

    //提取变量名
    pub fn extract_variable_names(&mut self) -> Option<Vec<String>> {
        let mut names_set = HashSet::new();
        if let Some(contents) = self.match_document_contents() {
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
    fn get_relationship(&mut self, file_name: &String) -> Box<[u8]> {
        if let Some(content) = self.get_document_content_by_name(file_name) {
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
    pub fn get_medias(&mut self) -> Option<HashMap<String, (String, Box<[u8]>)>> {
        let media_dir = self.office.root_dir().to_owned() + "media/";

        let mut medias = Vec::new();
        let mut names = vec![];

        for i in 0..self.archive.len() {
            if let Ok(mut file_in_zip) = self.archive.by_index(i) {
                if !file_in_zip.is_file() || !file_in_zip.name().starts_with(&media_dir) {
                    continue;
                }
                let mut buf = Vec::new();
                if let Ok(_) = file_in_zip.read_to_end(&mut buf) {
                    medias.push(buf.into_boxed_slice());
                    names.push(file_in_zip.name().to_string());
                }
            }
        }
        let mut media_map = HashMap::new();
        while let Some(file) = medias.pop() {
            let name = names.pop().unwrap().to_string();
            let id = generate_id(&file.to_vec());
            media_map.insert(id, (name, file));
        }
        if media_map.is_empty() {
            return None;
        }
        Some(media_map)
    }

    //获取媒体文件名
    pub fn get_media_names(&mut self) -> Option<HashMap<String, String>> {
        let media_dir = self.office.root_dir().to_owned() + "media/";

        let mut medias = Vec::new();
        let mut names = vec![];

        for i in 0..self.archive.len() {
            if let Ok(mut file_in_zip) = self.archive.by_index(i) {
                if !file_in_zip.is_file() || !file_in_zip.name().starts_with(&media_dir) {
                    continue;
                }
                let mut buf = Vec::new();
                if let Ok(_) = file_in_zip.read_to_end(&mut buf) {
                    medias.push(buf.into_boxed_slice());
                    names.push(file_in_zip.name().to_string());
                }
            }
        }
        let mut media_map = HashMap::new();
        while let Some(file) = medias.pop() {
            let name = names.pop().unwrap().to_string();
            let id = generate_id(&file.to_vec());
            media_map.insert(id, name);
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
    pub fn finish(&mut self) -> Box<[u8]> {
        let mut buffer = Vec::new();
        let mut writer = ZipWriter::new(Cursor::new(&mut buffer));
        let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for (name, relationships) in self.relationships.iter() {
            if relationships.is_empty() {
                continue;
            }
            let name = self.office.root_dir().to_owned() + "_rels/" + name;
            let content = {
                let mut bytes = Vec::new();
                if let Ok(mut reader) = self.archive.by_name(&name) {
                    if let Err(_) = reader.read_to_end(&mut bytes) {
                        bytes = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"></Relationships>"#.as_bytes().to_vec()
                    }
                } else {
                    bytes = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"></Relationships>"#.as_bytes().to_vec()
                }
                bytes
            };
            if let Ok(mut str) = String::from_utf8(content) {
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
            if !self.files.contains_key(entry.name()) {
                writer.raw_copy_file(entry).unwrap();
            }
        }

        writer.finish().unwrap();
        buffer.into_boxed_slice()
    }
}
